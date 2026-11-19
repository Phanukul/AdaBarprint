use chrono::{Datelike, Local, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use uuid::Uuid;

use crate::delta_engine::{compute_delta_candidates, DeltaSummary};
use crate::label_renderer::{
    check_printer_ready, cleanup_old_autoprint_data, default_chunk_sender, dispatch_run_queue,
    print_cover_sheet, resolve_printer_route, CoverSheetInfo,
};
use crate::models::{TCNTLabelSchedQueue, TCNTLabelSchedRun, TCNTLabelSchedule};
use crate::sync_master::{load_sync_config, sync_master_with_retry};

static SCHEDULER_BUSY: AtomicBool = AtomicBool::new(false);

struct BusyGuard;
impl Drop for BusyGuard {
    fn drop(&mut self) {
        SCHEDULER_BUSY.store(false, Ordering::SeqCst);
    }
}

fn try_acquire_busy() -> Option<BusyGuard> {
    if SCHEDULER_BUSY
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        Some(BusyGuard)
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerRunResult {
    pub ok: bool,
    pub mode: String, // DRY_RUN | PRINT_NOW | TIMER | MISFIRE
    pub run_id: Option<String>,
    pub status: String,
    pub biz_date: String,
    pub total_candidates: usize,
    pub skipped_dup: usize,
    pub nor_qty: i64,
    pub pmt_qty: i64,
    pub fail_qty: i64,
    pub err_msg: Option<String>,
    pub dry_run_delta: Option<DeltaSummary>,
}

fn day_allowed(days_mask: &str, local_now: &chrono::DateTime<Local>) -> bool {
    let idx = local_now.weekday().num_days_from_monday() as usize; // 0=Mon..6=Sun
    days_mask
        .as_bytes()
        .get(idx)
        .map(|b| *b == b'1')
        .unwrap_or(true)
}

fn compute_branch_jitter_sec(bch_code: &str, max_jitter_sec: i64) -> i64 {
    if max_jitter_sec <= 0 {
        return 0;
    }
    let mut h = DefaultHasher::new();
    bch_code.hash(&mut h);
    (h.finish() % (max_jitter_sec as u64)) as i64
}

pub async fn get_schedule(pool: &SqlitePool, code: &str) -> Result<Option<TCNTLabelSchedule>, sqlx::Error> {
    sqlx::query_as::<_, TCNTLabelSchedule>(
        r#"
        SELECT FTLscCode, FTLscName, FTLscStaUse, FTLscRunTime, FNLscPreSyncMin, FNLscJitterSec,
               FTLscDaysMask, FTLscTriggerRules, FTLscBchCode, FTLscPplCode, FTLscLbsCode,
               FNLscQtyMode, FTLscSyncMode, FNLscSyncRetry, FNLscRetryWaitSec, FTLscOnSyncFail,
               FTLscOnPrinterBack, FNLscMisfireMin, FTLscStaCoverSheet, FDLscLastRunAt,
               FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
        FROM TCNTLabelSchedule
        WHERE FTLscCode = ?
        LIMIT 1
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, Default)]
pub struct DryRunOverrides {
    pub lbs_code: Option<String>,
    pub trigger_rules: Option<String>,
    pub bch_code: Option<String>,
    pub qty_mode: Option<i64>,
}

/// Execute either a read-only `DRY_RUN` or real scheduled/manual Auto-Print run.
pub async fn execute_schedule_run(
    pool: &SqlitePool,
    sched_code: &str,
    trigger_by: &str, // "TIMER" | "MISFIRE" | "MANUAL"
    dry_run: bool,
    biz_date_opt: Option<&str>,
) -> Result<TriggerRunResult, String> {
    execute_schedule_run_with_overrides(pool, sched_code, trigger_by, dry_run, biz_date_opt, None).await
}

pub async fn execute_schedule_run_with_overrides(
    pool: &SqlitePool,
    sched_code: &str,
    trigger_by: &str, // "TIMER" | "MISFIRE" | "MANUAL"
    dry_run: bool,
    biz_date_opt: Option<&str>,
    dry_overrides: Option<DryRunOverrides>,
) -> Result<TriggerRunResult, String> {
    let mut sched = get_schedule(pool, sched_code)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("ไม่พบแผนตั้งเวลา: {}", sched_code))?;

    let biz_date = biz_date_opt
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    if dry_run {
        if let Some(ov) = dry_overrides {
            if let Some(l) = ov.lbs_code.filter(|s| !s.trim().is_empty()) {
                sched.FTLscLbsCode = l;
            }
            if let Some(r) = ov.trigger_rules.filter(|s| !s.trim().is_empty()) {
                sched.FTLscTriggerRules = r;
            }
            if let Some(b) = ov.bch_code {
                sched.FTLscBchCode = if b.trim().is_empty() { None } else { Some(b) };
            }
            if let Some(q) = ov.qty_mode {
                sched.FNLscQtyMode = q;
            }
        }
        let delta = compute_delta_candidates(pool, &sched, &biz_date)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(TriggerRunResult {
            ok: true,
            mode: "DRY_RUN".to_string(),
            run_id: None,
            status: "DRY_RUN".to_string(),
            biz_date,
            total_candidates: delta.total_candidates,
            skipped_dup: delta.skipped_dup,
            nor_qty: delta.ready_normal as i64,
            pmt_qty: delta.ready_promo as i64,
            fail_qty: 0,
            err_msg: None,
            dry_run_delta: Some(delta),
        });
    }

    let _guard = try_acquire_busy().ok_or_else(|| "กำลังมีงาน Auto-Print รันอยู่ กรุณารอสักครู่".to_string())?;

    // Check daily auto uniqueness cleanly before creating a run
    if trigger_by == "TIMER" || trigger_by == "MISFIRE" {
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT FTLsrStatus FROM TCNTLabelSchedRun WHERE FTLscCode = ? AND FDLsrBizDate = ? AND FTLsrTriggerBy IN ('TIMER','MISFIRE') AND FTLsrStatus IN ('SUCCESS','PARTIAL','SKIPPED_EMPTY') LIMIT 1",
        )
        .bind(&sched.FTLscCode)
        .bind(&biz_date)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        if let Some((st,)) = existing {
            return Ok(TriggerRunResult {
                ok: true,
                mode: trigger_by.to_string(),
                run_id: None,
                status: format!("ALREADY_{}", st),
                biz_date,
                total_candidates: 0,
                skipped_dup: 0,
                nor_qty: 0,
                pmt_qty: 0,
                fail_qty: 0,
                err_msg: None,
                dry_run_delta: None,
            });
        }
    }

    let run_id = Uuid::new_v4().to_string().replace('-', "").to_uppercase();
    let started_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO TCNTLabelSchedRun (FTLsrRunID, FTLscCode, FDLsrBizDate, FTLsrTriggerBy, FTLsrStatus, FDLsrStartedAt) VALUES (?, ?, ?, ?, 'RUNNING', ?)",
    )
    .bind(&run_id)
    .bind(&sched.FTLscCode)
    .bind(&biz_date)
    .bind(trigger_by)
    .bind(&started_at)
    .execute(pool)
    .await
    .map_err(|e| format!("ไม่สามารถสร้าง Run Record: {}", e))?;

    // Step 1: Sync Direct against API2PSMaster V6 (if configured / enabled)
    let mut sync_attempts = 0i64;
    let mut sync_proof_json: Option<String> = None;
    let mut synced_at: Option<String> = None;

    if sched.FTLscSyncMode.eq_ignore_ascii_case("DIRECT_API") {
        // Keep MANUAL button responsive (<= 3s retry delay), while TIMER/MISFIRE honors FNLscRetryWaitSec
        let wait_sec = if trigger_by == "MANUAL" {
            sched.FNLscRetryWaitSec.clamp(0, 3) as u64
        } else {
            sched.FNLscRetryWaitSec.max(0) as u64
        };
        let sync_cfg = load_sync_config(
            pool,
            sched.FNLscSyncRetry.max(1) as u32,
            Some(wait_sec),
        )
        .await;

        if !sync_cfg.api_url.is_empty() {
            match sync_master_with_retry(pool, &sync_cfg).await {
                Ok(proof) => {
                    sync_attempts = proof.attempts as i64;
                    synced_at = Some(proof.synced_at.clone());
                    sync_proof_json = serde_json::to_string(&proof).ok();
                }
                Err(sync_err) => {
                    sync_attempts = sync_err.attempts as i64;
                    if sched.FTLscOnSyncFail.eq_ignore_ascii_case("ABORT") {
                        let finished_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let msg = format!("{}: {}", sync_err.code, sync_err.message);
                        let _ = sqlx::query(
                            "UPDATE TCNTLabelSchedRun SET FTLsrStatus = 'SYNC_FAILED', FNLsrSyncAttempts = ?, FTLsrErrMsg = ?, FDLsrFinishedAt = ? WHERE FTLsrRunID = ?",
                        )
                        .bind(sync_attempts)
                        .bind(&msg)
                        .bind(&finished_at)
                        .bind(&run_id)
                        .execute(pool)
                        .await;
                        return Ok(TriggerRunResult {
                            ok: false,
                            mode: trigger_by.to_string(),
                            run_id: Some(run_id),
                            status: "SYNC_FAILED".to_string(),
                            biz_date,
                            total_candidates: 0,
                            skipped_dup: 0,
                            nor_qty: 0,
                            pmt_qty: 0,
                            fail_qty: 0,
                            err_msg: Some(msg),
                            dry_run_delta: None,
                        });
                    }
                }
            }
        }
    }

    // Step 2: Delta Engine (4 cases + PriceHash duplicate skip)
    let delta = compute_delta_candidates(pool, &sched, &biz_date)
        .await
        .map_err(|e| e.to_string())?;

    for item in &delta.items {
        let status = if item.skipped_dup { "SKIPPED_DUP" } else { "READY" };
        sqlx::query(
            r#"
            INSERT INTO TCNTLabelSchedQueue (
                FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc,
                FTLsqPriType, FTLbsCode, FCLsqOldPrice, FCLsqSalePrice,
                FTLsqPromoDocNo, FTLsqPriceHash, FNLsqQty, FTLsqStatus
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&run_id)
        .bind(item.seq)
        .bind(&item.pdt_code)
        .bind(&item.bar_code)
        .bind(&item.trigger_src)
        .bind(&item.pri_type)
        .bind(&item.lbs_code)
        .bind(item.old_price)
        .bind(item.sale_price)
        .bind(&item.promo_doc_no)
        .bind(&item.price_hash)
        .bind(item.qty)
        .bind(status)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Step 3: Dispatch READY queue items chunk-by-chunk
    let dispatch = dispatch_run_queue(
        pool,
        &run_id,
        &biz_date,
        false,
        default_chunk_sender(biz_date.clone(), run_id.clone()),
    )
    .await
    .map_err(|e| e.to_string())?;

    let finished_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let full_err = dispatch
        .err_msg
        .as_ref()
        .map(|m| format!("{}: {}", dispatch.err_code.as_deref().unwrap_or("E-PRT"), m));

    if sched.FTLscStaCoverSheet == "1" && (dispatch.printed_normal + dispatch.printed_promo) > 0 {
        let store_name: String = sqlx::query_scalar::<_, Option<String>>(
            "SELECT FTBchName FROM TCNMBranch_L WHERE FNLngID = 1 LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
        .unwrap_or_else(|| "AdaSoft Store".to_string());

        let summary_lines: Vec<String> = delta
            .items
            .iter()
            .filter(|it| !it.skipped_dup)
            .take(22)
            .map(|it| {
                format!(
                    "{} | {:<13} | {:>7.2} | [{}] {}",
                    if it.pri_type == "2" { "PMT" } else { "NOR" },
                    it.bar_code,
                    it.sale_price,
                    it.trigger_src,
                    it.pdt_name
                )
            })
            .collect();

        let _ = print_cover_sheet(
            pool,
            &CoverSheetInfo {
                run_id: run_id.clone(),
                job_id: dispatch.job_id.clone(),
                biz_date: biz_date.clone(),
                trigger_by: trigger_by.to_string(),
                store_name,
                lbs_code: sched.FTLscLbsCode.clone(),
                nor_qty: dispatch.printed_normal,
                pmt_qty: dispatch.printed_promo,
                skipped_dup: delta.skipped_dup,
                fail_qty: dispatch.failed_count,
                summary_lines,
            },
        )
        .await;
    }

    sqlx::query(
        r#"
        UPDATE TCNTLabelSchedRun
        SET FTLsrStatus = ?,
            FDLsrSyncedAt = ?,
            FTLsrSyncProofJson = ?,
            FNLsrSyncAttempts = ?,
            FNLsrCandTotal = ?,
            FNLsrSkippedDup = ?,
            FNLsrNorQty = ?,
            FNLsrPmtQty = ?,
            FNLsrFailQty = ?,
            FTLjbJobID = ?,
            FTLsrErrMsg = ?,
            FDLsrFinishedAt = ?
        WHERE FTLsrRunID = ?
        "#,
    )
    .bind(&dispatch.run_status)
    .bind(&synced_at)
    .bind(&sync_proof_json)
    .bind(sync_attempts)
    .bind(delta.total_candidates as i64)
    .bind(delta.skipped_dup as i64)
    .bind(dispatch.printed_normal)
    .bind(dispatch.printed_promo)
    .bind(dispatch.failed_count)
    .bind(&dispatch.job_id)
    .bind(&full_err)
    .bind(&finished_at)
    .bind(&run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let _ = sqlx::query("UPDATE TCNTLabelSchedule SET FDLscLastRunAt = ? WHERE FTLscCode = ?")
        .bind(&finished_at)
        .bind(&sched.FTLscCode)
        .execute(pool)
        .await;

    Ok(TriggerRunResult {
        ok: dispatch.run_status != "FAILED",
        mode: trigger_by.to_string(),
        run_id: Some(run_id),
        status: dispatch.run_status,
        biz_date,
        total_candidates: delta.total_candidates,
        skipped_dup: delta.skipped_dup,
        nor_qty: dispatch.printed_normal,
        pmt_qty: dispatch.printed_promo,
        fail_qty: dispatch.failed_count,
        err_msg: full_err,
        dry_run_delta: None,
    })
}

/// Reprint ONLY `FAILED` queue items of a given run (`1..5` printed items are untouched).
pub async fn reprint_failed_run(pool: &SqlitePool, run_id: &str) -> Result<TriggerRunResult, String> {
    let _guard = try_acquire_busy().ok_or_else(|| "กำลังมีงาน Auto-Print รันอยู่ กรุณารอสักครู่".to_string())?;

    let run = sqlx::query_as::<_, TCNTLabelSchedRun>(
        "SELECT FTLsrRunID, FTLscCode, FDLsrBizDate, FTLsrTriggerBy, FTLsrStatus, FDLsrStartedAt, FDLsrSyncedAt, FTLsrSyncProofJson, FNLsrSyncAttempts, FNLsrCandTotal, FNLsrSkippedDup, FNLsrNorQty, FNLsrPmtQty, FNLsrFailQty, FTLjbJobID, FTLsrErrMsg, FDLsrFinishedAt FROM TCNTLabelSchedRun WHERE FTLsrRunID = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("ไม่พบรอบรัน: {}", run_id))?;

    let dispatch = dispatch_run_queue(
        pool,
        run_id,
        &run.FDLsrBizDate,
        true,
        default_chunk_sender(run.FDLsrBizDate.clone(), run_id.to_string()),
    )
    .await
    .map_err(|e| e.to_string())?;

    let (nor_total, pmt_total, fail_total): (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN FTLsqStatus = 'PRINTED' AND FTLsqPriType = '1' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN FTLsqStatus = 'PRINTED' AND FTLsqPriType = '2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN FTLsqStatus = 'FAILED' THEN 1 ELSE 0 END), 0)
        FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ?
        "#,
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .unwrap_or((run.FNLsrNorQty, run.FNLsrPmtQty, dispatch.failed_count));

    let final_status = if fail_total == 0 {
        "SUCCESS"
    } else if nor_total + pmt_total > 0 {
        "PARTIAL"
    } else {
        "FAILED"
    };
    let finished_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let full_err = if fail_total == 0 {
        None
    } else {
        dispatch.err_msg.clone()
    };

    let _ = sqlx::query(
        "UPDATE TCNTLabelSchedRun SET FTLsrStatus = ?, FNLsrNorQty = ?, FNLsrPmtQty = ?, FNLsrFailQty = ?, FTLsrErrMsg = ?, FDLsrFinishedAt = ? WHERE FTLsrRunID = ?",
    )
    .bind(final_status)
    .bind(nor_total)
    .bind(pmt_total)
    .bind(fail_total)
    .bind(&full_err)
    .bind(&finished_at)
    .bind(run_id)
    .execute(pool)
    .await;

    Ok(TriggerRunResult {
        ok: fail_total == 0,
        mode: "REPRINT_FAILED".to_string(),
        run_id: Some(run_id.to_string()),
        status: final_status.to_string(),
        biz_date: run.FDLsrBizDate,
        total_candidates: run.FNLsrCandTotal as usize,
        skipped_dup: run.FNLsrSkippedDup as usize,
        nor_qty: nor_total,
        pmt_qty: pmt_total,
        fail_qty: fail_total,
        err_msg: full_err,
        dry_run_delta: None,
    })
}

fn elapsed_seconds_since(ts_str: Option<&str>, now_local: &chrono::DateTime<Local>) -> Option<i64> {
    let raw = ts_str?.trim();
    if raw.is_empty() {
        return None;
    }
    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S").ok()?;
    let diff = now_local.naive_local() - naive;
    Some(diff.num_seconds())
}

/// 24/7 Background Worker Loop: Pre-Sync with Jitter + 04:00 Execution + Misfire Catch-up + AUTO_RESUME + Cleanup.
pub fn spawn_scheduler_loop(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut last_presync_day = String::new();
        let mut last_cleanup_day = String::new();

        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let local_now = Local::now();
            let biz_date = local_now.format("%Y-%m-%d").to_string();

            // Periodic retention cleanup + stale RUNNING recovery once per loop check
            if last_cleanup_day != biz_date {
                let retention_days: i64 = sqlx::query_scalar::<_, Option<String>>(
                    "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'AutoPrintRetentionDays' LIMIT 1",
                )
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten()
                .flatten()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(60);
                cleanup_old_autoprint_data(&pool, retention_days).await;
                last_cleanup_day = biz_date.clone();
            }

            if !SCHEDULER_BUSY.load(Ordering::SeqCst) {
                let stale_cutoff = (local_now - chrono::Duration::minutes(15))
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                let finish_ts = local_now.format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = sqlx::query(
                    "UPDATE TCNTLabelSchedRun SET FTLsrStatus = 'FAILED', FTLsrErrMsg = COALESCE(FTLsrErrMsg, 'E-SYS-001: งานถูกขัดจังหวะกลางคัน (ไฟดับ/ปิดโปรแกรม)'), FDLsrFinishedAt = ? WHERE FTLsrStatus = 'RUNNING' AND FDLsrStartedAt < ?",
                )
                .bind(&finish_ts)
                .bind(&stale_cutoff)
                .execute(&pool)
                .await;
            }

            let schedules = sqlx::query_as::<_, TCNTLabelSchedule>(
                r#"
                SELECT FTLscCode, FTLscName, FTLscStaUse, FTLscRunTime, FNLscPreSyncMin, FNLscJitterSec,
                       FTLscDaysMask, FTLscTriggerRules, FTLscBchCode, FTLscPplCode, FTLscLbsCode,
                       FNLscQtyMode, FTLscSyncMode, FNLscSyncRetry, FNLscRetryWaitSec, FTLscOnSyncFail,
                       FTLscOnPrinterBack, FNLscMisfireMin, FTLscStaCoverSheet, FDLscLastRunAt,
                       FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
                FROM TCNTLabelSchedule
                WHERE FTLscStaUse = '1'
                "#,
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            for sched in schedules {
                if !day_allowed(&sched.FTLscDaysMask, &local_now) {
                    continue;
                }
                let Ok(run_t) = NaiveTime::parse_from_str(sched.FTLscRunTime.trim(), "%H:%M") else {
                    continue;
                };
                let now_t = local_now.time();
                let diff_sec = (now_t - run_t).num_seconds();

                // 1) Staggered Pre-Sync Window (e.g., 03:15 - 03:45 + Branch Jitter)
                let fallback_bch: String = if let Some(ref b) = sched.FTLscBchCode {
                    if !b.trim().is_empty() { b.clone() } else { "00001".to_string() }
                } else {
                    sqlx::query_scalar::<_, Option<String>>(
                        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode IN ('MasterBchCode', 'DefaultBranch') AND COALESCE(FTSopValue, '') != '' LIMIT 1",
                    )
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten()
                    .flatten()
                    .unwrap_or_else(|| "00001".to_string())
                };
                let jitter_min = compute_branch_jitter_sec(&fallback_bch, sched.FNLscJitterSec) / 60;
                let presync_offset_sec = (-(sched.FNLscPreSyncMin.max(5)) + jitter_min.min(sched.FNLscPreSyncMin - 2).max(0)) * 60;
                if diff_sec >= presync_offset_sec && diff_sec < 0 && last_presync_day != biz_date {
                    let sync_cfg = load_sync_config(&pool, 2, Some(30)).await;
                    if !sync_cfg.api_url.is_empty() {
                        let _ = sync_master_with_retry(&pool, &sync_cfg).await;
                    }
                    last_presync_day = biz_date.clone();
                }

                // 2) Target time / Misfire Catch-up window (strictly at or after run_t: 0 <= diff_sec <= FNLscMisfireMin*60)
                if diff_sec >= 0 && diff_sec <= sched.FNLscMisfireMin * 60 {
                    let existing: Option<(String, String, Option<String>)> = sqlx::query_as(
                        "SELECT FTLsrRunID, FTLsrStatus, COALESCE(FDLsrFinishedAt, FDLsrStartedAt) FROM TCNTLabelSchedRun WHERE FTLscCode = ? AND FDLsrBizDate = ? AND FTLsrTriggerBy IN ('TIMER','MISFIRE') ORDER BY FDLsrStartedAt DESC LIMIT 1",
                    )
                    .bind(&sched.FTLscCode)
                    .bind(&biz_date)
                    .fetch_optional(&pool)
                    .await
                    .unwrap_or(None);

                    match existing {
                        None => {
                            let trig = if diff_sec <= 120 { "TIMER" } else { "MISFIRE" };
                            let _ = execute_schedule_run(&pool, &sched.FTLscCode, trig, false, Some(&biz_date)).await;
                        }
                        Some((_run_id, status, last_ts)) if status == "SYNC_FAILED" => {
                            // If network recovered later inside misfire window (>= 15 min after failed attempt), retry automatically
                            if elapsed_seconds_since(last_ts.as_deref(), &local_now).unwrap_or(9999) >= 900 {
                                let _ = execute_schedule_run(&pool, &sched.FTLscCode, "MISFIRE", false, Some(&biz_date)).await;
                            }
                        }
                        Some((run_id, status, last_ts))
                            if (status == "PARTIAL" || status == "FAILED")
                                && sched.FTLscOnPrinterBack.eq_ignore_ascii_case("AUTO_RESUME") =>
                        {
                            // Wait >= 5 minutes between AUTO_RESUME attempts to avoid spooler spam if paper tray stays empty
                            if elapsed_seconds_since(last_ts.as_deref(), &local_now).unwrap_or(9999) < 300 {
                                continue;
                            }
                            let first_fail: Option<TCNTLabelSchedQueue> = sqlx::query_as(
                                "SELECT FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc, FTLsqPriType, FTLbsCode, FCLsqOldPrice, FCLsqSalePrice, FTLsqPromoDocNo, FTLsqPriceHash, FNLsqQty, FNLsqBatchPage, FTLsqStatus, FTLsqErrMsg FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ? AND FTLsqStatus = 'FAILED' ORDER BY FNLsqSeq ASC LIMIT 1",
                            )
                            .bind(&run_id)
                            .fetch_optional(&pool)
                            .await
                            .unwrap_or(None);

                            if let Some(q_item) = first_fail {
                                let route = resolve_printer_route(&pool, &q_item.FTLbsCode, &q_item.FTLsqPriType).await;
                                if check_printer_ready(&route).await.is_ok() {
                                    let _ = reprint_failed_run(&pool, &run_id).await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_label_schema;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn tc06_daily_auto_run_unique_index_prevents_double_run() {
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMPdt (FTPdtCode TEXT PRIMARY KEY, FTPdtStaActive TEXT, FDCreateOn TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPdt_L (FTPdtCode TEXT, FNLngID INTEGER, FTPdtName TEXT, PRIMARY KEY (FTPdtCode, FNLngID))",
            "CREATE TABLE IF NOT EXISTS TCNMPdtBar (FTPdtCode TEXT, FTBarCode TEXT, FNBarRefSeq INTEGER, FTBarStaUse TEXT, FDCreateOn TEXT, PRIMARY KEY (FTPdtCode, FTBarCode, FNBarRefSeq))",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTPdtCode TEXT, FDPghDStart TEXT, FTPghDStop TEXT, FTPplCode TEXT, FTPghDocType TEXT, FCPgdPriceRet REAL, FDCreateOn TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriHD (FTBchCode TEXT, FTXphDocNo TEXT, FDXphDStart TEXT, FDXphDStop TEXT, FTXphStaApv TEXT, FDCreateOn TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriDT (FTBchCode TEXT, FTXphDocNo TEXT, FNXpdSeq INTEGER, FTPdtCode TEXT, FCXpdPriceRet REAL)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD (FTBchCode TEXT, FTPmhDocNo TEXT, FDPmhDStart TEXT, FDPmhDStop TEXT, FTPmhStaClosed TEXT, FTPmhStaApv TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD_L (FTBchCode TEXT, FTPmhDocNo TEXT, FNLngID INTEGER, FTPmhName TEXT, FTPmhNameSlip TEXT, FTPmhRmk TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtDT (FTBchCode TEXT, FTPmhDocNo TEXT, FNPmdSeq INTEGER, FTPmdGrpName TEXT, FTPmdRefCode TEXT, FTPmdBarCode TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtCG (FTBchCode TEXT, FTPmhDocNo TEXT, FTPmdGrpName TEXT, FTPgtStaGetType TEXT, FCPgtGetvalue REAL)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        ensure_label_schema(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO TCNTLabelSchedule (FTLscCode, FTLscName, FTLscStaUse, FTLscSyncMode) VALUES ('SCH-0400', 'Daily', '1', 'LOCAL_ONLY')",
        )
        .execute(&pool).await.unwrap();

        // First catch-up MISFIRE succeeds cleanly with SKIPPED_EMPTY
        let r1 = execute_schedule_run(&pool, "SCH-0400", "MISFIRE", false, Some("2026-11-19")).await.unwrap();
        assert_eq!(r1.status, "SKIPPED_EMPTY");

        // Second reboot attempt same day skips cleanly without duplicating
        let r2 = execute_schedule_run(&pool, "SCH-0400", "MISFIRE", false, Some("2026-11-19")).await.unwrap();
        assert_eq!(r2.status, "ALREADY_SKIPPED_EMPTY");
    }
}
