use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::db::{apply_promo, parse_dt, resolve_accd_promotion, PromoResult};

pub const MISSING_PRICE: &str = "ไม่พบราคาขายที่ใช้ได้ในวันที่เลือก กรุณาตรวจสอบข้อมูลราคาก่อนพิมพ์";

#[derive(FromRow)]
struct V6PromoRow {
    #[sqlx(rename = "FTPmhDocNo")]
    doc_no: String,
    #[sqlx(rename = "FTPmhName")]
    name: Option<String>,
    #[sqlx(rename = "FTPmhRmk")]
    rmk: Option<String>,
    #[sqlx(rename = "FDPmhDStart")]
    dstart: Option<String>,
    #[sqlx(rename = "FDPmhDStop")]
    dstop: Option<String>,
    #[sqlx(rename = "FTPmcCondType")]
    cond_type: Option<String>,
    #[sqlx(rename = "FCPmcValue")]
    value: Option<f64>,
    #[sqlx(rename = "FTPmdGrpName")]
    grp_name: Option<String>,
}

fn map_v6_cond_type(raw: Option<&str>) -> i64 {
    let s = raw.unwrap_or("").trim().to_uppercase();
    if matches!(s.as_str(), "1" | "AMT" | "BAHT" | "DIS_AMT") {
        1
    } else if matches!(s.as_str(), "2" | "PCT" | "PER" | "DIS_PCT") {
        2
    } else if matches!(s.as_str(), "3" | "FIX" | "FIX_PRI" | "SET_PRI") {
        3
    } else if matches!(s.as_str(), "4" | "GIFT" | "FREE") {
        4
    } else { 0 }
}

fn date_in_range(dstart: Option<&str>, dstop: Option<&str>, on_date: DateTime<Utc>) -> bool {
    let target_day = on_date.format("%Y-%m-%d").to_string();
    let start_ok = match dstart.map(str::trim).filter(|s| !s.is_empty()) {
        None => true,
        Some(s) => {
            if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') {
                s[..10] <= *target_day
            } else {
                parse_dt(s).map(|d| d <= on_date).unwrap_or(true)
            }
        }
    };
    let stop_ok = match dstop.map(str::trim).filter(|s| !s.is_empty()) {
        None => true,
        Some(s) => {
            if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') {
                s[..10] >= *target_day
            } else {
                parse_dt(s).map(|d| d >= on_date).unwrap_or(true)
            }
        }
    };
    start_ok && stop_ok
}

/// Resolve active promotion from POS6 V6 tables (`TCNTPmtHD`, `TCNTPmtHD_L`, `TCNTPmtGrpDT`, `TCNTPmtCond`).
pub async fn validate_v6_schema(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let mut exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name='TCNTPmtHD' AND type IN ('table','view')")
        .fetch_one(pool).await?;
    let attached: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pragma_database_list WHERE name='master'")
        .fetch_one(pool).await?;
    if attached > 0 {
        exists += sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM master.sqlite_master WHERE name='TCNTPmtHD' AND type IN ('table','view')")
            .fetch_one(pool).await?;
    }
    if exists == 0 { return Ok(false); }
    if uses_canonical_rules(pool).await {
        sqlx::query("SELECT FTPmhDocNo, FTPmhName, FTPmhRmk, FDPmhDStart, FDPmhDStop, FTPmhStaActive, FTPmhStaDoc, FNPmhVersion, FTPmhRuleJson FROM TCNTPmtHD LIMIT 0").fetch_all(pool).await?;
        return Ok(true);
    }
    sqlx::query("SELECT hd.FTPmhStaDoc, hd.FNPmhVersion, gd.FTPmdRefCode, gd.FTPmdStaType, cnd.FTPmcCondType, cnd.FCPmcValue, cnd.FNPmcSeqNo FROM TCNTPmtHD hd, TCNTPmtGrpDT gd, TCNTPmtCond cnd LIMIT 0")
        .fetch_all(pool).await.map_err(|_| sqlx::Error::Protocol("โครงสร้างโปรโมชั่นของ StoreFront รุ่นนี้ยังไม่รองรับ กรุณาอัปเดตตัวอ่านโปรโมชั่นก่อนใช้งาน".into()))?;
    Ok(true)
}

pub async fn uses_canonical_rules(pool: &SqlitePool) -> bool {
    if sqlx::query("SELECT FTPgdRefCode FROM TCNTPmtGrpDT LIMIT 0").fetch_all(pool).await.is_ok() { return true; }
    sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNTPmtHD WHERE NULLIF(TRIM(FTPmhRuleJson),'') IS NOT NULL")
        .fetch_one(pool).await.map(|n|n>0).unwrap_or(false)
}

#[derive(FromRow)]
struct CanonicalRow {
    doc: String, name: Option<String>, note: Option<String>, start: Option<String>, stop: Option<String>, rule: Option<String>,
}

/// Expand a dated event through the same include/exclude matcher used for
/// manual labels. Flattening include references loses exclusions on expiry.
pub async fn canonical_event_barcodes(pool: &SqlitePool, from: &str, to: &str, by_stop: bool) -> Result<Vec<(String, String)>, sqlx::Error> {
    let column = if by_stop { "FDPmhDStop" } else { "FDPmhDStart" };
    let query = format!("SELECT hd.FTPmhDocNo, hd.FTPmhRuleJson FROM TCNTPmtHD hd WHERE COALESCE(hd.FTPmhStaActive,'1')='1' AND COALESCE(hd.FTPmhStaDoc,'')!='9' AND COALESCE(hd.FTPmhStaPrcDoc,'1') IN ('','1') AND hd.FNPmhVersion=(SELECT MAX(v.FNPmhVersion) FROM TCNTPmtHD v WHERE v.FTAgnCode=hd.FTAgnCode AND v.FTPmhDocNo=hd.FTPmhDocNo) AND substr(hd.{column},1,10) BETWEEN ? AND ?");
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(&query).bind(from).bind(to).fetch_all(pool).await?;
    if rows.is_empty() { return Ok(vec![]); }
    let bars: Vec<(String, String)> = sqlx::query_as("SELECT DISTINCT FTPdtCode,FTBarCode FROM TCNMPdtBar WHERE COALESCE(FTBarStaUse,'1')='1'")
        .fetch_all(pool).await?;
    let mut scopes = std::collections::HashMap::new();
    for (product, _) in &bars {
        if !scopes.contains_key(product) {
            scopes.insert(product.clone(), crate::promo_rule::product_scope(pool, product).await?);
        }
    }
    let mut found = std::collections::BTreeSet::new();
    for (doc, raw) in rows {
        let rule: serde_json::Value = serde_json::from_str(raw.as_deref().unwrap_or(""))
            .map_err(|_| sqlx::Error::Protocol(format!("ข้อมูลโปรโมชั่น {doc} ไม่สมบูรณ์")))?;
        if rule["groups"].is_null() && crate::promo_rule::receipt_only(&rule) { continue; }
        let groups = rule["groups"].as_object().ok_or_else(|| sqlx::Error::Protocol(format!("โปรโมชั่น {doc} ไม่มีกลุ่มสินค้า")))?;
        for (product, barcode) in &bars {
            for group in groups.values() {
                if crate::promo_rule::matches_group_with_scope(group, product, barcode, &scopes[product])
                    .map_err(|e| sqlx::Error::Protocol(format!("{doc}: {e}")))? {
                    found.insert((barcode.clone(), "BAR".to_string()));
                    break;
                }
            }
        }
    }
    Ok(found.into_iter().collect())
}

async fn resolve_canonical_promotion(pool: &SqlitePool, barcode: &str, product: &str, on_date: DateTime<Utc>) -> Result<Option<PromoResult>,sqlx::Error> {
    let rows:Vec<CanonicalRow> = sqlx::query_as("SELECT hd.FTPmhDocNo AS doc,hd.FTPmhName AS name,hd.FTPmhRmk AS note,hd.FDPmhDStart AS start,hd.FDPmhDStop AS stop,hd.FTPmhRuleJson AS rule FROM TCNTPmtHD hd WHERE COALESCE(hd.FTPmhStaActive,'1')='1' AND COALESCE(hd.FTPmhStaDoc,'')!='9' AND COALESCE(hd.FTPmhStaPrcDoc,'1') IN ('','1') AND hd.FNPmhVersion=(SELECT MAX(v.FNPmhVersion) FROM TCNTPmtHD v WHERE v.FTAgnCode=hd.FTAgnCode AND v.FTPmhDocNo=hd.FTPmhDocNo)")
        .fetch_all(pool).await?;
    let mut result=None;
    let scope = crate::promo_rule::product_scope(pool, product).await?;
    for row in rows {
        if !date_in_range(row.start.as_deref(),row.stop.as_deref(),on_date) { continue; }
        let rule:serde_json::Value=serde_json::from_str(row.rule.as_deref().unwrap_or(""))
            .map_err(|_|sqlx::Error::Protocol(format!("ข้อมูลเงื่อนไขโปรโมชั่น {} ไม่สมบูรณ์",row.doc)))?;
        if !date_in_range(rule["when"]["dateStart"].as_str(),rule["when"]["dateStop"].as_str(),on_date) {continue;}
        let Some(reward)=crate::promo_rule::unit_reward_with_scope(&rule,product,barcode,&scope)
            .map_err(|e|sqlx::Error::Protocol(format!("{}: {}",row.doc,e)))? else {continue;};
        if result.is_some() {return Err(sqlx::Error::Protocol("สินค้านี้มีโปรโมชั่นร่วมกันหลายรายการ ต้องตรวจการใช้สิทธิ์ร่วมก่อนกำหนดราคาป้าย".into()));}
        result=Some(PromoResult{code:row.doc,name:row.name.unwrap_or_default(),get_cond:reward.condition,get_value:reward.value,
            dstart:row.start,dstop:row.stop,cap_free:String::new(),cap_name:row.note.unwrap_or_default(),
            price_pmt:if reward.condition==3{Some(reward.value)}else{None},is_discount:true,is_gift:false});
    }
    Ok(result)
}

pub async fn resolve_v6_promotion(
    pool: &SqlitePool,
    barcode: &str,
    pdt_code: &str,
    on_date: DateTime<Utc>,
) -> Result<Option<PromoResult>, sqlx::Error> {
    // Legacy masters may not contain V6 promotions. An incompatible existing
    // schema is an error, not evidence that the product has no promotion.
    if !validate_v6_schema(pool).await? { return Ok(None); }
    if uses_canonical_rules(pool).await {return resolve_canonical_promotion(pool,barcode,pdt_code,on_date).await;}
    let rows_res: Result<Vec<V6PromoRow>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT hd.FTPmhDocNo,
               COALESCE(NULLIF(hdl.FTPmhName, ''), NULLIF(hd.FTPmhName, ''), hd.FTPmhDocNo) AS FTPmhName,
               COALESCE(NULLIF(hdl.FTPmhRmk, ''), hd.FTPmhRmk, '') AS FTPmhRmk,
               CAST(hd.FDPmhDStart AS TEXT) AS FDPmhDStart,
               CAST(hd.FDPmhDStop AS TEXT) AS FDPmhDStop,
               cnd.FTPmcCondType,
               cnd.FCPmcValue,
               gd.FTPmdGrpName
        FROM TCNTPmtHD hd
        JOIN TCNTPmtGrpDT gd
          ON gd.FTAgnCode = hd.FTAgnCode AND gd.FTPmhDocNo = hd.FTPmhDocNo
        LEFT JOIN TCNTPmtHD_L hdl
          ON hdl.FTAgnCode = hd.FTAgnCode
         AND hdl.FTPmhDocNo = hd.FTPmhDocNo
         AND hdl.FNPmhVersion = hd.FNPmhVersion
         AND hdl.FNLngID = 1
        LEFT JOIN TCNTPmtCond cnd
          ON cnd.FTAgnCode = hd.FTAgnCode AND cnd.FTPmhDocNo = hd.FTPmhDocNo
        WHERE COALESCE(hd.FTPmhStaActive, '1') = '1'
          AND (COALESCE(hd.FTPmhStaDoc, '1') = '1' OR COALESCE(hd.FTPmhStaDoc, '') = '')
          AND COALESCE(gd.FTPmdStaType, '1') != '2'
          AND (
                gd.FTPmdRefCode = ?
             OR gd.FTPmdRefCode = ?
             OR gd.FTPmdRefCode = '*'
          )
        ORDER BY COALESCE(hd.FNPmhPriority, 100) ASC, hd.FNPmhVersion DESC, cnd.FNPmcSeqNo ASC
        "#,
    )
    .bind(pdt_code)
    .bind(barcode)
    .fetch_all(pool)
    .await;

    let rows = match rows_res {
        Ok(r) => r,
        Err(error) => {
            eprintln!("Cannot read V6 promotion schema: {}", error);
            return Err(sqlx::Error::Protocol("ไม่สามารถอ่านข้อมูลโปรโมชั่นจากฐานข้อมูลนี้ได้ กรุณาตรวจสอบความเข้ากันได้กับ StoreFront ก่อนพิมพ์".into()));
        }
    };

    for r in rows {
        if !date_in_range(r.dstart.as_deref(), r.dstop.as_deref(), on_date) {
            continue;
        }
        let get_cond = map_v6_cond_type(r.cond_type.as_deref());
        if get_cond == 0 {
            return Err(sqlx::Error::Protocol("พบเงื่อนไขโปรโมชั่นที่ระบบยังไม่รองรับ กรุณาตรวจสอบก่อนพิมพ์".into()));
        }
        let get_value = r.value.ok_or_else(|| sqlx::Error::Protocol("โปรโมชั่นไม่มีมูลค่าส่วนลด กรุณาตรวจสอบข้อมูลก่อนพิมพ์".into()))?;
        if !get_value.is_finite() || get_value < 0.0 || (get_cond == 2 && get_value > 100.0) {
            return Err(sqlx::Error::Protocol("มูลค่าส่วนลดโปรโมชั่นไม่ถูกต้อง กรุณาตรวจสอบข้อมูลก่อนพิมพ์".into()));
        }
        let cap_free = r.grp_name.unwrap_or_default();
        let is_gift = get_cond == 4 || (get_cond == 3 && get_value == 0.0);
        let is_discount = [1, 2, 3].contains(&get_cond) && !is_gift;

        return Ok(Some(PromoResult {
            code: r.doc_no,
            name: r.name.unwrap_or_default(),
            get_cond,
            get_value,
            dstart: r.dstart,
            dstop: r.dstop,
            cap_free,
            cap_name: r.rmk.unwrap_or_default(),
            price_pmt: if get_cond == 3 { Some(get_value) } else { None },
            is_discount,
            is_gift,
        }));
    }

    Ok(None)
}

/// Resolve base normal price from `TCNTPdtPrice4PDT` (with optional `FTPplCode`) and approved `TCNTPdtAdjPriHD/DT` as fallback.
pub async fn resolve_normal_price_on_date(
    pool: &SqlitePool,
    pdt_code: &str,
    unit_code: &str,
    ppl_code: Option<&str>,
    biz_date: &str,
) -> Result<f64, sqlx::Error> {
    let ppl = ppl_code.unwrap_or("").trim();
    let price4: Option<(f64,)> = if !ppl.is_empty() {
        sqlx::query_as(
            r#"
            SELECT CAST(FCPgdPriceRet AS REAL)
            FROM TCNTPdtPrice4PDT
            WHERE FTPdtCode = ?
              AND (FTPunCode = ? OR COALESCE(FTPunCode, '') = '')
              AND FTPghDocType = '1' AND FCPgdPriceRet IS NOT NULL
              AND (FTPplCode = ? OR FTPplCode = '' OR FTPplCode IS NULL)
              AND (COALESCE(FDPghDStart, '') = '' OR substr(FDPghDStart, 1, 10) <= ?)
              AND (COALESCE(FDPghDStop, '') = '' OR substr(FDPghDStop, 1, 10) >= ?)
            ORDER BY CASE WHEN FTPplCode = ? THEN 0 ELSE 1 END,
                     CASE WHEN COALESCE(FTPunCode, '') = '' THEN 1 ELSE 0 END,
                     substr(COALESCE(FDPghDStart, ''), 1, 10) DESC,
                     FDCreateOn DESC
            LIMIT 1
            "#,
        )
        .bind(pdt_code)
        .bind(unit_code)
        .bind(ppl)
        .bind(biz_date)
        .bind(biz_date)
        .bind(ppl)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT CAST(FCPgdPriceRet AS REAL)
            FROM TCNTPdtPrice4PDT
            WHERE FTPdtCode = ?
              AND (FTPunCode = ? OR COALESCE(FTPunCode, '') = '')
              AND FTPghDocType = '1' AND FCPgdPriceRet IS NOT NULL
              AND COALESCE(FTPplCode, '') = ''
              AND (COALESCE(FDPghDStart, '') = '' OR substr(FDPghDStart, 1, 10) <= ?)
              AND (COALESCE(FDPghDStop, '') = '' OR substr(FDPghDStop, 1, 10) >= ?)
            ORDER BY CASE WHEN COALESCE(FTPunCode, '') = '' THEN 1 ELSE 0 END,
                     substr(COALESCE(FDPghDStart, ''), 1, 10) DESC, FDCreateOn DESC
            LIMIT 1
            "#,
        )
        .bind(pdt_code)
        .bind(unit_code)
        .bind(biz_date)
        .bind(biz_date)
        .fetch_optional(pool)
        .await?
    };

    if let Some((p,)) = price4 {
        return Ok(p);
    }

    let adj_price: Option<(f64,)> = sqlx::query_as(
        r#"
        SELECT CAST(dt.FCXpdPriceRet AS REAL)
        FROM TCNTPdtAdjPriDT dt
        JOIN TCNTPdtAdjPriHD hd
          ON hd.FTBchCode = dt.FTBchCode AND hd.FTXphDocNo = dt.FTXphDocNo AND hd.FTAgnCode=dt.FTAgnCode
        WHERE dt.FTPdtCode = ? AND dt.FTPunCode = ? AND dt.FCXpdPriceRet IS NOT NULL
          AND (COALESCE(hd.FTPplCode,'')='' OR hd.FTPplCode=?)
          AND COALESCE(hd.FTXphStaApv, '1') = '1'
          AND (COALESCE(hd.FDXphDStart, '') = '' OR substr(hd.FDXphDStart, 1, 10) <= ?)
          AND (COALESCE(hd.FDXphDStop, '') = '' OR substr(hd.FDXphDStop, 1, 10) >= ?)
        ORDER BY CASE WHEN hd.FTPplCode=? THEN 0 ELSE 1 END, substr(COALESCE(hd.FDXphDStart, ''), 1, 10) DESC, hd.FDCreateOn DESC
        LIMIT 1
        "#,
    )
    .bind(pdt_code)
    .bind(unit_code)
    .bind(ppl)
    .bind(biz_date)
    .bind(biz_date)
    .bind(ppl)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some((p,)) = adj_price {
        return Ok(p);
    }

    let pack_pri: Option<(f64,)> = sqlx::query_as(
        "SELECT CAST(FCPdtPriceRET AS REAL) FROM TCNMPdtPackSize WHERE FTPdtCode = ? AND FTPunCode = ? AND FCPdtPriceRET IS NOT NULL LIMIT 1",
    )
    .bind(pdt_code)
    .bind(unit_code)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some((p,)) = pack_pri {
        return Ok(p);
    }

    // Do not substitute a future/expired price or a Barprint-local mock price.
    Err(sqlx::Error::Protocol(MISSING_PRICE.into()))
}

#[derive(Debug, Clone)]
pub struct EffectivePriceInfo {
    pub normal_price: f64,
    pub sale_price: f64,
    pub pri_type: String, // "1" = Normal, "2" = Promo
    pub promo: Option<PromoResult>,
}

/// Resolve final normal price + active promotion (V6 or POS5) for a given business date (`YYYY-MM-DD`).
pub async fn resolve_effective_price(
    pool: &SqlitePool,
    pdt_code: &str,
    barcode: &str,
    ppl_code: Option<&str>,
    biz_date: &str,
    verify_promo: bool,
) -> Result<EffectivePriceInfo, sqlx::Error> {
    let unit_code: Option<String> = sqlx::query_scalar("SELECT FTPunCode FROM TCNMPdtBar WHERE FTPdtCode=? AND FTBarCode=? LIMIT 1")
        .bind(pdt_code).bind(barcode).fetch_optional(pool).await?.flatten();
    let unit_code = unit_code.ok_or_else(|| sqlx::Error::Protocol("ไม่พบหน่วยสินค้าของบาร์โค้ดที่เลือก".into()))?;
    let normal_price = resolve_normal_price_on_date(pool, pdt_code, &unit_code, ppl_code, biz_date).await?;
    if !normal_price.is_finite() || normal_price < 0.0 {
        return Err(sqlx::Error::Protocol("ราคาสินค้าไม่ถูกต้อง กรุณาตรวจสอบข้อมูลก่อนพิมพ์".into()));
    }
    let on_date = NaiveDate::parse_from_str(biz_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(12, 0, 0))
        .map(|ndt| ndt.and_utc())
        .unwrap_or_else(Utc::now);

    let promo = resolve_accd_promotion(pool, barcode, pdt_code, on_date, verify_promo).await?;
    if let Some(ref pr) = promo {
        if pr.is_discount || pr.is_gift {
            let sale_price = if pr.is_discount {
                apply_promo(normal_price, pr).max(0.0)
            } else {
                normal_price
            };
            return Ok(EffectivePriceInfo {
                normal_price,
                sale_price,
                pri_type: "2".to_string(),
                promo,
            });
        }
    }

    Ok(EffectivePriceInfo {
        normal_price,
        sale_price: normal_price,
        pri_type: "1".to_string(),
        promo: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_label_schema;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn normal_price_uses_the_barcode_unit_and_default_price_group() {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::raw_sql("CREATE TABLE TCNTPdtPrice4PDT(FTPdtCode TEXT,FTPunCode TEXT,FTPghDocType TEXT,FTPplCode TEXT,FDPghDStart TEXT,FDPghDStop TEXT,FDCreateOn TEXT,FCPgdPriceRet REAL);
            INSERT INTO TCNTPdtPrice4PDT VALUES('P','PIECE','1','','2026-01-01','','2026-01-01',10);
            INSERT INTO TCNTPdtPrice4PDT VALUES('P','BOX','1','','2026-02-01','','2026-02-01',100);
            INSERT INTO TCNTPdtPrice4PDT VALUES('P','PIECE','1','MEMBER','2026-03-01','','2026-03-01',8);").execute(&pool).await.unwrap();
        assert_eq!(resolve_normal_price_on_date(&pool,"P","PIECE",None,"2026-11-21").await.unwrap(),10.0);
        assert_eq!(resolve_normal_price_on_date(&pool,"P","BOX",None,"2026-11-21").await.unwrap(),100.0);
        assert_eq!(resolve_normal_price_on_date(&pool,"P","PIECE",Some("MEMBER"),"2026-11-21").await.unwrap(),8.0);
        pool.close().await;
    }

    #[tokio::test]
    async fn incompatible_v6_schema_is_not_reported_as_no_promotion() {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        assert!(!validate_v6_schema(&pool).await.unwrap());
        sqlx::query("CREATE TABLE TCNTPmtHD (FTPmhDocNo TEXT)").execute(&pool).await.unwrap();
        assert!(validate_v6_schema(&pool).await.is_err());
        assert!(resolve_v6_promotion(&pool, "BAR", "PDT", Utc::now()).await.is_err());
        pool.close().await;
    }

    #[tokio::test]
    async fn tc02_v6_promotion_resolver_works_when_pos5_is_empty() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        // Create minimal tables referenced by ensure_label_schema ALTERs
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTPdtCode TEXT, FTPunCode TEXT, FDPghDStart TEXT, FTPghTStart TEXT, FTPplCode TEXT, FDPghDStop TEXT, FTPghTStop TEXT, FTPghDocNo TEXT, FTPghDocType TEXT, FTPghStaAdj TEXT, FCPgdPriceRet REAL, FCPgdPriceWhs REAL, FCPgdPriceNet REAL, FDLastUpdOn TEXT, FTLastUpdBy TEXT, FDCreateOn TEXT, FTCreateBy TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriHD (FTBchCode TEXT, FTXphDocNo TEXT, FDXphDStart TEXT, FDXphDStop TEXT, FTXphStaApv TEXT, FDCreateOn TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtAdjPriDT (FTBchCode TEXT, FTXphDocNo TEXT, FNXpdSeq INTEGER, FTPdtCode TEXT, FCXpdPriceRet REAL)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD (FTBchCode TEXT, FTPmhDocNo TEXT, FDPmhDStart TEXT, FDPmhDStop TEXT, FTPmhStaClosed TEXT, FTPmhStaApv TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtPmtHD_L (FTBchCode TEXT, FTPmhDocNo TEXT, FNLngID INTEGER, FTPmhName TEXT, FTPmhNameSlip TEXT, FTPmhRmk TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtPmtDT (FTBchCode TEXT, FTPmhDocNo TEXT, FNPmdSeq INTEGER, FTPmdGrpName TEXT, FTPmdRefCode TEXT, FTPmdBarCode TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS TCNTPdtPmtCG (FTBchCode TEXT, FTPmhDocNo TEXT, FTPmdGrpName TEXT, FTPgtStaGetType TEXT, FCPgtGetvalue REAL)")
            .execute(&pool).await.unwrap();
        ensure_label_schema(&pool).await.unwrap();
        crate::storage::test_master_promotion_schema(&pool).await;

        sqlx::query("CREATE TABLE IF NOT EXISTS TCNMPdtBar (FTPdtCode TEXT, FTBarCode TEXT, FTPunCode TEXT)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdtBar (FTPdtCode, FTBarCode, FTPunCode) VALUES ('PDT001','8850001000014','001')").execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO TCNTPdtPrice4PDT (FTAgnCode, FTPdtCode, FTPunCode, FDPghDStart, FTPghTStart, FTPplCode, FTPghDocNo, FTPghDocType, FCPgdPriceRet) VALUES ('001', 'PDT001', '001', '2026-11-01', '00:00:00', '', 'DOC01', '1', 100.0)")
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO TCNTPmtHD (FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FTPmhName, FTPmhStaDoc, FDPmhDStart, FDPmhDStop, FNPmhPriority) VALUES ('001', 'PMT-V6-001', 1, '1', 'ลดราคา V6 สดใหม่', '1', '2026-11-18', '2026-11-30', 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdRefType, FTPmdRefCode) VALUES ('001', 'PMT-V6-001', 'GRP1', 1, '1', 'PDT001')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FCPmcValue) VALUES ('001', 'PMT-V6-001', 1, 'FIX_PRI', 79.0)")
            .execute(&pool).await.unwrap();

        let info = resolve_effective_price(&pool, "PDT001", "8850001000014", None, "2026-11-19", true)
            .await
            .unwrap();
        assert_eq!(info.pri_type, "2");
        assert!((info.normal_price - 100.0).abs() < 0.01);
        assert!((info.sale_price - 79.0).abs() < 0.01);
        assert_eq!(info.promo.unwrap().code, "PMT-V6-001");
    }
}
