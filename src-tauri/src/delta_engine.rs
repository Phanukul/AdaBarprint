use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use std::collections::{BTreeMap, HashSet};

use crate::models::TCNTLabelSchedule;
use crate::promo_v6::resolve_effective_price;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaCandidate {
    pub seq: i64,
    pub pdt_code: String,
    pub bar_code: String,
    pub pdt_name: String,
    pub trigger_src: String, // PRICE_CHG | PROMO_START | PROMO_END_REVERT | NEW_PDT
    pub pri_type: String,    // "1" Normal | "2" Promo
    pub lbs_code: String,
    pub old_price: Option<f64>,
    pub sale_price: f64,
    pub promo_doc_no: Option<String>,
    pub promo_name: Option<String>,
    pub promo_dstart: Option<String>,
    pub promo_dstop: Option<String>,
    pub price_hash: String,
    pub qty: i64,
    pub skipped_dup: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaSummary {
    pub biz_date: String,
    pub yesterday: String,
    pub total_candidates: usize,
    pub skipped_dup: usize,
    pub ready_normal: usize,
    pub ready_promo: usize,
    pub items: Vec<DeltaCandidate>,
}

#[derive(FromRow)]
struct BarBasicRow {
    #[sqlx(rename = "FTPdtCode")]
    pdt_code: String,
    #[sqlx(rename = "FTBarCode")]
    bar_code: String,
    #[sqlx(rename = "FTPdtName")]
    pdt_name: Option<String>,
}

pub fn compute_price_hash(
    barcode: &str,
    lbs_code: &str,
    pri_type: &str,
    sale_price: f64,
    promo_doc_no: Option<&str>,
    promo_dstop: Option<&str>,
) -> String {
    let payload = format!(
        "{}|{}|{}|{:.2}|{}|{}",
        barcode.trim(),
        lbs_code.trim(),
        pri_type.trim(),
        sale_price,
        promo_doc_no.unwrap_or("").trim(),
        promo_dstop.unwrap_or("").trim(),
    );
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

fn rule_enabled(rules_csv: &str, rule: &str) -> bool {
    rules_csv
        .split(',')
        .map(|s| s.trim())
        .any(|s| s.eq_ignore_ascii_case(rule))
}

fn previous_day(biz_date: &str) -> String {
    NaiveDate::parse_from_str(biz_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.pred_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| biz_date.to_string())
}

pub fn next_day(biz_date: &str) -> String {
    NaiveDate::parse_from_str(biz_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.succ_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| biz_date.to_string())
}

pub fn target_date_with_offset(base_date: NaiveDate, offset_days: i32) -> String {
    base_date
        .checked_add_signed(chrono::TimeDelta::days(offset_days as i64))
        .unwrap_or(base_date)
        .format("%Y-%m-%d")
        .to_string()
}

async fn resolve_default_lbs_and_qty(
    pool: &SqlitePool,
    pdt_code: &str,
    barcode: &str,
    fallback_lbs: &str,
    qty_mode: i64,
) -> (String, i64) {
    let row: Option<(String, i64)> = sqlx::query_as(
        r#"
        SELECT FTLbsCode, COALESCE(FNPdlQty, 1)
        FROM TCNMPdtLabel
        WHERE FTPdtCode = ? AND FTBarCode = ?
        ORDER BY CASE WHEN FTPdlStaDef = '1' THEN 0 ELSE 1 END, FDLastUpdOn DESC
        LIMIT 1
        "#,
    )
    .bind(pdt_code)
    .bind(barcode)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((lbs, pdt_qty)) => {
            let final_lbs = if lbs.trim().is_empty() { fallback_lbs.to_string() } else { lbs };
            let final_qty = if qty_mode == 2 { pdt_qty.max(1) } else { 1 };
            (final_lbs, final_qty)
        }
        None => (fallback_lbs.to_string(), 1),
    }
}

async fn collect_bars_for_pdt(pool: &SqlitePool, pdt_code: &str) -> Vec<BarBasicRow> {
    sqlx::query_as::<_, BarBasicRow>(
        r#"
        SELECT b.FTPdtCode, b.FTBarCode, COALESCE(pl.FTPdtName, b.FTPdtCode) AS FTPdtName
        FROM TCNMPdtBar b
        LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
        WHERE b.FTPdtCode = ?
          AND COALESCE(b.FTBarStaUse, '1') = '1'
        GROUP BY b.FTPdtCode, b.FTBarCode
        ORDER BY b.FNBarRefSeq ASC, b.FTBarCode ASC
        "#,
    )
    .bind(pdt_code)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

async fn resolve_effective_ppl_and_bch(pool: &SqlitePool, schedule: &TCNTLabelSchedule) -> (Option<String>, String) {
    // Only filter POS5 branch-scoped promo/adj docs when the user explicitly selected a branch on the schedule;
    // when FTLscBchCode is empty/None ("ใช้สาขาหลักที่ตั้งไว้ในระบบ"), match all store promos just like resolve_accd_promotion.
    let explicit_bch = schedule
        .FTLscBchCode
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();

    if let Some(ref ppl) = schedule.FTLscPplCode {
        if !ppl.trim().is_empty() {
            return (Some(ppl.trim().to_string()), explicit_bch);
        }
    }

    let lookup_bch = if !explicit_bch.is_empty() {
        explicit_bch.clone()
    } else {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'DefaultBranch' LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
        .unwrap_or_default()
        .trim()
        .to_string()
    };

    if !lookup_bch.is_empty() {
        let ppl_opt: Option<String> = sqlx::query_scalar::<_, Option<String>>(
            "SELECT FTPplCode FROM TCNMBranch WHERE FTBchCode = ? LIMIT 1",
        )
        .bind(&lookup_bch)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
        return (ppl_opt, explicit_bch);
    }

    (None, explicit_bch)
}

/// Evaluate all 4 change cases for `biz_date` without `LIMIT 500` restriction.
/// Strictly read-only: NEVER modifies `TCNTLabelSchedWatermark` (safe for Dry-Run `FR-07`).
pub async fn compute_delta_candidates(
    pool: &SqlitePool,
    schedule: &TCNTLabelSchedule,
    biz_date: &str,
) -> Result<DeltaSummary, sqlx::Error> {
    compute_delta_candidates_since(pool, schedule, biz_date, biz_date).await
}

/// Evaluate exact-date Profile candidates where Effective Date == target_date (`System Date + n`),
/// cleanly separating Base Price (`PRICE_CHG`, `PROMO_END_REVERT`) vs Base Price + Price Off (`PROMO_START`),
/// without consulting accumulated watermark logs.
pub async fn compute_profile_candidates_exact(
    pool: &SqlitePool,
    schedule: &TCNTLabelSchedule,
    target_date: &str,
) -> Result<DeltaSummary, sqlx::Error> {
    compute_delta_candidates_impl(pool, schedule, target_date, target_date, true).await
}

/// Select changes since the last completed round, resolving prices as of today.
pub async fn compute_delta_candidates_since(
    pool: &SqlitePool, schedule: &TCNTLabelSchedule, biz_date: &str, since: &str,
) -> Result<DeltaSummary, sqlx::Error> {
    compute_delta_candidates_impl(pool, schedule, biz_date, since, false).await
}

async fn compute_delta_candidates_impl(
    pool: &SqlitePool,
    schedule: &TCNTLabelSchedule,
    biz_date: &str,
    since: &str,
    exact_profile_mode: bool,
) -> Result<DeltaSummary, sqlx::Error> {
    crate::promo_v6::validate_v6_schema(pool).await?;
    let yesterday = previous_day(biz_date);
    let since_stop = previous_day(since);
    let (eff_ppl, eff_bch) = resolve_effective_ppl_and_bch(pool, schedule).await;
    let mut cand_map: BTreeMap<(String, String), (String, String)> = BTreeMap::new(); // (pdt_code, bar_code) -> (trigger_src, pdt_name)

    if rule_enabled(&schedule.FTLscTriggerRules, "ALL_PRODUCTS") {
        return Err(sqlx::Error::Protocol("รองรับงานพิมพ์ 4 ประเภทเท่านั้น: ราคาเปลี่ยน สินค้าใหม่ เริ่มโปรโมชั่น และสิ้นสุดโปรโมชั่น".into()));
    }

    // Case 1: PRICE_CHG (TCNTPdtPrice4PDT or approved TCNTPdtAdjPriHD effective on biz_date)
    if rule_enabled(&schedule.FTLscTriggerRules, "PRICE_CHG") {
        let bars: Vec<BarBasicRow> = sqlx::query_as(
            r#"SELECT DISTINCT b.FTPdtCode, b.FTBarCode, pl.FTPdtName
            FROM TCNMPdtBar b
            LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode=b.FTPdtCode AND pl.FNLngID=1
            JOIN (
                SELECT FTPdtCode, FTPunCode, FTPplCode, FTBchCode FROM TCNTPdtPrice4PDT
                WHERE FTPghDocType='1' AND substr(COALESCE(FDPghDStart,''),1,10) BETWEEN ? AND ?
                UNION
                SELECT dt.FTPdtCode, dt.FTPunCode, hd.FTPplCode, hd.FTBchCode
                FROM TCNTPdtAdjPriDT dt JOIN TCNTPdtAdjPriHD hd
                  ON hd.FTAgnCode=dt.FTAgnCode AND hd.FTBchCode=dt.FTBchCode AND hd.FTXphDocNo=dt.FTXphDocNo
                WHERE COALESCE(hd.FTXphStaApv,'1')='1'
                  AND substr(COALESCE(hd.FDXphDStart,''),1,10) BETWEEN ? AND ?
            ) changed ON changed.FTPdtCode=b.FTPdtCode
              AND (COALESCE(changed.FTPunCode,'')='' OR changed.FTPunCode=b.FTPunCode)
            WHERE COALESCE(b.FTBarStaUse,'1')='1'
              AND (COALESCE(changed.FTPplCode,'')='' OR changed.FTPplCode=?)
              AND (COALESCE(changed.FTBchCode,'')='' OR changed.FTBchCode=?)"#,
        ).bind(since).bind(biz_date).bind(since).bind(biz_date)
         .bind(eff_ppl.as_deref().unwrap_or("")).bind(&eff_bch)
         .fetch_all(pool).await?;
        for b in bars {
            cand_map.entry((b.pdt_code, b.bar_code))
                .or_insert(("PRICE_CHG".into(), b.pdt_name.unwrap_or_default()));
        }
    }

    // Helper closure logic to collect bars from promo refs (both V6 and POS5)
    async fn resolve_promo_refs(pool: &SqlitePool, target_date: &str, since_date: &str, by_stop: bool, bch_code: &str) -> Result<Vec<(String, String)>, sqlx::Error> {
        let col_v6 = if by_stop { "hd.FDPmhDStop" } else { "hd.FDPmhDStart" };
        let col_v5 = if by_stop { "hd.FDPmhDStop" } else { "hd.FDPmhDStart" };

        let q_v6 = format!(
            r#"
            SELECT DISTINCT gd.FTPmdRefCode, COALESCE(gd.FTPmdRefType, '1') AS RefType
            FROM TCNTPmtHD hd
            JOIN TCNTPmtGrpDT gd
              ON gd.FTAgnCode = hd.FTAgnCode AND gd.FTPmhDocNo = hd.FTPmhDocNo
            WHERE COALESCE(hd.FTPmhStaActive, '1') = '1'
              AND (COALESCE(hd.FTPmhStaDoc, '1') = '1' OR COALESCE(hd.FTPmhStaDoc, '') = '')
              AND COALESCE(gd.FTPmdStaType, '1') != '2'
              AND substr(COALESCE({}, ''), 1, 10) BETWEEN ? AND ?
            "#,
            col_v6
        );
        let canonical=crate::promo_v6::uses_canonical_rules(pool).await;
        let q_v5 = format!(
            r#"
            SELECT DISTINCT COALESCE(NULLIF(NULLIF(dt.FTPmdBarCode, ''), '*'), dt.FTPmdRefCode) AS RefCode,
                            CASE WHEN COALESCE(dt.FTPmdBarCode, '') != '' AND dt.FTPmdBarCode != '*' THEN 'BAR' ELSE 'PDT' END AS RefType
            FROM TCNTPdtPmtHD hd
            JOIN TCNTPdtPmtDT dt
              ON dt.FTBchCode = hd.FTBchCode AND dt.FTPmhDocNo = hd.FTPmhDocNo
            WHERE COALESCE(hd.FTPmhStaApv, '1') = '1'
              AND COALESCE(hd.FTPmhStaClosed, '2') != '1'
              AND (? = '' OR COALESCE(hd.FTBchCode, '') = '' OR hd.FTBchCode = ?)
              AND substr(COALESCE({}, ''), 1, 10) BETWEEN ? AND ?
            "#,
            col_v5
        );

        let (has_v6, has_v5): (bool, bool) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('TCNTPmtHD')), EXISTS(SELECT 1 FROM pragma_table_info('TCNTPdtPmtHD'))"
        ).fetch_one(pool).await?;
        if !has_v6 && !has_v5 {
            return Err(sqlx::Error::Protocol("ไม่พบตารางโปรโมชั่นในฐานข้อมูลสินค้า".into()));
        }
        let mut refs = if canonical {
            crate::promo_v6::canonical_event_barcodes(pool, since_date, target_date, by_stop).await?
        } else if has_v6 {
            sqlx::query_as::<_, (String, String)>(&q_v6)
                .bind(since_date).bind(target_date).fetch_all(pool).await?
        } else {
            Vec::new()
        };
        if has_v5 {
            let v5_refs: Vec<(String, String)> = sqlx::query_as(&q_v5)
                .bind(bch_code).bind(bch_code).bind(since_date).bind(target_date)
                .fetch_all(pool).await?;
            refs.extend(v5_refs);
        }
        Ok(refs)
    }

    async fn expand_refs_to_bars(pool: &SqlitePool, refs: &[(String, String)]) -> Result<Vec<BarBasicRow>,sqlx::Error> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (code, kind) in refs {
            let kind=kind.to_ascii_uppercase();
            if !["1","PDT","SKU","SKC","BAR","BARCODE","ALL"].contains(&kind.as_str()) {
                return Err(sqlx::Error::Protocol("โปรโมชั่นใช้กลุ่มสินค้าที่ยังต้องตรวจขอบเขตก่อนสร้างรายการ Auto".into()));
            }
            let all=kind=="ALL" || code=="*";
            let found = sqlx::query_as::<_, BarBasicRow>(
                r#"
                SELECT b.FTPdtCode, b.FTBarCode, COALESCE(pl.FTPdtName, b.FTPdtCode) AS FTPdtName
                FROM TCNMPdtBar b
                LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
                WHERE (? OR (? AND b.FTPdtCode = ?) OR (? AND b.FTBarCode = ?))
                  AND COALESCE(b.FTBarStaUse, '1') = '1'
                GROUP BY b.FTPdtCode, b.FTBarCode
                "#,
            )
            .bind(all)
            .bind(["1","PDT","SKU","SKC"].contains(&kind.as_str()))
            .bind(code)
            .bind(["1","BAR","BARCODE"].contains(&kind.as_str()))
            .bind(code)
            .fetch_all(pool)
            .await?;

            for b in found {
                let k = (b.pdt_code.clone(), b.bar_code.clone());
                if seen.insert(k) {
                    out.push(b);
                }
            }
        }
        Ok(out)
    }

    // Case 2: PROMO_START
    if rule_enabled(&schedule.FTLscTriggerRules, "PROMO_START") {
        let refs = resolve_promo_refs(pool, biz_date, since, false, &eff_bch).await?;
        for b in expand_refs_to_bars(pool, &refs).await? {
            cand_map
                .entry((b.pdt_code, b.bar_code))
                .or_insert(("PROMO_START".to_string(), b.pdt_name.unwrap_or_default()));
        }
    }

    // Case 3: PROMO_END_REVERT
    // - Exact Profile mode: matches FDPmhDStop == target_date (`System Date + n`) directly and resolves Base Price on next_day(target_date).
    // - Legacy delta mode: matches promotions that stopped yesterday (`biz_date - 1`).
    if rule_enabled(&schedule.FTLscTriggerRules, "PROMO_END_REVERT") {
        let (stop_from, stop_to) = if exact_profile_mode {
            (biz_date, biz_date)
        } else {
            (since_stop.as_str(), yesterday.as_str())
        };
        let refs = resolve_promo_refs(pool, stop_to, stop_from, true, &eff_bch).await?;
        for b in expand_refs_to_bars(pool, &refs).await? {
            cand_map
                .entry((b.pdt_code, b.bar_code))
                .or_insert(("PROMO_END_REVERT".to_string(), b.pdt_name.unwrap_or_default()));
        }
    }

    // Case 4: NEW_PDT
    if rule_enabled(&schedule.FTLscTriggerRules, "NEW_PDT") {
        let new_bars = sqlx::query_as::<_, BarBasicRow>(
            r#"
            SELECT b.FTPdtCode, b.FTBarCode, COALESCE(pl.FTPdtName, b.FTPdtCode) AS FTPdtName
            FROM TCNMPdtBar b
            LEFT JOIN TCNMPdt p ON p.FTPdtCode = b.FTPdtCode
            LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
            WHERE COALESCE(b.FTBarStaUse, '1') = '1'
              AND (
                   substr(COALESCE(p.FDCreateOn, ''), 1, 10) BETWEEN ? AND ?
                OR substr(COALESCE(b.FDCreateOn, ''), 1, 10) BETWEEN ? AND ?
              )
            GROUP BY b.FTPdtCode, b.FTBarCode
            "#,
        )
        .bind(since).bind(biz_date)
        .bind(since).bind(biz_date)
        .fetch_all(pool)
        .await?;

        for b in new_bars {
            cand_map
                .entry((b.pdt_code, b.bar_code))
                .or_insert(("NEW_PDT".to_string(), b.pdt_name.unwrap_or_default()));
        }
    }

    let mut items = Vec::with_capacity(cand_map.len());
    let mut skipped_dup = 0usize;
    let mut ready_normal = 0usize;
    let mut ready_promo = 0usize;
    let mut seq = 0i64;

    for ((pdt_code, bar_code), (trigger_src, pdt_name)) in cand_map {
        seq += 1;
        let (lbs_code, qty) = resolve_default_lbs_and_qty(
            pool,
            &pdt_code,
            &bar_code,
            &schedule.FTLscLbsCode,
            schedule.FNLscQtyMode,
        )
        .await;

        let (price_date, verify_promo) = if exact_profile_mode {
            match trigger_src.as_str() {
                "PRICE_CHG" => (biz_date.to_string(), false),
                "PROMO_END_REVERT" => (next_day(biz_date), true),
                _ => (biz_date.to_string(), true),
            }
        } else {
            (biz_date.to_string(), true)
        };

        let eff = resolve_effective_price(
            pool,
            &pdt_code,
            &bar_code,
            eff_ppl.as_deref(),
            &price_date,
            verify_promo,
        )
        .await?;

        if exact_profile_mode && trigger_src == "PRICE_CHG" {
            // Compare published prices in the same unit/group. A first price
            // has no previous publication and still needs a label.
            let previous: Option<f64> = sqlx::query_scalar(
                "SELECT CAST(pr.FCPgdPriceRet AS REAL) FROM TCNTPdtPrice4PDT pr
                 JOIN TCNMPdtBar b ON b.FTPdtCode=pr.FTPdtCode AND b.FTBarCode=?
                 WHERE pr.FTPdtCode=? AND pr.FTPghDocType='1' AND pr.FCPgdPriceRet IS NOT NULL
                   AND (COALESCE(pr.FTPunCode,'')='' OR pr.FTPunCode=b.FTPunCode)
                   AND (COALESCE(pr.FTPplCode,'')='' OR pr.FTPplCode=?)
                   AND (COALESCE(pr.FTBchCode,'')='' OR pr.FTBchCode=?)
                   AND substr(COALESCE(pr.FDPghDStart,''),1,10)<=?
                   AND (COALESCE(pr.FDPghDStop,'')='' OR substr(pr.FDPghDStop,1,10)>=?)
                 ORDER BY CASE WHEN COALESCE(pr.FTPplCode,'')=? THEN 0 ELSE 1 END,
                          CASE WHEN pr.FTPunCode=b.FTPunCode THEN 0 ELSE 1 END,
                          substr(pr.FDPghDStart,1,10) DESC, pr.FDCreateOn DESC LIMIT 1"
            ).bind(&bar_code).bind(&pdt_code).bind(eff_ppl.as_deref().unwrap_or(""))
             .bind(&eff_bch).bind(&yesterday).bind(&yesterday).bind(eff_ppl.as_deref().unwrap_or(""))
             .fetch_optional(pool).await?;
            if previous.is_some_and(|p| (p * 100.0).round() == (eff.normal_price * 100.0).round()) { continue; }
        }
        if exact_profile_mode && trigger_src == "PROMO_END_REVERT" && eff.pri_type == "2" {
            // Still covered by a replacement promotion on revert day -> do not print Base Price revert.
            continue;
        }
        if exact_profile_mode && trigger_src == "PROMO_START" && eff.pri_type != "2" {
            continue;
        }

        let promo_doc = eff.promo.as_ref().map(|p| p.code.clone());
        let promo_name = eff.promo.as_ref().map(|p| p.name.clone());
        let promo_dstart = eff.promo.as_ref().and_then(|p| p.dstart.clone());
        let promo_dstop = eff.promo.as_ref().and_then(|p| p.dstop.clone());

        let old_price = if eff.pri_type == "2" {
            Some(eff.normal_price)
        } else {
            None
        };

        let price_hash = compute_price_hash(
            &bar_code,
            &lbs_code,
            &eff.pri_type,
            eff.sale_price,
            promo_doc.as_deref(),
            promo_dstop.as_deref(),
        );

        let is_dup = if exact_profile_mode {
            false
        } else {
            let wm_hash: Option<(String,)> = sqlx::query_as(
                "SELECT FTLswLastHash FROM TCNTLabelSchedWatermark WHERE FTBarCode = ? AND FTLbsCode = ? LIMIT 1",
            )
            .bind(&bar_code)
            .bind(&lbs_code)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
            wm_hash.map(|(h,)| h == price_hash).unwrap_or(false)
        };
        if is_dup {
            skipped_dup += 1;
        } else if eff.pri_type == "2" {
            ready_promo += 1;
        } else {
            ready_normal += 1;
        }

        items.push(DeltaCandidate {
            seq,
            pdt_code,
            bar_code,
            pdt_name,
            trigger_src,
            pri_type: eff.pri_type,
            lbs_code,
            old_price,
            sale_price: eff.sale_price,
            promo_doc_no: promo_doc,
            promo_name,
            promo_dstart,
            promo_dstop,
            price_hash,
            qty,
            skipped_dup: is_dup,
            context: None,
        });
    }

    Ok(DeltaSummary {
        biz_date: biz_date.to_string(),
        yesterday,
        total_candidates: items.len(),
        skipped_dup,
        ready_normal,
        ready_promo,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_label_schema;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_delta_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMPdt (FTPdtCode TEXT PRIMARY KEY, FTPdtStaActive TEXT, FTPgpChain TEXT, FTPtyCode TEXT, FTPbnCode TEXT, FTPmoCode TEXT, FDCreateOn TEXT, FTCreateBy TEXT, FDLastUpdOn TEXT, FTLastUpdBy TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPdt_L (FTPdtCode TEXT, FNLngID INTEGER, FTPdtName TEXT, FTPdtNameOth TEXT, FTPdtNameABB TEXT, FTPdtRmk TEXT, PRIMARY KEY (FTPdtCode, FNLngID))",
            "CREATE TABLE IF NOT EXISTS TCNMPdtBar (FTPdtCode TEXT, FTBarCode TEXT, FTPunCode TEXT, FTBarStaUse TEXT, FTBarStaAlwSale TEXT, FDCreateOn TEXT, FTCreateBy TEXT, FDLastUpdOn TEXT, FTLastUpdBy TEXT, FNBarRefSeq INTEGER, PRIMARY KEY (FTPdtCode, FTBarCode, FNBarRefSeq))",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTBchCode TEXT, FTPdtCode TEXT, FTPunCode TEXT, FDPghDStart TEXT, FTPghTStart TEXT, FTPplCode TEXT, FDPghDStop TEXT, FTPghTStop TEXT, FTPghDocNo TEXT, FTPghDocType TEXT, FTPghStaAdj TEXT, FCPgdPriceRet REAL, FCPgdPriceWhs REAL, FCPgdPriceNet REAL, FDLastUpdOn TEXT, FTLastUpdBy TEXT, FDCreateOn TEXT, FTCreateBy TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriHD (FTAgnCode TEXT DEFAULT '', FTPplCode TEXT, FTBchCode TEXT, FTXphDocNo TEXT, FDXphDStart TEXT, FDXphDStop TEXT, FTXphStaApv TEXT, FDCreateOn TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriDT (FTAgnCode TEXT DEFAULT '', FTPunCode TEXT, FTBchCode TEXT, FTXphDocNo TEXT, FNXpdSeq INTEGER, FTPdtCode TEXT, FCXpdPriceRet REAL)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD (FTBchCode TEXT, FTPmhDocNo TEXT, FDPmhDStart TEXT, FDPmhDStop TEXT, FTPmhStaClosed TEXT, FTPmhStaApv TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD_L (FTBchCode TEXT, FTPmhDocNo TEXT, FNLngID INTEGER, FTPmhName TEXT, FTPmhNameSlip TEXT, FTPmhRmk TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtDT (FTBchCode TEXT, FTPmhDocNo TEXT, FNPmdSeq INTEGER, FTPmdGrpName TEXT, FTPmdRefCode TEXT, FTPmdBarCode TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPmtCG (FTBchCode TEXT, FTPmhDocNo TEXT, FTPmdGrpName TEXT, FTPgtStaGetType TEXT, FCPgtGetvalue REAL)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        ensure_label_schema(&pool).await.unwrap();
        crate::storage::test_master_promotion_schema(&pool).await;
        pool
    }

    fn test_sched() -> TCNTLabelSchedule {
        TCNTLabelSchedule {
            FTLscCode: "SCH-0400".into(),
            FTLscName: "Test".into(),
            FTLscStaUse: "1".into(),
            FTLscRunTime: "04:00".into(),
            FNLscPreSyncMin: 30,
            FNLscJitterSec: 900,
            FTLscDaysMask: "1111111".into(),
            FTLscTriggerRules: "PRICE_CHG,PROMO_START,PROMO_END_REVERT,NEW_PDT".into(),
            FTLscBchCode: None,
            FTLscPplCode: None,
            FTLscLbsCode: "ADA-100x38".into(),
            FNLscQtyMode: 1,
            FTLscSyncMode: "DIRECT_API".into(),
            FNLscSyncRetry: 3,
            FNLscRetryWaitSec: 0,
            FTLscOnSyncFail: "ABORT".into(),
            FTLscOnPrinterBack: "WAIT_CONFIRM".into(),
            FNLscMisfireMin: 300,
            FTLscStaCoverSheet: "1".into(),
            FDLscLastRunAt: None,
            FDCreateOn: None,
            FTCreateBy: None,
            FDLastUpdOn: None,
            FTLastUpdBy: None,
        }
    }

    #[tokio::test]
    async fn tc01_and_tc03_delta_4_cases_and_watermark_skip() {
        let pool = setup_delta_db().await;
        let biz = "2026-11-19";
        let yest = "2026-11-18";

        // 1) PRICE_CHG
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P01', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P01', '885001', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P01', ?, '1', 19.0)").bind(biz).execute(&pool).await.unwrap();

        // 2) PROMO_START today
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P02', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P02', '885002', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P02', '2026-10-01', '1', 50.0)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtHD (FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FDPmhDStart, FDPmhDStop) VALUES ('', 'PM-START', 1, '1', ?, '2026-11-30')").bind(biz).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdRefCode) VALUES ('', 'PM-START', 'G', 1, 'P02')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FCPmcValue) VALUES ('', 'PM-START', 1, 'FIX_PRI', 39.0)").execute(&pool).await.unwrap();

        // 3) PROMO_END_REVERT (ended yesterday -> must print Normal price 100.0 today!)
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P03', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P03', '885003', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P03', '2026-10-01', '1', 100.0)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtHD (FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FDPmhDStart, FDPmhDStop) VALUES ('', 'PM-END', 1, '1', '2026-11-01', ?)").bind(yest).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdRefCode) VALUES ('', 'PM-END', 'G', 1, 'P03')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FCPmcValue) VALUES ('', 'PM-END', 1, 'FIX_PRI', 79.0)").execute(&pool).await.unwrap();

        // 4) NEW_PDT created today
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P04', ?)").bind(biz).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P04', '885004', 1, '1', ?)").bind(biz).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P04', '2026-11-01', '1', 45.0)").execute(&pool).await.unwrap();

        sqlx::query("UPDATE TCNMPdtBar SET FTPunCode='UNIT'").execute(&pool).await.unwrap();
        sqlx::query("UPDATE TCNTPdtPrice4PDT SET FTPunCode='UNIT'").execute(&pool).await.unwrap();
        let sched = test_sched();
        let res = compute_delta_candidates(&pool, &sched, biz).await.unwrap();
        assert_eq!(res.total_candidates, 4);
        assert_eq!(res.ready_normal, 3); // P01 + P03(revert) + P04
        assert_eq!(res.ready_promo, 1);  // P02
        let p03 = res.items.iter().find(|i| i.pdt_code == "P03").unwrap();
        assert_eq!(p03.trigger_src, "PROMO_END_REVERT");
        assert_eq!(p03.pri_type, "1");
        assert!((p03.sale_price - 100.0).abs() < 0.01);

        // Dry-run guarantee (FR-07): watermark table must still be empty!
        let wm_cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelSchedWatermark").fetch_one(&pool).await.unwrap();
        assert_eq!(wm_cnt.0, 0);

        // TC-03: Once Watermark is written for printed items, second delta skips them!
        for it in &res.items {
            sqlx::query("INSERT INTO TCNTLabelSchedWatermark (FTBarCode, FTLbsCode, FTLswLastHash, FTLswLastPriType, FCLswLastPrice, FDLswPrintedAt, FTLsrRunID) VALUES (?, ?, ?, ?, ?, ?, 'RUN1')")
                .bind(&it.bar_code).bind(&it.lbs_code).bind(&it.price_hash).bind(&it.pri_type).bind(it.sale_price).bind(biz)
                .execute(&pool).await.unwrap();
        }
        let res2 = compute_delta_candidates(&pool, &sched, biz).await.unwrap();
        assert_eq!(res2.skipped_dup, 4);
        assert_eq!(res2.ready_normal + res2.ready_promo, 0);
    }

    #[tokio::test]
    async fn profile_exact_date_offset_base_price_and_price_off() {
        let pool = setup_delta_db().await;
        let base_today = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();
        let yesterday = target_date_with_offset(base_today, -1); // 2026-11-23
        let today = target_date_with_offset(base_today, 0);      // 2026-11-24
        let tomorrow = target_date_with_offset(base_today, 1);   // 2026-11-25
        assert_eq!((yesterday.as_str(), today.as_str(), tomorrow.as_str()), ("2026-11-23", "2026-11-24", "2026-11-25"));

        // P01: Base price starts tomorrow (+1), also has active promo today -> PRICE_CHG must still use Base Price
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P01', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FTPunCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P01', '885101', 'UNIT', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FTPunCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P01', 'UNIT', ?, '1', 89.0)").bind(&tomorrow).execute(&pool).await.unwrap();

        // P02: Promo starts tomorrow (+1) with Base Price 50 -> Price Off 35
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P02', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FTPunCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P02', '885102', 'UNIT', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FTPunCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P02', 'UNIT', '2026-10-01', '1', 50.0)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtHD (FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FDPmhDStart, FDPmhDStop) VALUES ('', 'PM-TMR', 1, '1', ?, '2026-12-01')").bind(&tomorrow).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdRefCode) VALUES ('', 'PM-TMR', 'G', 1, 'P02')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FCPmcValue) VALUES ('', 'PM-TMR', 1, 'FIX_PRI', 35.0)").execute(&pool).await.unwrap();

        // P03: Promo stopped yesterday (-1: 2026-11-23), so today (2026-11-24) prints Base Price 120.0
        sqlx::query("INSERT INTO TCNMPdt (FTPdtCode, FDCreateOn) VALUES ('P03', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FTPunCode, FNBarRefSeq, FTBarStaUse, FDCreateOn) VALUES ('P03', '885103', 'UNIT', 1, '1', '2026-10-01')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTPdtCode, FTPunCode, FDPghDStart, FTPghDocType, FCPgdPriceRet) VALUES ('P03', 'UNIT', '2026-10-01', '1', 120.0)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtHD (FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FDPmhDStart, FDPmhDStop) VALUES ('', 'PM-STOP', 1, '1', '2026-11-01', ?)").bind(&yesterday).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdRefCode) VALUES ('', 'PM-STOP', 'G', 1, 'P03')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FCPmcValue) VALUES ('', 'PM-STOP', 1, 'FIX_PRI', 79.0)").execute(&pool).await.unwrap();

        let mut sched = test_sched();
        sched.FTLscTriggerRules = "PRICE_CHG".into();
        let chg = compute_profile_candidates_exact(&pool, &sched, &tomorrow).await.unwrap();
        assert_eq!(chg.items.len(), 1);
        assert_eq!(chg.items[0].bar_code, "885101");
        assert_eq!(chg.items[0].pri_type, "1");
        assert!((chg.items[0].sale_price - 89.0).abs() < 0.01);

        sched.FTLscTriggerRules = "PROMO_START".into();
        let start = compute_profile_candidates_exact(&pool, &sched, &tomorrow).await.unwrap();
        assert_eq!(start.items.len(), 1);
        assert_eq!(start.items[0].bar_code, "885102");
        assert_eq!(start.items[0].pri_type, "2");
        assert_eq!(start.items[0].old_price, Some(50.0));
        assert!((start.items[0].sale_price - 35.0).abs() < 0.01);

        sched.FTLscTriggerRules = "PROMO_END_REVERT".into();
        let stop = compute_profile_candidates_exact(&pool, &sched, &yesterday).await.unwrap();
        assert_eq!(stop.items.len(), 1);
        assert_eq!(stop.items[0].bar_code, "885103");
        assert_eq!(stop.items[0].pri_type, "1");
        assert!((stop.items[0].sale_price - 120.0).abs() < 0.01);
        assert!(!stop.items[0].skipped_dup);
    }
}
