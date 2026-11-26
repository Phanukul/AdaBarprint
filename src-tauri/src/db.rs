use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{
    FromRow, SqlitePool,
};
use std::path::PathBuf;

use crate::templates::get_templates;

pub fn load_env() {
    let _ = dotenvy::from_filename(".env")
        .or_else(|_| dotenvy::from_filename("../.env"))
        .or_else(|_| dotenvy::from_filename("../../.env"));
}

pub fn resolve_db_path() -> PathBuf { crate::storage::print_db_path() }

pub fn resolve_exports_dir() -> PathBuf { crate::storage::data_root().join("Exports") }

pub async fn connect_db() -> Result<SqlitePool, sqlx::Error> {
    crate::storage::connect().await
}

pub async fn ensure_label_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let ddls = [
        r#"CREATE TABLE IF NOT EXISTS "TCNMPrinterServer" (
            "FTPsvCode" TEXT NOT NULL PRIMARY KEY,
            "FTPsvName" TEXT,
            "FTPsvRmk" TEXT,
            "FTPsvStaUse" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNSSysOption" (
            "FTSopCode" TEXT NOT NULL PRIMARY KEY,
            "FTSopApp" TEXT DEFAULT 'PrintBar',
            "FTSopName" TEXT,
            "FTSopValue" TEXT,
            "FTSopDesc" TEXT,
            "FNSopSeq" INTEGER DEFAULT 0,
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMPaperSize" (
            "FTPpsCode" TEXT NOT NULL PRIMARY KEY,
            "FTPpsName" TEXT NOT NULL,
            "FCPpsWidth" REAL NOT NULL,
            "FCPpsHeight" REAL NOT NULL,
            "FTPpsWinKind" TEXT DEFAULT 'A4',
            "FTPpsStaCustom" TEXT NOT NULL DEFAULT '2',
            "FNPpsSeq" INTEGER DEFAULT 0,
            "FTPpsStaUse" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMPreprint" (
            "FTPprCode" TEXT NOT NULL PRIMARY KEY,
            "FTPprName" TEXT NOT NULL,
            "FTPprFileName" TEXT,
            "FTPpsCode" TEXT NOT NULL DEFAULT 'A4',
            "FCPprWidth" REAL NOT NULL DEFAULT 210.0,
            "FCPprHeight" REAL NOT NULL DEFAULT 297.0,
            "FNPprCol" INTEGER NOT NULL DEFAULT 1,
            "FNPprRow" INTEGER NOT NULL DEFAULT 1,
            "FCPprLabelW" REAL,
            "FCPprLabelH" REAL,
            "FCPprMagTop" REAL DEFAULT 3.0,
            "FCPprMagLeft" REAL DEFAULT 3.0,
            "FCPprMagBottom" REAL,
            "FCPprMagRight" REAL,
            "FCPprGapH" REAL DEFAULT 2.0,
            "FCPprGapV" REAL DEFAULT 2.0,
            "FNPprPageNo" INTEGER DEFAULT 1,
            "FTPprScope" TEXT NOT NULL DEFAULT 'sheet',
            "FCPprOpacity" REAL DEFAULT 0.72,
            "FTPprSheetImg" TEXT,
            "FTPprCellImg" TEXT,
            "FTPprCropJson" TEXT,
            "FTPprStaUse" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMLabelField" (
            "FTLbfCode" TEXT NOT NULL PRIMARY KEY,
            "FTLbfName" TEXT NOT NULL,
            "FTLbfType" TEXT NOT NULL DEFAULT 'text',
            "FTLbfSample" TEXT,
            "FCLbfDefW" REAL DEFAULT 40.0,
            "FCLbfDefH" REAL DEFAULT 8.0,
            "FNLbfDefFont" INTEGER DEFAULT 10,
            "FTLbfDefWeight" TEXT DEFAULT 'normal',
            "FTLbfDefColor" TEXT DEFAULT '#111111',
            "FNLbfSeq" INTEGER DEFAULT 0,
            "FTLbfStaAdv" TEXT NOT NULL DEFAULT '2',
            "FTLbfStaUse" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMLabelSize" (
            "FTLbsCode" TEXT NOT NULL PRIMARY KEY,
            "FTLbsGroup" TEXT NOT NULL DEFAULT '1',
            "FTLbsName" TEXT,
            "FCLbsWidth" REAL NOT NULL,
            "FCLbsHeight" REAL NOT NULL,
            "FNLbsCol" INTEGER,
            "FNLbsRow" INTEGER,
            "FCLbsMagTop" REAL DEFAULT 3.0,
            "FCLbsMagLeft" REAL DEFAULT 3.0,
            "FCLbsMagBottom" REAL,
            "FCLbsMagRight" REAL,
            "FCLbsGapH" REAL DEFAULT 2.0,
            "FCLbsGapV" REAL DEFAULT 2.0,
            "FNLbsQtyPageNor" INTEGER,
            "FNLbsQtyPagePmt" INTEGER,
            "FTLbsTmplNor" TEXT,
            "FTLbsTmplPmt" TEXT,
            "FTLbsHhtCode" TEXT,
            "FTLbsStaUse" TEXT NOT NULL DEFAULT '1',
            "FTLbsPaperSize" TEXT DEFAULT 'A4',
            "FCLbsPaperWidth" REAL DEFAULT 210.0,
            "FCLbsPaperHeight" REAL DEFAULT 297.0,
            "FCLbsOffsetX" REAL DEFAULT 0.0,
            "FCLbsOffsetY" REAL DEFAULT 0.0,
            "FTPprCode" TEXT,
            "FTLbsStdTmplNor" TEXT,
            "FTLbsStdTmplPmt" TEXT,
            "FTLbsRmk" TEXT,
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMLabelTmpl" (
            "FTLtmCode" TEXT NOT NULL PRIMARY KEY,
            "FTLtmName" TEXT,
            "FTLbsCode" TEXT,
            "FTLtmPriType" TEXT NOT NULL DEFAULT '1',
            "FTLtmLayout" TEXT,
            "FTLtmStaUse" TEXT NOT NULL DEFAULT '1',
            "FTLtmMode" TEXT NOT NULL DEFAULT 'full',
            "FTPprCode" TEXT,
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMLabelPrinter" (
            "FTLbsCode" TEXT NOT NULL,
            "FTPsvCode" TEXT NOT NULL,
            "FTLbpPriType" TEXT NOT NULL,
            "FTLbpPrinter" TEXT,
            "FTLbpTray" TEXT,
            "FTLbpProtocol" TEXT NOT NULL DEFAULT 'PDF',
            "FTLbpStaUse" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT,
            PRIMARY KEY ("FTLbsCode", "FTPsvCode", "FTLbpPriType")
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNMPdtLabel" (
            "FTPdtCode" TEXT NOT NULL,
            "FTBarCode" TEXT NOT NULL,
            "FTLbsCode" TEXT NOT NULL,
            "FNPdlQty" INTEGER NOT NULL DEFAULT 1,
            "FTPdlSource" TEXT NOT NULL DEFAULT '2',
            "FTPdlStaDef" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT,
            PRIMARY KEY ("FTPdtCode", "FTBarCode", "FTLbsCode")
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelJob" (
            "FTLjbJobID" TEXT NOT NULL PRIMARY KEY,
            "FTLjbBchCode" TEXT,
            "FTLjbSource" TEXT NOT NULL DEFAULT '1',
            "FTLjbPriType" TEXT NOT NULL DEFAULT '1',
            "FDLjbFrom" DATETIME,
            "FDLjbTo" DATETIME,
            "FTLjbStaVfy" TEXT NOT NULL DEFAULT '2',
            "FTLjbLbsOvr" TEXT,
            "FTLjbStatus" TEXT NOT NULL DEFAULT '1',
            "FTLjbRmk" TEXT,
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelJobDt" (
            "FTLjbJobID" TEXT NOT NULL,
            "FNLjdSeq" INTEGER NOT NULL,
            "FTLjdBarCode" TEXT,
            "FTLjdPdtCode" TEXT,
            "FTLjdName" TEXT,
            "FTLjdNameOth" TEXT,
            "FTLjdVolume" TEXT,
            "FTLjdSubDept" TEXT,
            "FTLjdReple" TEXT,
            "FDLjdPriDate" DATETIME,
            "FTLjdPmtIntv" TEXT,
            "FTLjdPriStat" TEXT,
            "FTLjdSellUnit" TEXT,
            "FCLjdSetPrice" REAL,
            "FCLjdSalePrice" REAL,
            "FTLjdPhasing" TEXT,
            "FTLjdPerUnit" TEXT,
            "FTLjdCapFree" TEXT,
            "FTLjdCapName" TEXT,
            "FTLjdLabSize" TEXT,
            "FNLjdLabQty" INTEGER NOT NULL DEFAULT 1,
            "FNLjdQtyOrg" INTEGER NOT NULL DEFAULT 1,
            "FCLjdPmtCond" REAL,
            "FCLjdPmtVal" REAL,
            "FDLjdPmtFrom" DATETIME,
            "FDLjdPmtTo" DATETIME,
            "FNLjdBarSeq" INTEGER NOT NULL DEFAULT 1,
            "FNLjdPage" INTEGER NOT NULL DEFAULT 1,
            "FTLjdPriTyp" TEXT NOT NULL DEFAULT '1',
            "FTLjdTmpl" TEXT,
            "FTLjdStaSel" TEXT NOT NULL DEFAULT '1',
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT,
            PRIMARY KEY ("FTLjbJobID", "FNLjdSeq")
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelLog" (
            "FTLlgLogID" TEXT NOT NULL PRIMARY KEY,
            "FTLjbJobID" TEXT,
            "FTLlgLbsCode" TEXT,
            "FTLlgPriTyp" TEXT,
            "FTLlgPsvCode" TEXT,
            "FTLlgPrinter" TEXT,
            "FNLlgQtyLabel" INTEGER NOT NULL DEFAULT 0,
            "FNLlgQtyPage" INTEGER NOT NULL DEFAULT 0,
            "FDLlgPriOn" DATETIME,
            "FTLlgPriBy" TEXT,
            "FTLlgStatus" TEXT NOT NULL DEFAULT '4',
            "FTLlgErrMsg" TEXT,
            "FTLlgFileRef" TEXT,
            "FDCreateOn" DATETIME,
            "FTCreateBy" TEXT,
            "FDLastUpdOn" DATETIME,
            "FTLastUpdBy" TEXT
        )"#,
    ];

    for ddl in ddls {
        sqlx::query(ddl).execute(pool).await?;
    }

    let extra_size_cols = [
        "ALTER TABLE TCNMLabelSize ADD COLUMN FTLbsPaperSize TEXT DEFAULT 'A4'",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsPaperWidth REAL DEFAULT 210.0",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsPaperHeight REAL DEFAULT 297.0",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsOffsetX REAL DEFAULT 0.0",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsOffsetY REAL DEFAULT 0.0",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FTPprCode TEXT",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FTLbsStdTmplNor TEXT",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FTLbsStdTmplPmt TEXT",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsMagBottom REAL",
        "ALTER TABLE TCNMLabelSize ADD COLUMN FCLbsMagRight REAL",
        "ALTER TABLE TCNMPreprint ADD COLUMN FCPprMagBottom REAL",
        "ALTER TABLE TCNMPreprint ADD COLUMN FCPprMagRight REAL",
        "ALTER TABLE TCNMLabelTmpl ADD COLUMN FTLtmMode TEXT NOT NULL DEFAULT 'full'",
        "ALTER TABLE TCNMLabelTmpl ADD COLUMN FTPprCode TEXT",
        "ALTER TABLE TCNMLabelTmpl ADD COLUMN FTLtmOriginal TEXT",
    ];
    for alter_sql in extra_size_cols {
        let _ = sqlx::query(alter_sql).execute(pool).await;
    }

    // Backfill STD template reference columns once for built-in formats
    let _ = sqlx::query(
        "UPDATE TCNMLabelSize SET FTLbsStdTmplNor = FTLbsTmplNor WHERE FTLbsStdTmplNor IS NULL AND FTLbsTmplNor IS NOT NULL AND FTLbsTmplNor NOT LIKE '%-DESIGN-%'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "UPDATE TCNMLabelSize SET FTLbsStdTmplNor = (SELECT t.FTLtmCode FROM TCNMLabelTmpl t WHERE t.FTLbsCode = TCNMLabelSize.FTLbsCode AND t.FTLtmPriType = '1' AND COALESCE(t.FTCreateBy, 'SYSTEM') = 'SYSTEM' LIMIT 1) WHERE FTLbsStdTmplNor IS NULL",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "UPDATE TCNMLabelSize SET FTLbsStdTmplPmt = FTLbsTmplPmt WHERE FTLbsStdTmplPmt IS NULL AND FTLbsTmplPmt IS NOT NULL AND FTLbsTmplPmt NOT LIKE '%-DESIGN-%'",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "UPDATE TCNMLabelSize SET FTLbsStdTmplPmt = (SELECT t.FTLtmCode FROM TCNMLabelTmpl t WHERE t.FTLbsCode = TCNMLabelSize.FTLbsCode AND t.FTLtmPriType = '2' AND COALESCE(t.FTCreateBy, 'SYSTEM') = 'SYSTEM' LIMIT 1) WHERE FTLbsStdTmplPmt IS NULL",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "UPDATE TCNMLabelSize SET FCLbsMagTop = 3.0, FCLbsMagLeft = 3.0 WHERE FCLbsMagTop = 8.0 AND FCLbsMagLeft = 8.0 AND COALESCE(FTLastUpdBy, FTCreateBy, 'SYSTEM') = 'SYSTEM'"
    )
    .execute(pool)
    .await;

    // ─── V6 Promotion Tables Guard (for fresh DB) + 5 Auto-Print Tables ────
    let auto_ddls = [
        // 1) แผนตั้งเวลาพิมพ์อัตโนมัติ
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelSchedule" (
            "FTLscCode"          TEXT NOT NULL PRIMARY KEY,
            "FTLscName"          TEXT NOT NULL,
            "FTLscStaUse"        TEXT NOT NULL DEFAULT '1',
            "FTLscRunTime"       TEXT NOT NULL DEFAULT '04:00',
            "FNLscPreSyncMin"    INTEGER NOT NULL DEFAULT 30,
            "FNLscJitterSec"     INTEGER NOT NULL DEFAULT 900,
            "FTLscDaysMask"      TEXT NOT NULL DEFAULT '1111111',
            "FTLscTriggerRules"  TEXT NOT NULL DEFAULT 'PRICE_CHG,PROMO_START,PROMO_END_REVERT,NEW_PDT',
            "FTLscBchCode"       TEXT,
            "FTLscPplCode"       TEXT,
            "FTLscLbsCode"       TEXT NOT NULL DEFAULT 'UFM-NOR-16',
            "FNLscQtyMode"       INTEGER NOT NULL DEFAULT 1,
            "FTLscSyncMode"      TEXT NOT NULL DEFAULT 'STOREFRONT',
            "FNLscSyncRetry"     INTEGER NOT NULL DEFAULT 3,
            "FNLscRetryWaitSec"  INTEGER NOT NULL DEFAULT 300,
            "FTLscOnSyncFail"    TEXT NOT NULL DEFAULT 'ABORT',
            "FTLscOnPrinterBack" TEXT NOT NULL DEFAULT 'WAIT_CONFIRM',
            "FNLscMisfireMin"    INTEGER NOT NULL DEFAULT 300,
            "FTLscStaCoverSheet" TEXT NOT NULL DEFAULT '1',
            "FDLscLastRunAt"     DATETIME,
            "FDCreateOn"         DATETIME,
            "FTCreateBy"         TEXT,
            "FDLastUpdOn"        DATETIME,
            "FTLastUpdBy"        TEXT
        )"#,
        // 2) ประวัติการรันแต่ละวัน + Unique lock กัน Auto ซ้ำวัน
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelSchedRun" (
            "FTLsrRunID"         TEXT NOT NULL PRIMARY KEY,
            "FTLscCode"          TEXT NOT NULL,
            "FDLsrBizDate"       TEXT NOT NULL,
            "FTLsrTriggerBy"     TEXT NOT NULL DEFAULT 'TIMER',
            "FTLsrStatus"        TEXT NOT NULL,
            "FDLsrStartedAt"     DATETIME NOT NULL,
            "FDLsrSyncedAt"      DATETIME,
            "FTLsrSyncProofJson" TEXT,
            "FNLsrSyncAttempts"  INTEGER NOT NULL DEFAULT 0,
            "FNLsrCandTotal"     INTEGER NOT NULL DEFAULT 0,
            "FNLsrSkippedDup"    INTEGER NOT NULL DEFAULT 0,
            "FNLsrNorQty"        INTEGER NOT NULL DEFAULT 0,
            "FNLsrPmtQty"        INTEGER NOT NULL DEFAULT 0,
            "FNLsrFailQty"       INTEGER NOT NULL DEFAULT 0,
            "FTLjbJobID"         TEXT,
            "FTLsrErrMsg"        TEXT,
            "FDLsrFinishedAt"    DATETIME
        )"#,
        r#"CREATE UNIQUE INDEX IF NOT EXISTS "UX_SchedRun_DailyAuto"
            ON "TCNTLabelSchedRun"("FTLscCode", "FDLsrBizDate")
            WHERE "FTLsrTriggerBy" IN ('TIMER', 'MISFIRE')
              AND "FTLsrStatus" IN ('SUCCESS', 'PARTIAL', 'SKIPPED_EMPTY')"#,
        // 3) คิวรายการรายบาร์โค้ดต่อ 1 รอบรัน
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelSchedQueue" (
            "FTLsrRunID"      TEXT NOT NULL,
            "FNLsqSeq"        INTEGER NOT NULL,
            "FTPdtCode"       TEXT NOT NULL,
            "FTBarCode"       TEXT NOT NULL,
            "FTLsqTriggerSrc" TEXT NOT NULL,
            "FTLsqPriType"    TEXT NOT NULL,
            "FTLbsCode"       TEXT NOT NULL,
            "FCLsqOldPrice"   REAL,
            "FCLsqSalePrice"  REAL NOT NULL,
            "FTLsqPromoDocNo" TEXT,
            "FTLsqPriceHash"  TEXT NOT NULL,
            "FNLsqQty"        INTEGER NOT NULL DEFAULT 1,
            "FNLsqBatchPage"  INTEGER,
            "FTLsqStatus"     TEXT NOT NULL DEFAULT 'READY',
            "FTLsqErrMsg"     TEXT,
            PRIMARY KEY ("FTLsrRunID", "FNLsqSeq")
        )"#,
        r#"CREATE INDEX IF NOT EXISTS "IX_SchedQueue_Status"
            ON "TCNTLabelSchedQueue"("FTLsrRunID", "FTLsqStatus")"#,
        // 4) Watermark กันพิมพ์ซ้ำ
        r#"CREATE TABLE IF NOT EXISTS "TCNTLabelSchedWatermark" (
            "FTBarCode"        TEXT NOT NULL,
            "FTLbsCode"        TEXT NOT NULL,
            "FTLswLastHash"    TEXT NOT NULL,
            "FTLswLastPriType" TEXT NOT NULL,
            "FCLswLastPrice"   REAL NOT NULL,
            "FDLswPrintedAt"   DATETIME NOT NULL,
            "FTLsrRunID"       TEXT NOT NULL,
            PRIMARY KEY ("FTBarCode", "FTLbsCode")
        )"#,
        // 5) Sync Checkpoint Local State
    ];
    for ddl in auto_ddls {
        let _ = sqlx::query(ddl).execute(pool).await;
    }

    Ok(())
}

pub async fn is_seeded(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNMLabelSize")
        .fetch_one(pool)
        .await?;
    Ok(row.0 > 0)
}

#[derive(Debug, Clone)]
pub struct PromoResult {
    pub code: String,
    pub name: String,
    pub get_cond: i64,
    pub get_value: f64,
    pub dstart: Option<String>,
    pub dstop: Option<String>,
    pub cap_free: String,
    pub cap_name: String,
    pub price_pmt: Option<f64>,
    pub is_discount: bool,
    pub is_gift: bool,
}

#[derive(FromRow)]
struct PromoRow {
    #[sqlx(rename = "FTPmhCode")]
    code: String,
    #[sqlx(rename = "FTPmhName")]
    name: Option<String>,
    #[sqlx(rename = "FTPmhNameSlip")]
    name_slip: Option<String>,
    #[sqlx(rename = "FTPmhRmk")]
    rmk: Option<String>,
    #[sqlx(rename = "FDPmhDStart")]
    dstart: Option<String>,
    #[sqlx(rename = "FDPmhDStop")]
    dstop: Option<String>,
    #[sqlx(rename = "FCPmdPricePmt")]
    price_pmt: Option<f64>,
    #[sqlx(rename = "FNPmdGetCond")]
    get_cond: Option<i64>,
    #[sqlx(rename = "FCPmdGetValue")]
    get_value: Option<f64>,
}

pub(crate) fn parse_dt(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(ms) = s.parse::<i64>() {
        return DateTime::from_timestamp_millis(ms);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(ndt.and_utc());
    }
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return nd.and_hms_opt(0, 0, 0).map(|ndt| ndt.and_utc());
    }
    None
}

pub fn format_date_th(s: Option<&str>) -> String {
    let Some(raw) = s else {
        return "-".to_string();
    };
    let Some(dt) = parse_dt(raw) else {
        return raw.to_string();
    };
    dt.format("%d/%m/%Y").to_string()
}

pub fn promo_interval(dstart: Option<&str>, dstop: Option<&str>) -> String {
    let Some(start) = dstart else {
        return String::new();
    };
    format!("เริ่ม {} ถึง {}", format_date_th(Some(start)), format_date_th(dstop))
}

pub fn apply_promo(sale_price: f64, promo: &PromoResult) -> f64 {
    match promo.get_cond {
        1 => (((sale_price - promo.get_value).max(0.0)) * 100.0).round() / 100.0,
        2 => (sale_price * (1.0 - promo.get_value / 100.0) * 100.0).round() / 100.0,
        3 => promo.price_pmt.unwrap_or(promo.get_value).min(sale_price),
        _ => sale_price,
    }
}

pub async fn resolve_promotion(
    pool: &SqlitePool,
    barcode: &str,
    on_date: DateTime<Utc>,
    verify: bool,
) -> Result<Option<PromoResult>, sqlx::Error> {
    if !verify {
        return Ok(None);
    }

    let rows: Vec<PromoRow> = sqlx::query_as(
        r#"
        SELECT hd.FTPmhCode, hd.FTPmhName, hd.FTPmhNameSlip, hd.FTPmhRmk,
               CAST(hd.FDPmhDStart AS TEXT) AS FDPmhDStart,
               CAST(hd.FDPmhDStop AS TEXT) AS FDPmhDStop,
               dt.FCPmdPricePmt, dt.FNPmdGetCond, dt.FCPmdGetValue
        FROM TCNMPmtDT dt
        JOIN TCNMPmtHD hd ON dt.FTPmhCode = hd.FTPmhCode
        WHERE (dt.FTBarCode = ? OR dt.FTBarCode = '*')
          AND hd.FTPmhStaUse = '1'
          AND hd.FTPmhStaApv = '1'
          AND dt.FTPmdStaUse = '1'
        ORDER BY dt.FTBarCode DESC
        "#,
    )
    .bind(barcode)
    .fetch_all(pool)
    .await?;

    for r in rows {
        let start_ok = r
            .dstart
            .as_deref()
            .and_then(parse_dt)
            .map(|d| d <= on_date)
            .unwrap_or(true);
        let stop_ok = r
            .dstop
            .as_deref()
            .and_then(parse_dt)
            .map(|d| d >= on_date)
            .unwrap_or(true);
        if !start_ok || !stop_ok {
            continue;
        }

        let get_cond = r.get_cond.unwrap_or(0);
        let get_value = r.get_value.unwrap_or(0.0);
        let cap_free = r.name_slip.unwrap_or_default();
        let is_gift = get_cond == 4 || (get_cond == 3 && get_value == 0.0 && !cap_free.is_empty());
        let is_discount = [1, 2, 3].contains(&get_cond) && !is_gift;

        return Ok(Some(PromoResult {
            code: r.code,
            name: r.name.unwrap_or_default(),
            get_cond,
            get_value,
            dstart: r.dstart,
            dstop: r.dstop,
            cap_free,
            cap_name: r.rmk.unwrap_or_default(),
            price_pmt: r.price_pmt,
            is_discount,
            is_gift,
        }));
    }

    Ok(None)
}

#[derive(FromRow)]
struct AccdPromoRow {
    #[sqlx(rename = "FTPmhDocNo")]
    doc_no: String,
    #[sqlx(rename = "FTPmhName")]
    name: Option<String>,
    #[sqlx(rename = "FTPmhNameSlip")]
    name_slip: Option<String>,
    #[sqlx(rename = "FTPmhRmk")]
    rmk: Option<String>,
    #[sqlx(rename = "FDPmhDStart")]
    dstart: Option<String>,
    #[sqlx(rename = "FDPmhDStop")]
    dstop: Option<String>,
    #[sqlx(rename = "FTPgtStaGetType")]
    get_type: Option<String>,
    #[sqlx(rename = "FCPgtGetvalue")]
    get_value: Option<f64>,
    #[sqlx(rename = "FTPmdGrpName")]
    grp_name: Option<String>,
}

pub async fn resolve_accd_promotion(
    accd: &SqlitePool,
    barcode: &str,
    pdt_code: &str,
    on_date: DateTime<Utc>,
    verify: bool,
) -> Result<Option<PromoResult>, sqlx::Error> {
    if !verify {
        return Ok(None);
    }

    // 1) Try V6 Promotion tables (TCNTPmtHD / TCNTPmtHD_L / TCNTPmtGrpDT / TCNTPmtCond)
    if let Some(v6_promo) = crate::promo_v6::resolve_v6_promotion(accd, barcode, pdt_code, on_date).await? {
        return Ok(Some(v6_promo));
    }

    // 2) Fallback to POS5 Promotion tables (TCNTPdtPmtDT / HD / HD_L / CG)
    let rows: Vec<AccdPromoRow> = sqlx::query_as(
        r#"
        SELECT hd.FTPmhDocNo,
               COALESCE(hdl.FTPmhName, hd.FTPmhDocNo) AS FTPmhName,
               hdl.FTPmhNameSlip,
               hdl.FTPmhRmk,
               CAST(hd.FDPmhDStart AS TEXT) AS FDPmhDStart,
               CAST(hd.FDPmhDStop AS TEXT) AS FDPmhDStop,
               cg.FTPgtStaGetType,
               cg.FCPgtGetvalue,
               dt.FTPmdGrpName
        FROM TCNTPdtPmtDT dt
        JOIN TCNTPdtPmtHD hd ON dt.FTPmhDocNo = hd.FTPmhDocNo AND dt.FTBchCode = hd.FTBchCode
        LEFT JOIN TCNTPdtPmtHD_L hdl ON hd.FTPmhDocNo = hdl.FTPmhDocNo AND hd.FTBchCode = hdl.FTBchCode AND hdl.FNLngID = 1
        LEFT JOIN TCNTPdtPmtCG cg ON hd.FTPmhDocNo = cg.FTPmhDocNo AND hd.FTBchCode = cg.FTBchCode AND dt.FTPmdGrpName = cg.FTPmdGrpName
        WHERE (dt.FTPmdBarCode = ? OR dt.FTPmdRefCode = ? OR dt.FTPmdBarCode = '*')
          AND COALESCE(hd.FTPmhStaApv, '1') = '1'
          AND COALESCE(hd.FTPmhStaClosed, '2') != '1'
        ORDER BY dt.FNPmdSeq ASC
        "#,
    )
    .bind(barcode)
    .bind(pdt_code)
    .fetch_all(accd)
    .await?;

    for r in rows {
        let start_ok = r
            .dstart
            .as_deref()
            .and_then(parse_dt)
            .map(|d| d <= on_date)
            .unwrap_or(true);
        let stop_ok = r
            .dstop
            .as_deref()
            .and_then(parse_dt)
            .map(|d| d >= on_date)
            .unwrap_or(true);
        if !start_ok || !stop_ok {
            continue;
        }

        let get_type = r.get_type.as_deref().unwrap_or("4");
        let get_cond: i64 = match get_type {
            "1" => 1, // ลดบาท
            "2" => 2, // ลด%
            "3" => 3, // ปรับราคา
            _ => 4,   // ของแถม / เงื่อนไขพิเศษ
        };
        let get_value = r.get_value.unwrap_or(0.0);
        let cap_free = r
            .name_slip
            .filter(|s| !s.is_empty())
            .or(r.grp_name)
            .unwrap_or_default();
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

pub async fn seed_defaults(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Seed Paper Sizes master (idempotent)
    let paper_sizes = [
        ("A4", "A4 (210 × 297 mm)", 210.0, 297.0, "A4", "2", 1_i64),
        ("A4_L", "A4 แนวนอน (297 × 210 mm)", 297.0, 210.0, "A4", "2", 2),
        ("A5", "A5 (148 × 210 mm)", 148.0, 210.0, "A5", "2", 3),
        ("A5_L", "A5 แนวนอน (210 × 148 mm)", 210.0, 148.0, "A5", "2", 4),
        ("A6", "A6 (105 × 148 mm)", 105.0, 148.0, "A6", "2", 5),
        ("A3", "A3 (297 × 420 mm)", 297.0, 420.0, "A3", "2", 6),
        ("B5_JIS", "B5 JIS (182 × 257 mm)", 182.0, 257.0, "B5_JIS", "2", 7),
        ("B5_ISO", "B5 ISO (176 × 250 mm)", 176.0, 250.0, "B5_ISO", "2", 8),
        ("LETTER", "Letter (215.9 × 279.4 mm)", 215.9, 279.4, "LETTER", "2", 9),
        ("LEGAL", "Legal / F4 (215.9 × 355.6 mm)", 215.9, 355.6, "LEGAL", "2", 10),
        ("ROLL", "ม้วนสติ๊กเกอร์ (Thermal Roll)", 50.0, 38.0, "CUSTOM", "1", 15),
        ("CUSTOM", "กำหนดเอง (mm)", 210.0, 297.0, "CUSTOM", "1", 99),
    ];
    for (code, name, w, h, wk, sta_cus, seq) in paper_sizes {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMPaperSize (FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, '1', ?, 'SYSTEM', ?, 'SYSTEM')",
        )
        .bind(code).bind(name).bind(w).bind(h).bind(wk).bind(sta_cus).bind(seq).bind(&now).bind(&now)
        .execute(pool)
        .await?;
    }

    // Seed Label Designer data fields master (idempotent)
    let label_fields: Vec<(&str, &str, &str, &str, f64, f64, i64, &str, &str, i64, &str)> = vec![
        ("product.name", "ชื่อสินค้า (ไทย)", "text", "กาแฟอาราบิก้าคั่วกลาง 250 ก.", 55.0, 8.0, 11, "bold", "#111111", 1, "2"),
        ("product.name_oth", "ชื่อรอง / อังกฤษ", "text", "ROASTED ARABICA COFFEE 250G", 55.0, 6.0, 8, "normal", "#475569", 2, "2"),
        ("price.value", "ราคาขาย (ปกติ/โปรฯ)", "price", "129.00", 42.0, 16.0, 28, "black", "#111111", 3, "2"),
        ("price.old_value", "ราคาเดิม (ขีดฆ่า)", "price", "159.00", 28.0, 6.0, 9, "semibold", "#475569", 4, "2"),
        ("product.barcode", "บาร์โค้ด", "barcode", "8850123456789", 40.0, 14.0, 9, "normal", "#111111", 5, "2"),
        ("product.code", "รหัสสินค้า (SKU)", "text", "88501234", 30.0, 5.0, 8, "medium", "#334155", 6, "2"),
        ("tag.selling_unit", "หน่วยขาย", "text", "ถุง / BAG", 24.0, 5.0, 8, "semibold", "#111111", 7, "2"),
        ("tag.pri_per_unit", "ราคาต่อหน่วย", "text", "51.60 / 100ก.", 32.0, 5.0, 8, "normal", "#334155", 8, "2"),
        ("promotion.text", "ข้อความโปรโมชัน", "text", "ลดพิเศษ 20%", 40.0, 6.0, 9, "bold", "#DC2626", 9, "2"),
        ("promotion.interval", "ช่วงวันที่โปรฯ", "text", "เริ่ม 01/11/26 ถึง 30/11/26", 46.0, 5.0, 7, "normal", "#334155", 10, "2"),
        ("tag.print_date", "วันที่พิมพ์ป้าย", "date", "18/11/2569", 24.0, 5.0, 7, "normal", "#475569", 11, "2"),
        ("product.dim", "ขนาด/ปริมาณ", "text", "250 กรัม", 28.0, 5.0, 8, "normal", "#334155", 12, "2"),
        ("product.image", "รูปสินค้า (อัตโนมัติจาก DB)", "image", "", 22.0, 22.0, 9, "normal", "#111111", 13, "2"),
        ("store.name", "ชื่อสาขา / ร้านค้า", "text", "AdaSoft", 42.0, 5.0, 8, "semibold", "#111111", 14, "1"),
        ("tag.sub_dept", "รหัสหมวด (SubDept)", "text", "D01", 20.0, 5.0, 8, "semibold", "#334155", 15, "1"),
        ("tag.pri_status", "สถานะราคา", "text", "AC", 16.0, 5.0, 8, "semibold", "#334155", 16, "1"),
    ];
    for (code, name, ftype, sample, dw, dh, df, dwt, dc, seq, adv) in label_fields {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMLabelField (FTLbfCode, FTLbfName, FTLbfType, FTLbfSample, FCLbfDefW, FCLbfDefH, FNLbfDefFont, FTLbfDefWeight, FTLbfDefColor, FNLbfSeq, FTLbfStaAdv, FTLbfStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'SYSTEM', ?, 'SYSTEM')",
        )
        .bind(code).bind(name).bind(ftype).bind(sample).bind(dw).bind(dh).bind(df).bind(dwt).bind(dc).bind(seq).bind(adv)
        .bind(&now).bind(&now)
        .execute(pool)
        .await?;
    }

    // Seed system options defaults (idempotent; never overwrite user values)
    let sys_opts = [
        ("DefaultBranch", "PrintBar", "สาขาเริ่มต้น", "", "รหัสสาขาเริ่มต้นเมื่อเข้าใช้งาน", 1_i64),
        ("DefaultTemplate", "PrintBar", "รูปแบบป้ายเริ่มต้น", "ADA-100x38", "รหัสแบบป้ายเริ่มต้นเมื่อเปิดหน้าจอ", 2),
        ("DefaultPrinter", "PrintBar", "เครื่องพิมพ์เริ่มต้น", "", "ชื่อเครื่องพิมพ์เริ่มต้น", 3),
        ("DefaultDpi", "PrintBar", "ความละเอียด DPI เริ่มต้น", "203", "ความละเอียดเครื่องพิมพ์เริ่มต้น", 4),
        ("UrlTemplate", "PrintBar", "URL Template สำหรับ QR", "", "รูปแบบ URL สินค้าใน QR Code", 5),
        ("DefaultUser", "PrintBar", "รหัสผู้ใช้งานเริ่มต้น", "ADMIN", "ชื่อผู้ใช้งานที่บันทึกลง Transaction Log", 6),
        ("AutoPrintRetentionDays", "PrintBar", "จำนวนวันเก็บไฟล์และคิวพิมพ์", "60", "ลบประวัติคิวและไฟล์ชั่วคราวที่เก่าเกินกำหนดอัตโนมัติ", 14),
    ];
    for (code, app, name, val, desc, seq) in sys_opts {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNSSysOption (FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, 'SYSTEM', ?, 'SYSTEM')",
        )
        .bind(code).bind(app).bind(name).bind(val).bind(desc).bind(seq).bind(&now).bind(&now)
        .execute(pool)
        .await?;
    }

    // Default Auto-Print @ 04:00 schedule (disabled until Store User turns it on in UI)
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO TCNTLabelSchedule (
            FTLscCode, FTLscName, FTLscStaUse, FTLscRunTime, FNLscPreSyncMin, FNLscJitterSec,
            FTLscDaysMask, FTLscTriggerRules, FTLscLbsCode, FNLscQtyMode, FTLscSyncMode,
            FNLscSyncRetry, FNLscRetryWaitSec, FTLscOnSyncFail, FTLscOnPrinterBack,
            FNLscMisfireMin, FTLscStaCoverSheet, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
        ) VALUES (
            'SCH-0400', 'พิมพ์ป้ายอัตโนมัติประจำวัน (04:00 น.)', '2', '04:00', 30, 900,
            '1111111', 'PRICE_CHG,PROMO_START,PROMO_END_REVERT,NEW_PDT', 'ADA-100x38', 1, 'STOREFRONT',
            3, 300, 'ABORT', 'WAIT_CONFIRM', 300, '1', ?, 'SYSTEM', ?, 'SYSTEM'
        )
        "#,
    )
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO TCNMPrinterServer (FTPsvCode, FTPsvName, FTPsvStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES ('LOCAL', 'เครื่องนี้ (Local)', '1', ?, 'SYSTEM', ?, 'SYSTEM')"
    )
    .bind(&now).bind(&now)
    .execute(pool).await?;

    // Run label sizes & templates initial seed only when SeedVersion is not recorded yet
    let has_seed_ver: Option<(String,)> = sqlx::query_as(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'SeedVersion' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let size_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNMLabelSize")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if has_seed_ver.is_some() && size_count.0 > 0 {
        return Ok(());
    }

    let label_sizes: Vec<(&str, &str, &str, f64, f64, i64, i64, Option<&str>, Option<&str>, Option<&str>, &str)> = vec![
        ("SH-100x38", "1", "Price Label ใหญ่ 10x3.8 ซม.", 100.0, 38.0, 2, 7, Some("SH-100x38-NOR"), Some("SH-100x38-PMT"), Some("1"), "1"),
        ("SH-50x38", "1", "Price Label เล็ก 5x3.8 ซม.", 50.0, 38.0, 4, 7, Some("SH-50x38-NOR"), Some("SH-50x38-PMT"), Some("2"), "1"),
        ("SH-50x35", "1", "Price Label KPC 5x3.5", 50.0, 35.0, 4, 7, None, None, Some("8"), "2"),
        ("SH-200x95", "1", "Price Card 1/3 A4", 200.0, 95.0, 1, 3, None, None, Some("18"), "2"),
        ("SH-200x142", "1", "Price Card A5 (2 บรรทัดโปรฯ)", 200.0, 142.0, 1, 2, None, None, Some("15"), "1"),
        ("SH-100x142", "1", "Price Card A6", 100.0, 142.0, 2, 2, None, None, Some("16"), "2"),
        ("SH-48x72", "1", "ป้ายราคา A8", 48.0, 72.0, 4, 4, None, None, None, "2"),
        ("SH-35x50", "1", "ป้ายราคา A9", 35.0, 50.0, 5, 5, None, None, None, "2"),
        ("SH-100x65", "1", "Spec Card A7 (Appliance)", 100.0, 65.0, 2, 4, None, None, Some("27"), "2"),
        ("BC-25x32", "2", "สติ๊กเกอร์บาร์โค้ด 25x32", 25.0, 32.0, 7, 8, Some("BC-25x32-NOR"), None, None, "1"),
        ("BC-25x32-NR", "2", "สติ๊กเกอร์ NOT FOR SALE", 25.0, 32.0, 7, 8, Some("BC-25x32-NR"), None, None, "1"),
        ("BC-32x24", "2", "Ticket Price 32x24", 32.0, 24.0, 6, 12, Some("BC-32x24-NOR"), None, Some("3"), "1"),
        ("BC-32x30", "2", "สติ๊กเกอร์ สคบ. (OCPB)", 32.0, 30.0, 6, 9, None, None, None, "2"),
        ("BC-50x25", "2", "บาร์โค้ดกลาง 50x25", 50.0, 25.0, 4, 11, None, None, None, "2"),
        ("BC-100x50", "2", "บาร์โค้ดลัง ITF-14", 100.0, 50.0, 2, 5, None, None, None, "2"),
        ("BC-QR-30x30", "2", "QR Code สินค้า 30x30", 30.0, 30.0, 6, 9, Some("BC-QR-30x30-NOR"), None, None, "1"),
        ("UFM-NOR-16", "1", "UFM ป้ายราคาปกติ (16 ดวง/A4)", 49.0, 70.0, 4, 4, None, None, None, "1"),
        ("UFM-SALE-8", "1", "UFM ป้ายโปรโมชัน SALE (8 ดวง/A4)", 100.0, 70.0, 2, 4, None, None, None, "1"),
        ("UFM-PRO-A5", "1", "UFM ป้ายโปรโมชันใหญ่ A5", 200.0, 142.0, 1, 2, None, None, None, "1"),
        ("UFM-TIMESALE", "1", "UFM ป้าย Time Sale! 50% OFF (4 ดวง/A4)", 200.0, 70.0, 1, 4, None, None, None, "1"),
        ("ADA-100x38", "1", "ADA ป้ายราคา 10x3.8 ซม. (14 ดวง/A4)", 100.0, 38.0, 2, 7, None, None, None, "1"),
        ("ADA-50x38", "1", "ADA ป้ายราคาเล็ก 5x3.8 ซม. (28 ดวง/A4)", 50.0, 38.0, 4, 7, None, None, None, "1"),
        ("ADA-200x142", "1", "ADA Price Card A5 ลดกระหน่ำ (2 ดวง/A4)", 200.0, 142.0, 1, 2, None, None, None, "1"),
        ("ADA-200x95", "1", "ADA Price Card 1/3 A4 ลดกระหน่ำ (3 ดวง/A4)", 200.0, 95.0, 1, 3, None, None, None, "1"),
        ("ADA-200x95-SP", "1", "ADA Price Card 1/3 A4 สินค้าพิเศษ (3 ดวง/A4)", 200.0, 95.0, 1, 3, None, None, None, "1"),
        ("ADA-100x142", "1", "ADA Price Card A6 สินค้าจัดรายการ (4 ดวง/A4)", 100.0, 142.0, 2, 2, None, None, None, "1"),
    ];

    for (code, grp, name, w, h, col, row, tn, tp, hht, sta) in label_sizes {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMLabelSize (FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsStdTmplNor, FTLbsStdTmplPmt, FTLbsHhtCode, FTLbsStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'SYSTEM', ?, 'SYSTEM')"
        )
        .bind(code).bind(grp).bind(name).bind(w).bind(h).bind(col).bind(row)
        .bind(col * row).bind(col * row).bind(tn).bind(tp).bind(tn).bind(tp).bind(hht).bind(sta)
        .bind(&now).bind(&now)
        .execute(pool).await?;
    }

    for t in get_templates() {
        let pri_type = if t.code.ends_with("PMT") { "2" } else { "1" };
        let def_json = serde_json::to_string(&t).unwrap_or_default();
        let lbs_code = if t.code.starts_with("BC-QR-30x30") {
            "BC-QR-30x30"
        } else if t.code.starts_with("BC-25x32-NR") {
            "BC-25x32-NR"
        } else {
            t.code.rsplit_once('-').map(|(a, _)| a).unwrap_or("SH-100x38")
        };
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMLabelTmpl (FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, '1', 'full', ?, 'SYSTEM', ?, 'SYSTEM')"
        )
        .bind(&t.code).bind(lbs_code).bind(pri_type).bind(&t.name).bind(&def_json)
        .bind(&now).bind(&now)
        .execute(pool).await?;
    }

    sqlx::query(
        "INSERT OR IGNORE INTO TCNSSysOption (FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES ('SeedVersion', 'PrintBar', 'เวอร์ชันข้อมูลตั้งต้น', '2', 'ป้องกันการเขียนทับซ้ำเมื่อผู้ใช้แก้ไข/ลบข้อมูล', 99, ?, 'SYSTEM', ?, 'SYSTEM')",
    )
    .bind(&now).bind(&now)
    .execute(pool).await?;

    Ok(())
}

pub async fn ensure_ufm_label_sizes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let has_done: Option<(String,)> = sqlx::query_as(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode IN ('SeedUfmDone', 'SeedVersion') LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    if has_done.is_some() {
        return Ok(());
    }
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let ufm_sizes: Vec<(&str, &str, &str, f64, f64, i64, i64)> = vec![
        ("UFM-NOR-16", "1", "UFM ป้ายราคาปกติ (16 ดวง/A4)", 49.0, 70.0, 4, 4),
        ("UFM-SALE-8", "1", "UFM ป้ายโปรโมชัน SALE (8 ดวง/A4)", 100.0, 70.0, 2, 4),
        ("UFM-PRO-A5", "1", "UFM ป้ายโปรโมชันใหญ่ A5", 200.0, 142.0, 1, 2),
        ("UFM-TIMESALE", "1", "UFM ป้าย Time Sale! 50% OFF (4 ดวง/A4)", 200.0, 70.0, 1, 4),
    ];
    for (code, grp, name, w, h, col, row) in ufm_sizes {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMLabelSize (FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'SYSTEM', ?, 'SYSTEM')"
        )
        .bind(code).bind(grp).bind(name).bind(w).bind(h).bind(col).bind(row)
        .bind(col * row).bind(col * row)
        .bind(&now).bind(&now)
        .execute(pool).await?;
    }
    sqlx::query(
        "INSERT OR IGNORE INTO TCNSSysOption (FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES ('SeedUfmDone', 'PrintBar', 'สถานะสร้างขนาด UFM ตั้งต้น', '1', 'ป้องกันการสร้างซ้ำเมื่อผู้ใช้ลบออก', 97, ?, 'SYSTEM', ?, 'SYSTEM')"
    )
    .bind(&now).bind(&now)
    .execute(pool).await?;
    Ok(())
}

pub async fn ensure_ada_label_sizes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let has_done: Option<(String,)> = sqlx::query_as(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode IN ('SeedAdaDone', 'SeedVersion') LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    if has_done.is_some() {
        return Ok(());
    }
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let ada_sizes: Vec<(&str, &str, &str, f64, f64, i64, i64)> = vec![
        ("ADA-100x38", "1", "ADA ป้ายราคา 10x3.8 ซม. (14 ดวง/A4)", 100.0, 38.0, 2, 7),
        ("ADA-50x38", "1", "ADA ป้ายราคาเล็ก 5x3.8 ซม. (28 ดวง/A4)", 50.0, 38.0, 4, 7),
        ("ADA-200x142", "1", "ADA Price Card A5 ลดกระหน่ำ (2 ดวง/A4)", 200.0, 142.0, 1, 2),
        ("ADA-200x95", "1", "ADA Price Card 1/3 A4 ลดกระหน่ำ (3 ดวง/A4)", 200.0, 95.0, 1, 3),
        ("ADA-200x95-SP", "1", "ADA Price Card 1/3 A4 สินค้าพิเศษ (3 ดวง/A4)", 200.0, 95.0, 1, 3),
        ("ADA-100x142", "1", "ADA Price Card A6 สินค้าจัดรายการ (4 ดวง/A4)", 100.0, 142.0, 2, 2),
    ];
    for (code, grp, name, w, h, col, row) in ada_sizes {
        sqlx::query(
            "INSERT OR IGNORE INTO TCNMLabelSize (FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'SYSTEM', ?, 'SYSTEM')"
        )
        .bind(code).bind(grp).bind(name).bind(w).bind(h).bind(col).bind(row)
        .bind(col * row).bind(col * row)
        .bind(&now).bind(&now)
        .execute(pool).await?;
    }
    sqlx::query(
        "INSERT OR IGNORE INTO TCNSSysOption (FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES ('SeedAdaDone', 'PrintBar', 'สถานะสร้างขนาด ADA ตั้งต้น', '1', 'ป้องกันการสร้างซ้ำเมื่อผู้ใช้ลบออก', 98, ?, 'SYSTEM', ?, 'SYSTEM')"
    )
    .bind(&now).bind(&now)
    .execute(pool).await?;
    Ok(())
}
