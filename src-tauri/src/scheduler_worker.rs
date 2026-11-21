use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::delta_engine::{compute_delta_candidates, DeltaSummary};
use crate::label_renderer::{
    default_chunk_sender, dispatch_run_queue,
    print_cover_sheet, CoverSheetInfo,
};
use crate::models::{TCNTLabelSchedRun, TCNTLabelSchedule};


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

    // Master is owned by StoreFront. Legacy dispatch never downloads data.
    let sync_attempts = 0i64;
    let sync_proof_json: Option<String> = None;
    let synced_at: Option<String> = None;

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

pub fn spawn_scheduler_loop(pool: SqlitePool) {
    crate::print_center::spawn(pool);
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
