//! Branch-local print rules, persistent FIFO and read-only StoreFront master access.
//! SUBMITTED means accepted by the transport, never proof of physical output.
use crate::{
    label_renderer::ResolvedPrinterRoute,
    models::{TCNMLabelSize, TCNTLabelSchedule},
    routes::AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
#[cfg(test)]
use chrono::Datelike;
use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;

type Result<T> = std::result::Result<T, String>;
fn err(e: impl ToString) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error":e.to_string()})),
    )
        .into_response()
}
fn now() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_true() -> bool {
    true
}
fn default_time() -> String {
    "00:00".into()
}
fn default_days() -> String {
    "1111111".into()
}
fn default_protocol() -> String {
    "WINDOWS".into()
}
fn default_port() -> u16 {
    9100
}
fn default_dpi() -> u32 {
    203
}
fn default_qty() -> i64 {
    1
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintProfileTarget {
    pub size: String,
    pub template: String,
    pub printer: String,
    #[serde(default)]
    pub tray: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    #[serde(default = "default_qty")]
    pub quantity: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrintRule {
    pub id: String,
    #[serde(default)]
    pub code: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub branch: String,
    pub source: String,
    #[serde(default = "default_time")]
    pub time: String,
    #[serde(default = "default_days")]
    pub days: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub printer: String,
    #[serde(default)]
    pub tray: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    #[serde(default = "default_qty")]
    pub quantity: i64,
    #[serde(default)]
    pub catch_up: bool,
    #[serde(default)]
    pub offset_days: i32,
    #[serde(default)]
    pub targets: Vec<PrintProfileTarget>,
}

fn effective_targets(r: &PrintRule) -> Vec<PrintProfileTarget> {
    if !r.targets.is_empty() {
        return r.targets.clone();
    }
    vec![PrintProfileTarget {
        size: r.size.clone(),
        template: r.template.clone(),
        printer: r.printer.clone(),
        tray: r.tray.clone(),
        protocol: r.protocol.clone(),
        ip: r.ip.clone(),
        port: r.port,
        dpi: r.dpi,
        quantity: r.quantity,
    }]
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPolicy {
    pub enabled: bool,
    pub interval_minutes: i64,
    pub max_age_minutes: i64,
}
impl Default for SyncPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 15,
            max_age_minutes: 60,
        }
    }
}
#[derive(FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueJob {
    pub id: String,
    pub seq: i64,
    pub source: String,
    pub name: String,
    pub status: String,
    pub created: String,
    pub target_date: Option<String>,
    pub error: Option<String>,
    pub completed: i64,
    pub total: i64,
}

pub async fn ensure_schema(pool: &SqlitePool) -> std::result::Result<(), sqlx::Error> {
    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS ABPrintRule(id TEXT PRIMARY KEY, config TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS ABPrintSetting(id TEXT PRIMARY KEY, value TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS ABPrintJob(seq INTEGER PRIMARY KEY AUTOINCREMENT,id TEXT UNIQUE NOT NULL,
        dedup TEXT UNIQUE,source TEXT NOT NULL,name TEXT NOT NULL,status TEXT NOT NULL,created TEXT NOT NULL,
        config TEXT NOT NULL,error TEXT,completed INTEGER NOT NULL DEFAULT 0,total INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS ABPrintUnit(job TEXT NOT NULL,seq INTEGER NOT NULL,kind TEXT NOT NULL,
        payload TEXT NOT NULL,status TEXT NOT NULL DEFAULT 'READY',error TEXT,PRIMARY KEY(job,seq));
      CREATE TABLE IF NOT EXISTS ABPrintWatermark(rule TEXT NOT NULL,barcode TEXT NOT NULL,hash TEXT NOT NULL,
        PRIMARY KEY(rule,barcode));
      CREATE TABLE IF NOT EXISTS ABPrintAudit(id INTEGER PRIMARY KEY AUTOINCREMENT,job TEXT,action TEXT,created TEXT);
        CREATE TABLE IF NOT EXISTS ABPrintHistory(job TEXT PRIMARY KEY, finished TEXT NOT NULL, destinations TEXT);
        CREATE INDEX IF NOT EXISTS ABPrintHistoryDue ON ABPrintHistory(finished) WHERE destinations IS NULL;
        CREATE TRIGGER IF NOT EXISTS ABPrintHistoryFinished AFTER UPDATE OF status ON ABPrintJob
        WHEN NEW.status IN ('SUBMITTED','FAILED','CANCELLED') AND NEW.status <> OLD.status
          AND EXISTS (SELECT 1 FROM ABPrintUnit WHERE job=NEW.id)
        BEGIN
            INSERT OR REPLACE INTO ABPrintHistory(job,finished) VALUES(NEW.id,datetime('now','localtime'));
        END;").execute(pool).await?;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    // Older releases recorded creation time only. Allow a full retention period after upgrade.
    let seeded = sqlx::query("INSERT OR IGNORE INTO ABPrintSetting VALUES('historyRetentionMigrated','1')")
        .execute(&mut *tx).await?.rows_affected();
    if seeded > 0 {
        sqlx::query("INSERT OR IGNORE INTO ABPrintHistory(job,finished)
            SELECT id,datetime('now','localtime') FROM ABPrintJob
            WHERE status IN ('SUBMITTED','FAILED','CANCELLED') AND EXISTS (SELECT 1 FROM ABPrintUnit WHERE job=id)")
            .execute(&mut *tx).await?;
    }
    let rows: Vec<(String,String)> = sqlx::query_as("SELECT id,trim(coalesce(json_extract(config,'$.code'),'')) FROM ABPrintRule ORDER BY id").fetch_all(&mut *tx).await?;
    let mut used: std::collections::HashSet<_> = rows.iter().map(|(_,code)|code.to_ascii_uppercase()).collect();
    let mut next = 1;
    for (id,code) in rows {
        if !code.is_empty() { continue; }
        loop {
            let candidate = format!("{next:03}");
            next += 1;
            if used.insert(candidate.clone()) {
                sqlx::query("UPDATE ABPrintRule SET config=json_set(config,'$.code',?) WHERE id=?")
                    .bind(candidate).bind(id).execute(&mut *tx).await?;
                break;
            }
        }
    }
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS ABPrintRuleCode ON ABPrintRule(upper(trim(json_extract(config,'$.code')))) WHERE trim(coalesce(json_extract(config,'$.code'),''))<>''")
        .execute(&mut *tx).await?;
    tx.commit().await
}
async fn setting(pool: &SqlitePool, id: &str) -> String {
    sqlx::query_scalar("SELECT value FROM ABPrintSetting WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}
async fn put(pool: &SqlitePool, id: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT INTO ABPrintSetting(id,value) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET value=excluded.value")
      .bind(id).bind(value).execute(pool).await.map_err(|e|e.to_string())?;
    Ok(())
}
async fn rules(pool: &SqlitePool) -> Result<Vec<PrintRule>> {
    let rows: Vec<String> = sqlx::query_scalar("SELECT config FROM ABPrintRule ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    rows.into_iter().map(|raw|serde_json::from_str(&raw).map_err(|e|e.to_string())).collect()
}
async fn rule(pool: &SqlitePool, id: &str) -> Result<PrintRule> {
    rules(pool)
        .await?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or("ไม่พบกฎพิมพ์".into())
}
#[derive(Serialize, FromRow)]
pub struct JobDestination {
    printer: Option<String>,
    label: Option<String>,
    template: Option<String>,
    units: i64,
}

pub async fn job_details(State(st): State<AppState>, axum::extract::Path(id): axum::extract::Path<String>) -> Response {
    let result = async {
        let mut tx = st.pool.begin().await?;
        let archive: Option<String> = sqlx::query_scalar("SELECT destinations FROM ABPrintHistory WHERE job=? AND destinations IS NOT NULL")
            .bind(&id).fetch_optional(&mut *tx).await?;
        if let Some(data) = archive {
            return serde_json::from_str::<Value>(&data).map_err(|e| sqlx::Error::Protocol(e.to_string()));
        }
        let rows = sqlx::query_as::<_, JobDestination>(DESTINATIONS).bind(&id).fetch_all(&mut *tx).await?;
        Ok(json!(rows))
    }.await;
    match result {
        Ok(rows) => Json(rows).into_response(),
        Err(error) => err(error),
    }
}

const DESTINATIONS: &str = r#"
        WITH units AS (
            SELECT u.seq, u.payload, j.config, u.kind,
                   json_extract(u.payload,'$.marks[0].rule') AS scope
            FROM ABPrintUnit u JOIN ABPrintJob j ON j.id=u.job WHERE u.job=?
        )
        SELECT coalesce(nullif(json_extract(payload,'$.route.printer_name'),''),
                        json_extract(payload,'$.printerName'), json_extract(payload,'$.route.lan_ip'), json_extract(payload,'$.ip')) AS printer,
               coalesce(nullif(json_extract(payload,'$.size.FTLbsName'),''),
                        nullif(json_extract(payload,'$.lbsName'),''), json_extract(payload,'$.lbsCode'),
                        json_extract(payload,'$.route.lbs_code')) AS label,
               coalesce(json_extract(payload,'$.template'), CASE WHEN kind='AUTO' THEN
                   CASE WHEN json_valid(scope) AND json_type(config,'$.targets')='array'
                        THEN json_extract(config,'$.targets[' || json_extract(scope,'$[2]') || '].template')
                        ELSE json_extract(config,'$.template') END END) AS template,
               count(*) AS units
        FROM units GROUP BY printer,label,template ORDER BY min(seq)
    "#;

pub(crate) async fn archive_finished_jobs(pool: &SqlitePool) -> std::result::Result<usize, sqlx::Error> {
    let days: Option<String> = sqlx::query_scalar("SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode='AutoPrintRetentionDays'")
        .fetch_optional(pool).await?;
    let days = days.and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(60).max(7);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let jobs: Vec<String> = sqlx::query_scalar("SELECT h.job FROM ABPrintHistory h JOIN ABPrintJob j ON j.id=h.job
        WHERE h.destinations IS NULL AND h.finished < datetime('now','localtime',?)
          AND j.status IN ('SUBMITTED','FAILED','CANCELLED') ORDER BY h.finished LIMIT 10")
        .bind(format!("-{days} days")).fetch_all(&mut *tx).await?;
    for job in &jobs {
        let summary = sqlx::query_as::<_, JobDestination>(DESTINATIONS).bind(job).fetch_all(&mut *tx).await?;
        sqlx::query("UPDATE ABPrintHistory SET destinations=? WHERE job=?")
            .bind(json!(summary).to_string()).bind(job).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM ABPrintUnit WHERE job=?").bind(job).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(jobs.len())
}

pub async fn get_center(State(st): State<AppState>) -> Response {
    let jobs=sqlx::query_as::<_,QueueJob>("SELECT id,seq,source,name,status,created,json_extract(config,'$.targetDate') AS target_date,error,completed,total FROM ABPrintJob WHERE status NOT IN ('SUBMITTED','EMPTY','NO_MATCH','ALREADY_SUBMITTED','FAILED','CANCELLED') OR id IN (SELECT id FROM ABPrintJob WHERE status IN ('SUBMITTED','EMPTY','NO_MATCH','ALREADY_SUBMITTED','FAILED','CANCELLED') ORDER BY seq DESC LIMIT 100) ORDER BY seq DESC").fetch_all(&st.pool).await;
    match (rules(&st.pool).await,jobs) {
      (Ok(r),Ok(j))=>Json(json!({"rules":r,"jobs":j,
        "master":crate::storage::status(&st.pool).await,"heartbeat":setting(&st.pool,"heartbeat").await,
        "separateSettingsVersion":1,"bulkVersion":1,"bulkSettings":serde_json::from_str::<Value>(&setting(&st.pool,&format!("bulk:{}",crate::storage::configured_branch(&st.pool).await)).await).unwrap_or(Value::Null),
        "legacySchedules": sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNTLabelSchedule").fetch_one(&st.pool).await.unwrap_or(0)
      })).into_response(), (Err(e),_)=>err(e),(_,Err(e))=>err(e)
    }
}
async fn validate_rule(pool: &SqlitePool, r: &PrintRule) -> Result<()> {
    if r.id.trim().is_empty() || r.name.trim().is_empty() || r.branch.trim().is_empty() {
        return Err("กรุณาระบุชื่อ Profile และสาขา".into());
    }
    let code = r.code.trim();
    if code.is_empty() || code.len() > 24 || !code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err("กรุณาระบุรหัส Profile เป็นตัวเลขหรือตัวอักษรภาษาอังกฤษ 1–24 ตัว (ห้ามเว้นวรรค)".into());
    }
    if !(-30..=30).contains(&r.offset_days) {
        return Err("ค่าชดเชยวัน (Offset) ต้องอยู่ระหว่าง -30 ถึง +30".into());
    }
    if ![
        "PRICE_CHG",
        "PROMO_START",
        "PROMO_END_REVERT",
        "NEW_PDT",
    ]
    .contains(&r.source.as_str())
    {
        return Err("ประเภทข้อมูลไม่รองรับ".into());
    }
    if !r.time.trim().is_empty() {
        NaiveTime::parse_from_str(&r.time, "%H:%M").map_err(|_| "เวลาไม่ถูกต้อง")?;
    }
    if !r.days.trim().is_empty()
        && (r.days.len() != 7 || !r.days.bytes().all(|b| b == b'0' || b == b'1') || !r.days.contains('1'))
    {
        return Err("เลือกวันอย่างน้อยหนึ่งวัน".into());
    }
    let branch = crate::storage::configured_branch(pool).await;
    if branch.is_empty() || branch != r.branch {
        return Err("สาขาของ Profile ต้องตรงกับแหล่งข้อมูลที่ตั้งไว้".into());
    }
    let targets = effective_targets(r);
    if targets.is_empty() {
        return Err("กรุณาเพิ่มแบบป้ายและเครื่องพิมพ์อย่างน้อย 1 รายการ".into());
    }
    for (idx, t) in targets.iter().enumerate() {
        let prefix = if targets.len() > 1 {
            format!("รายการกระดาษที่ {}: ", idx + 1)
        } else {
            String::new()
        };
        if t.printer.trim().is_empty() || !(1..=999).contains(&t.quantity) {
            return Err(format!("{prefix}ระบุเครื่องพิมพ์และจำนวน 1–999"));
        }
        if !t.ip.is_empty() {
            t.ip.parse::<std::net::IpAddr>()
                .map_err(|_| format!("{prefix}IP เครื่องพิมพ์ไม่ถูกต้อง"))?;
            if t.port == 0 {
                return Err(format!("{prefix}Port ไม่ถูกต้อง"));
            }
        }
        load_layout_for(pool, &t.size, &t.template, t.ip.is_empty())
            .await
            .map_err(|e| format!("{prefix}{e}"))?;
    }
    Ok(())
}
async fn load_layout(
    pool: &SqlitePool,
    r: &PrintRule,
) -> Result<(TCNMLabelSize, Vec<crate::templates::TemplateElement>)> {
    load_layout_for(pool, &r.size, &r.template, r.ip.is_empty()).await
}
// Capture the full designer document, including colours, fonts, layer order and conditional fields.
async fn windows_design(pool:&SqlitePool, size:&str, template:&str, windows:bool)->Result<Value> {
    if !windows || template.starts_with("legacy:") {return Ok(Value::Null);}
    let raw:Option<String>=sqlx::query_scalar("SELECT FTLtmLayout FROM TCNMLabelTmpl WHERE FTLbsCode=? AND FTLtmCode=? AND FTLtmStaUse='1'")
        .bind(size).bind(template).fetch_optional(pool).await.map_err(|e|e.to_string())?.flatten();
    let doc: Value=serde_json::from_str(raw.as_deref().unwrap_or("null")).map_err(|e|e.to_string())?;
    Ok(if doc["customDesign"]==true {doc} else {Value::Null})
}
async fn load_layout_for(
    pool: &SqlitePool,
    size_code: &str,
    template_code: &str,
    windows: bool,
) -> Result<(TCNMLabelSize, Vec<crate::templates::TemplateElement>)> {
    let size = sqlx::query_as::<_, TCNMLabelSize>(
        "SELECT * FROM TCNMLabelSize WHERE FTLbsCode=? AND FTLbsStaUse='1'",
    )
    .bind(size_code)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or("ขนาดป้ายไม่พบหรือปิดใช้งาน")?;
    if template_code.strip_prefix("legacy:") == Some(size_code) {
        return Ok((size, Vec::new()));
    }
    let raw:Option<String>=sqlx::query_scalar("SELECT FTLtmLayout FROM TCNMLabelTmpl WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmStaUse='1'")
      .bind(template_code).bind(size_code).fetch_optional(pool).await.map_err(|e|e.to_string())?.flatten();
    let raw = raw.ok_or("ไม่พบ Template ที่เลือกสำหรับขนาดนี้ กรุณาบันทึกแบบป้ายก่อน")?;
    let layout: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if !windows && layout["designMode"] == "preprint" {
        return Err("แบบกระดาษพิมพ์พื้นหลังสำเร็จให้ใช้ Manual ก่อน Auto ยังไม่รองรับการชดเชยแบบนี้".into());
    }
    let visible: Vec<Value> = layout["elements"].as_array().ok_or("Template ไม่ถูกต้อง")?.iter()
        .filter(|el|el["visible"].as_bool()!=Some(false)).cloned().collect();
    let mut elements: Vec<crate::templates::TemplateElement> =
        serde_json::from_value(json!(visible)).map_err(|e| e.to_string())?;
    if elements.is_empty() {
        return Err("Template ไม่มีองค์ประกอบ".into());
    }
    // Refuse unsupported headless elements instead of silently losing parts of a label.
    for el in &visible {
        if windows && layout["customDesign"] == true {break;}
        let kind = el["type"].as_str().unwrap_or("");
        let rich = windows && (matches!(kind,"image"|"qrcode") || (kind=="icon" && el["iconName"]=="heart"));
        if (!rich && !["text", "static", "price", "date", "barcode", "rect", "line"].contains(&kind))
            || el["rotation"].as_f64().unwrap_or(0.0) != 0.0
            || el["opacity"].as_f64().unwrap_or(1.0) != 1.0
        {
            return Err("แบบนี้มีองค์ประกอบที่ Auto headless ยังรองรับไม่ครบ ให้ใช้ Manual หรือเลือกแบบ text/price/barcode/rect/line ที่ไม่หมุน".into());
        }
    }
    if layout["designMode"] == "preprint" {
        elements[0].preprinted_paper = Some(true);
    }
    Ok((size, elements))
}
#[derive(Deserialize)]
pub struct BulkRules {
    branch: String,
    rules: Vec<PrintRule>,
    preferences: Value,
}
async fn persist_bulk(pool: &SqlitePool, mut body: BulkRules) -> Result<()> {
    let branch=crate::storage::configured_branch(pool).await;
    if branch.is_empty() || body.branch!=branch || body.preferences["branch"].as_str()!=Some(branch.as_str()) {
        return Err("สาขาของการตั้งค่าต้องตรงกับแหล่งข้อมูล StoreFront".into());
    }
    let old=rules(pool).await?;
    let mut ids=std::collections::HashSet::new();
    let mut codes=std::collections::HashSet::new();
    for r in &mut body.rules {
        if r.code.trim().is_empty() {
            if let Some(previous) = old.iter().find(|x|x.id==r.id) { r.code.clone_from(&previous.code); }
        }
        r.code = r.code.trim().to_owned();
        if !ids.insert(r.id.clone()) {return Err("พบรหัสภายในซ้ำในชุดบันทึก".into());}
        if !r.code.trim().is_empty() && !codes.insert(r.code.trim().to_ascii_uppercase()) {
            return Err(format!("พบรหัส Profile \"{}\" ซ้ำกัน", r.code.trim()));
        }
        if r.branch!=branch {return Err("พบ Profile ของสาขาอื่นในชุดบันทึก".into());}
        if let Some(previous)=old.iter().find(|x|x.id==r.id) {
            if previous.branch!=branch {return Err("ไม่สามารถแก้ไข Profile ของสาขาอื่น".into());}
            // Allow stopping an existing rule even if its template/printer was removed or legacy code was empty.
            let mut stopped=previous.clone();stopped.enabled=false;
            if !r.enabled && serde_json::to_value(&stopped).unwrap()==serde_json::to_value(&*r).unwrap() {continue;}
        }
        validate_rule(pool,r).await.map_err(|e|format!("{}: {}",r.name,e))?;
    }
    let mut tx=pool.begin().await.map_err(|e|e.to_string())?;
    for r in &body.rules {
        sqlx::query("INSERT INTO ABPrintRule(id,config) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET config=excluded.config")
            .bind(&r.id).bind(serde_json::to_string(r).map_err(|e|e.to_string())?).execute(&mut *tx).await.map_err(|e|e.to_string())?;
        sqlx::query("INSERT OR IGNORE INTO ABPrintSetting(id,value) VALUES(?,?)")
            .bind(format!("ruleSince:{}",r.id)).bind(Local::now().format("%Y-%m-%d").to_string()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    }
    sqlx::query("INSERT INTO ABPrintSetting(id,value) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET value=excluded.value")
        .bind(format!("bulk:{}",branch)).bind(body.preferences.to_string()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    tx.commit().await.map_err(|e|e.to_string())?;
    Ok(())
}
pub async fn save_rules(State(st): State<AppState>, Json(body): Json<BulkRules>) -> Response {
    match persist_bulk(&st.pool,body).await {Ok(())=>Json(json!({"ok":true})).into_response(),Err(e)=>err(e)}
}

pub async fn save_rule(State(st): State<AppState>, Json(mut r): Json<PrintRule>) -> Response {
    r.code = r.code.trim().to_string();
    let targets = effective_targets(&r);
    if let Some(first) = targets.first() {
        r.size = first.size.clone();
        r.template = first.template.clone();
        r.printer = first.printer.clone();
        r.tray = first.tray.clone();
        r.protocol = first.protocol.clone();
        r.ip = first.ip.clone();
        r.port = first.port;
        r.dpi = first.dpi;
        r.quantity = first.quantity;
    }
    r.targets = targets;
    if let Err(e) = validate_rule(&st.pool, &r).await {
        return err(e);
    }
    if let Ok(existing) = rules(&st.pool).await {
        if existing.iter().any(|x| x.id != r.id && x.code.eq_ignore_ascii_case(&r.code)) {
            return err(format!("รหัส Profile \"{}\" ถูกใช้งานแล้ว กรุณาใช้รหัสอื่น", r.code));
        }
    }
    match sqlx::query("INSERT INTO ABPrintRule(id,config) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET config=excluded.config")
      .bind(&r.id).bind(serde_json::to_string(&r).unwrap()).execute(&st.pool).await {
        Ok(_)=>{
          let key=format!("ruleSince:{}",r.id);
          if setting(&st.pool,&key).await.is_empty(){let _=put(&st.pool,&key,&Local::now().format("%Y-%m-%d").to_string()).await;}
          Json(json!({"ok":true,"rule":r})).into_response()
        },Err(e)=>err(e)
      }
}

pub async fn delete_rule(State(st): State<AppState>, Json(body): Json<Value>) -> Response {
    let id = body["id"].as_str().unwrap_or("").trim();
    if id.is_empty() {
        return err("ไม่พบรหัส Profile ที่ต้องการลบ");
    }
    match sqlx::query("DELETE FROM ABPrintRule WHERE id=?")
        .bind(id)
        .execute(&st.pool)
        .await
    {
        Ok(res) if res.rows_affected() > 0 => Json(json!({"ok":true})).into_response(),
        Ok(_) => err("ไม่พบ Profile ที่ต้องการลบ"),
        Err(e) => err(e),
    }
}
fn schedule(r: &PrintRule) -> TCNTLabelSchedule {
    TCNTLabelSchedule {
        FTLscCode: r.id.clone(),
        FTLscName: r.name.clone(),
        FTLscStaUse: "1".into(),
        FTLscRunTime: r.time.clone(),
        FNLscPreSyncMin: 0,
        FNLscJitterSec: 0,
        FTLscDaysMask: r.days.clone(),
        FTLscTriggerRules: r.source.clone(),
        FTLscBchCode: Some(r.branch.clone()),
        FTLscPplCode: None,
        FTLscLbsCode: r.size.clone(),
        FNLscQtyMode: 1,
        FTLscSyncMode: "LOCAL".into(),
        FNLscSyncRetry: 1,
        FNLscRetryWaitSec: 0,
        FTLscOnSyncFail: "ABORT".into(),
        FTLscOnPrinterBack: "WAIT_CONFIRM".into(),
        FNLscMisfireMin: 1440,
        FTLscStaCoverSheet: "2".into(),
        FDLscLastRunAt: None,
        FDCreateOn: None,
        FTCreateBy: None,
        FDLastUpdOn: None,
        FTLastUpdBy: None,
    }
}
async fn candidates(
    pool: &SqlitePool,
    r: &PrintRule,
) -> Result<Vec<crate::delta_engine::DeltaCandidate>> {
    let first = effective_targets(r)
        .into_iter()
        .next()
        .ok_or_else(|| "ไม่มีรายการแบบป้ายใน Profile".to_string())?;
    let date = crate::delta_engine::target_date_with_offset(Local::now().date_naive(), r.offset_days);
    candidates_for_target(pool, r, &first, &date).await
}

async fn candidates_for_target(
    pool: &SqlitePool,
    r: &PrintRule,
    target: &PrintProfileTarget,
    target_date: &str,
) -> Result<Vec<crate::delta_engine::DeltaCandidate>> {
    let mut sched = schedule(r);
    sched.FTLscLbsCode = target.size.clone();
    let mut items =
        crate::delta_engine::compute_profile_candidates_exact(pool, &sched, target_date)
            .await
            .map_err(|e| e.to_string())?
            .items;
    let (_, els) = load_layout_for(pool, &target.size, &target.template, target.ip.is_empty()).await?;
    let (price_date, verify_promo) = match r.source.as_str() {
        "PRICE_CHG" => (target_date.to_owned(), false),
        "PROMO_END_REVERT" => (crate::delta_engine::next_day(target_date), true),
        _ => (target_date.to_owned(), true),
    };
    for it in &mut items {
        it.lbs_code = target.size.clone();
        it.qty = target.quantity;
        // Share the existing manual data/price builder; only offset targeting and Base Price/Price Off differ.
        let request = crate::routes::BuildLineBody {
            barcode: it.bar_code.clone(),
            pdt_code: Some(it.pdt_code.clone()),
            qty: Some(target.quantity),
            bch_code: r.branch.clone(),
            pri_type: "1".into(),
            verify: Some(verify_promo),
            override_lbs: Some(target.size.clone()),
            effective_date: Some(price_date.clone()),
            price_group: None,
        };
        let response =
            crate::routes::post_build_line(State(AppState { pool: pool.clone() }), Json(request))
                .await;
        let success = response.status().is_success();
        let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .map_err(|e| e.to_string())?;
        let built: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if !success {
            return Err(format!("{}: {}", it.bar_code, built["error"]));
        }
        let mut ctx = built["ctx"].clone();
        ctx["product"]["bar_bits"] = json!(crate::barcode::encode_barcode_modules(&it.bar_code, "EAN13"));
        for key in ["value", "old_value"] {
            if let Some(price) = ctx["price"][key].as_f64() {
                ctx["price"][key] = json!(format!("{price:.2}"));
            }
        }
        it.sale_price = built["setPrice"].as_f64().ok_or("ไม่พบราคาปัจจุบัน")?;
        it.pri_type = built["priType"].as_str().unwrap_or("1").into();
        it.pdt_name = built["pdtName"]
            .as_str()
            .unwrap_or(&it.pdt_name)
            .to_string();
        let mut hash_ctx = ctx.clone();
        if let Some(tag) = hash_ctx.get_mut("tag").and_then(Value::as_object_mut) {
            tag.remove("print_date");
        }
        use sha2::{Digest, Sha256};
        it.price_hash = format!(
            "{:x}",
            Sha256::digest(
                format!(
                    "{}|{}|{}|{}|{}",
                    r.id,
                    r.branch,
                    hash_ctx,
                    target.quantity,
                    serde_json::to_string(&els).unwrap()
                )
                .as_bytes()
            )
        );
        if target.template.starts_with("legacy:") { ctx["legacyPriType"] = json!(it.pri_type); }
        it.context = Some(ctx);
        it.skipped_dup = false;
    }
    items.retain(|it| {
        !(r.source == "PROMO_START" && it.pri_type != "2")
            && !(r.source == "PROMO_END_REVERT" && it.pri_type == "2")
    });
    Ok(items)
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ManualCandidates { pub source:String, pub date:String, pub size:String, pub quantity:i64 }
// Read only: uses exact Effective Date matching without watermark skipping.
pub async fn manual_candidates(State(st):State<AppState>,Json(body):Json<ManualCandidates>)->Response {
    let result=async {
        if !["PRICE_CHG","PROMO_START","PROMO_END_REVERT","NEW_PDT"].contains(&body.source.as_str()) {return Err("เหตุการณ์ไม่ถูกต้อง".to_string());}
        chrono::NaiveDate::parse_from_str(&body.date,"%Y-%m-%d").map_err(|_|"วันที่ไม่ถูกต้อง".to_string())?;
        if !(1..=999).contains(&body.quantity){return Err("จำนวนต้องอยู่ระหว่าง 1–999".into());}
        let branch=crate::storage::configured_branch(&st.pool).await;
        if branch.is_empty(){return Err("กรุณาตั้งค่าแหล่งข้อมูล StoreFront".into());}
        let r=PrintRule{id:"manual-preview".into(),code:"MANUAL".into(),name:"ค้นหาสินค้า".into(),enabled:false,branch:branch.clone(),source:body.source.clone(),time:"00:00".into(),days:"1111111".into(),size:body.size.clone(),template:String::new(),printer:String::new(),tray:String::new(),protocol:"WINDOWS".into(),ip:String::new(),port:9100,dpi:203,quantity:body.quantity,catch_up:false,offset_days:0,targets:vec![]};
        let delta=crate::delta_engine::compute_profile_candidates_exact(&st.pool,&schedule(&r),&body.date).await.map_err(|e|e.to_string())?;
        let (price_date, verify_promo) = match body.source.as_str() {
            "PRICE_CHG" => (body.date.clone(), false),
            "PROMO_END_REVERT" => (crate::delta_engine::next_day(&body.date), true),
            _ => (body.date.clone(), true),
        };
        let mut lines=vec![];
        for item in delta.items {
            let response=crate::routes::post_build_line(State(st.clone()),Json(crate::routes::BuildLineBody{barcode:item.bar_code,pdt_code:Some(item.pdt_code),qty:Some(body.quantity),bch_code:branch.clone(),pri_type:"1".into(),verify:Some(verify_promo),override_lbs:Some(body.size.clone()),effective_date:Some(price_date.clone()),price_group:None})).await;
            let ok=response.status().is_success();
            let bytes=axum::body::to_bytes(response.into_body(),4*1024*1024).await.map_err(|e|e.to_string())?;
            let line:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
            if !ok{return Err(line["error"].as_str().unwrap_or("สร้างข้อมูลป้ายไม่สำเร็จ").to_string());}
            let promo=line["isPromo"]==true;
            if (body.source=="PROMO_START"&&!promo)||(body.source=="PROMO_END_REVERT"&&promo){continue;}
            lines.push(line);
        }
        Ok::<_,String>(lines)
    }.await;
    match result {Ok(lines)=>Json(json!({"lines":lines})).into_response(),Err(e)=>err(e)}
}

pub async fn preview_rule(State(st): State<AppState>, Json(r): Json<PrintRule>) -> Response {
    let date = crate::delta_engine::target_date_with_offset(Local::now().date_naive(), r.offset_days);
    if let Err(e) = validate_rule(&st.pool, &r).await {
        return err(e);
    }
    let targets = effective_targets(&r);
    let mut groups = Vec::with_capacity(targets.len());
    let mut first_items = Vec::new();
    for (idx, t) in targets.iter().enumerate() {
        match candidates_for_target(&st.pool, &r, t, &date).await {
            Ok(items) => {
                if idx == 0 {
                    first_items = items.clone();
                }
                let (size, _) = match load_layout_for(&st.pool, &t.size, &t.template, t.ip.is_empty()).await {
                    Ok(layout) => layout, Err(e) => return err(e),
                };
                let (area, warning) = match crate::printable_area::read(&t.printer, &t.tray, &t.ip, &size).await {
                    Ok(area) => {
                        let count = items.len().saturating_mul(t.quantity as usize);
                        let warning = area.as_ref().and_then(|a| a.warning(&size, t.template.starts_with("legacy:"), count));
                        (area, warning)
                    }
                    Err(e) => (None, Some(e)),
                };
                groups.push(json!({
                    "printableArea": area, "paperWarning": warning,
                    "size": t.size,
                    "template": t.template,
                    "printer": t.printer,
                    "tray": t.tray,
                    "quantity": t.quantity,
                    "items": items
                }));
            }
            Err(e) => return err(e),
        }
    }
    Json(json!({
        "items": first_items,
        "targetDate": date,
        "groups": groups,
        "asOf": now(),
        "master": crate::storage::status(&st.pool).await
    }))
    .into_response()
}
async fn insert_job(
    pool: &SqlitePool,
    source: &str,
    name: &str,
    config: Value,
    dedup: Option<&str>,
    status: &str,
    units: Vec<(String, Value)>,
) -> Result<String> {
    let mut tx = pool.begin().await.map_err(|e|e.to_string())?;
    let id = insert_job_rows(&mut tx,source,name,config,dedup,status,units).await?;
    tx.commit().await.map_err(|e|e.to_string())?;
    Ok(id)
}
async fn insert_job_rows(conn: &mut sqlx::SqliteConnection, source: &str, name: &str,
    config: Value, dedup: Option<&str>, status: &str, units: Vec<(String,Value)>) -> Result<String> {
    let id = uid();
    // An overlapping invocation of the same Profile must not create a second
    // active job. Terminal jobs do not block a deliberate later reprint.
    let inserted=sqlx::query("INSERT OR IGNORE INTO ABPrintJob(id,dedup,source,name,status,created,config,total) SELECT ?,?,?,?,?,?,?,? WHERE ? != 'PROFILE' OR NOT EXISTS (SELECT 1 FROM ABPrintJob WHERE source='PROFILE' AND upper(json_extract(config,'$.code'))=upper(?) AND status NOT IN ('SUBMITTED','EMPTY','NO_MATCH','ALREADY_SUBMITTED','FAILED','CANCELLED'))")
      .bind(&id).bind(dedup).bind(source).bind(name).bind(status).bind(now()).bind(config.to_string()).bind(units.len() as i64)
      .bind(source).bind(config["code"].as_str().unwrap_or(""))
      .execute(&mut *conn).await.map_err(|e|e.to_string())?;
    if inserted.rows_affected() == 0 {
        return Err("งานนี้มีอยู่ในคิวแล้ว กรุณาตรวจสถานะงานเดิมก่อนเรียกซ้ำ".into());
    }
    for (n, (kind, payload)) in units.into_iter().enumerate() {
        sqlx::query("INSERT INTO ABPrintUnit(job,seq,kind,payload) VALUES(?,?,?,?)")
            .bind(&id)
            .bind(n as i64)
            .bind(kind)
            .bind(payload.to_string())
            .execute(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(id)
}
pub async fn enqueue_rule(State(st): State<AppState>, Json(body): Json<Value>) -> Response {
    let result = async {
        let r = rule(&st.pool, body["id"].as_str().unwrap_or("")).await?;
        validate_rule(&st.pool, &r).await?;
        insert_job(
            &st.pool,
            "AUTO",
            &r.name,
            serde_json::to_value(&r).unwrap(),
            None,
            "WAIT_DATA",
            vec![],
        )
        .await
    }
    .await;
    match result {
        Ok(id) => Json(json!({"ok":true,"id":id,"status":"WAIT_DATA"})).into_response(),
        Err(e) => err(e),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetCheck {
    size: TCNMLabelSize,
    count: usize,
}

async fn manual_sheet_check(printer: &str, tray: &str, check: &SheetCheck) -> Result<Value> {
    let area = crate::printable_area::read(printer, tray, "", &check.size).await?;
    let warning = area.as_ref().and_then(|a| a.warning(&check.size, false, check.count));
    Ok(json!({"area": area, "warning": warning}))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetPreviewRequest {
    printer_name: String,
    tray_name: String,
    sheet_check: SheetCheck,
}

pub async fn preview_manual_sheet(Json(req): Json<SheetPreviewRequest>) -> Response {
    match manual_sheet_check(&req.printer_name, &req.tray_name, &req.sheet_check).await {
        Ok(result) => Json(result).into_response(), Err(e) => err(e),
    }
}

pub async fn enqueue_manual(State(st): State<AppState>, Json(body): Json<Value>) -> Response {
    let result=async {
      let requests=body["units"].as_array().ok_or("ไม่มีข้อมูลพิมพ์")?;
      let mut units=vec![];
      for req in requests {
        let payload=req["payload"].clone();
        match req["kind"].as_str().unwrap_or("") {
          "WINDOWS"=>{
            let _:crate::routes::WindowsDirectPrintBody=serde_json::from_value(payload.clone()).map_err(|e|e.to_string())?;
            if payload["printerName"].as_str().unwrap_or("").trim().is_empty(){return Err("ยังไม่ได้เลือกเครื่องพิมพ์ Windows".into())}
            if payload["testMode"]==true { units.push(("WINDOWS".into(),payload)); continue; }
            if let Some(snapshot) = payload.get("sheetCheck").filter(|v| !v.is_null()) {
                let check: SheetCheck = serde_json::from_value(snapshot.clone()).map_err(|e| e.to_string())?;
                let printer = payload["printerName"].as_str().unwrap_or_default();
                let result = manual_sheet_check(printer, payload["trayName"].as_str().unwrap_or_default(), &check).await?;
                if let Some(message) = result["warning"].as_str() { return Err(format!("{printer}: {message}")); }
            }
            for page in payload["pageImages"].as_array().ok_or("ไม่มีภาพหน้าพิมพ์")? {
              let mut p=payload.clone();p["pageImages"]=json!([page]); units.push(("WINDOWS".into(),p));
            }
          },
          "LAN"=>{
            let mut requests:Vec<(Value,u64)>=vec![];
            if payload["testMode"]==true {requests.push((payload.clone(),1));} else {
              for line in payload["lines"].as_array().ok_or("ไม่มีรายการ LAN")?.iter().filter(|l|l["selected"]==true) {
                let qty=line["qty"].as_u64().unwrap_or(1);
                if qty==0 || qty>10000 {return Err("จำนวนพิมพ์ต่อรายการต้องอยู่ระหว่าง 1–10000".into())}
                let mut one=line.clone();one["qty"]=json!(1);
                let mut request=payload.clone();request["lines"]=json!([one]);requests.push((request,qty));
              }
            }
            for (mut request,qty) in requests {
              let label_code = request["lines"][0]["lbsCode"].clone();
              let template = request["lines"][0]["tmplCode"].clone();
              request["previewOnly"]=json!(true);
              let response=crate::routes::post_print_send(State(st.clone()),Json(request)).await;
              let ok=response.status().is_success();
              let bytes=axum::body::to_bytes(response.into_body(),32*1024*1024).await.map_err(|e|e.to_string())?;
              let v:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
              if !ok || v["success"]!=true {return Err(v.to_string())}
              for _ in 0..qty {units.push(("RAW".into(),json!({"ip":payload["printerIp"],"port":payload["printerPort"],"command":v["command"],"lbsCode":label_code,"template":template})));}
            }
          }, _=>return Err("ชนิดงานพิมพ์ไม่รองรับ".into())
        }
      }
      if units.is_empty(){return Err("ไม่มีหน้าพิมพ์".into())}
      insert_job(&st.pool,"MANUAL",body["name"].as_str().unwrap_or("พิมพ์ด้วยตนเอง"),body["jobId"].clone(),body["requestId"].as_str(),"QUEUED",units).await
    }.await;
    match result {Ok(id)=>Json(json!({"ok":true,"success":true,"id":id,"status":"QUEUED","message":"บันทึกเข้าคิวแล้ว ดูผลในเมนูประวัติการพิมพ์"})).into_response(),Err(e)=>err(e)}
}
pub async fn legacy_disabled() -> Response {
    err("ตารางรุ่นเดิมหยุดใช้งานแล้ว กรุณาใช้ตั้งค่า > ตั้งเวลาพิมพ์")
}
pub async fn enqueue_windows(State(st): State<AppState>, Json(payload): Json<Value>) -> Response {
    enqueue_manual(
        State(st),
        Json(json!({"name":"Windows / ทดสอบพิมพ์","units":[{"kind":"WINDOWS","payload":payload}]})),
    )
    .await
}
pub async fn enqueue_lan(State(st): State<AppState>, Json(payload): Json<Value>) -> Response {
    if payload["previewOnly"] == true {
        return crate::routes::post_print_send(State(st), Json(payload)).await;
    }
    enqueue_manual(
        State(st),
        Json(json!({"name":"LAN / ทดสอบพิมพ์","units":[{"kind":"LAN","payload":payload}]})),
    )
    .await
}
pub async fn queue_action(State(st): State<AppState>, Json(b): Json<Value>) -> Response {
    let result:Result<()>=async {
      let id=b["id"].as_str().ok_or("ไม่พบงาน")?;let action=b["action"].as_str().unwrap_or("");
      let mut tx=st.pool.begin().await.map_err(|e|e.to_string())?;
      let status:String=sqlx::query_scalar("SELECT status FROM ABPrintJob WHERE id=?").bind(id).fetch_one(&mut *tx).await.map_err(|e|e.to_string())?;
      if ["PREPARING","SENDING"].contains(&status.as_str()){return Err("งานกำลังทำงาน รอจบหน้าปัจจุบันก่อน".into())}
      match action {
        "CANCEL"=>{sqlx::query("UPDATE ABPrintJob SET status='CANCELLED' WHERE id=? AND status NOT IN ('SUBMITTED','EMPTY','NO_MATCH','ALREADY_SUBMITTED','FAILED')").bind(id).execute(&mut *tx).await.map_err(|e|e.to_string())?;},
        "RETRY"|"CONFIRM_SENT"=>{
          if !["BLOCKED","NEEDS_REVIEW"].contains(&status.as_str()){return Err("งานนี้ไม่อยู่ในสถานะรอตรวจสอบ".into())}
          if action=="CONFIRM_SENT" && status!="NEEDS_REVIEW"{return Err("ยืนยันได้เฉพาะหน้าที่ไม่ทราบผล".into())}
          if action=="CONFIRM_SENT" {
            let payloads:Vec<String>=sqlx::query_scalar("SELECT payload FROM ABPrintUnit WHERE job=? AND status='UNKNOWN'").bind(id).fetch_all(&mut *tx).await.map_err(|e|e.to_string())?;
            for payload in payloads {let v:Value=serde_json::from_str(&payload).map_err(|e|e.to_string())?;if let Some(marks)=v["marks"].as_array(){for m in marks {
              sqlx::query("INSERT INTO ABPrintWatermark(rule,barcode,hash) VALUES(?,?,?) ON CONFLICT(rule,barcode) DO UPDATE SET hash=excluded.hash")
                .bind(m["rule"].as_str()).bind(m["barcode"].as_str()).bind(m["hash"].as_str()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
            }}}
          }
          sqlx::query("UPDATE ABPrintUnit SET status=?,error=NULL WHERE job=? AND status IN ('UNKNOWN','SENDING')")
            .bind(if action=="RETRY"{"READY"}else{"SUBMITTED"}).bind(id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
          let count:i64=sqlx::query_scalar("SELECT COUNT(*) FROM ABPrintUnit WHERE job=?").bind(id).fetch_one(&mut *tx).await.map_err(|e|e.to_string())?;
          sqlx::query("UPDATE ABPrintJob SET status=?,error=NULL WHERE id=?").bind(if count==0{"WAIT_DATA"}else{"QUEUED"}).bind(id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
        }, _=>return Err("คำสั่งไม่ถูกต้อง".into())
      }
      sqlx::query("INSERT INTO ABPrintAudit(job,action,created) VALUES(?,?,?)").bind(id).bind(action).bind(now()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
      tx.commit().await.map_err(|e|e.to_string())?;Ok(())
    }.await;
    match result {
        Ok(_) => Json(json!({"ok":true})).into_response(),
        Err(e) => err(e),
    }
}
pub async fn save_sync(State(_st): State<AppState>, Json(_p): Json<SyncPolicy>) -> Response {
    err("การดาวน์โหลดข้อมูลจัดการโดย StoreFront")
}
pub async fn request_sync(State(_st): State<AppState>) -> Response {
    err("AdaBarprint อ่านฐานข้อมูล StoreFront เท่านั้น กรุณาอัปเดตข้อมูลจาก StoreFront")
}

async fn fresh(pool: &SqlitePool) -> bool {
    crate::storage::auto_ready(pool).await
}

async fn prepare_auto(pool: &SqlitePool, id: &str, config: &str) -> Result<()> {
    let r: PrintRule = serde_json::from_str(config).map_err(|e| e.to_string())?;
    validate_rule(pool, &r).await?;
    if r.source.starts_with("PROMO_") && !crate::storage::promo_ready(pool).await { return Err("รอ StoreFront อัปเดตข้อมูลโปรโมชั่นให้พร้อม".into()); }
    let (lbs, elements) = load_layout(pool, &r).await?;
    let design=windows_design(pool,&r.size,&r.template,r.ip.is_empty()).await?;
    let sync_revision=crate::storage::master_revision(pool).await;
    let items = candidates(pool, &r).await?;
    if crate::storage::master_revision(pool).await!=sync_revision {return Err("ข้อมูล Sync เปลี่ยนระหว่างเตรียมงาน กรุณาทำต่อเพื่อสร้าง snapshot ใหม่".into());}
    let route = ResolvedPrinterRoute {
        lbs_code: r.size.clone(),
        pri_type: "1".into(),
        protocol: r.protocol.clone(),
        printer_name: r.printer.clone(),
        tray_name: r.tray.clone(),
        lan_ip: if r.ip.is_empty() {
            None
        } else {
            Some(r.ip.clone())
        },
        lan_port: r.port,
        dpi: r.dpi,
    };
    let date = Local::now().format("%Y-%m-%d").to_string();
    let mut pages: Vec<(Value, Value)> = vec![];
    for it in items.into_iter().filter(|i| !i.skipped_dup) {
        let ctx = it.context.clone().ok_or("ไม่มี snapshot ข้อมูลป้าย")?;
        for copy in 0..r.quantity {
            pages.push((
                ctx.clone(),
                if copy == r.quantity - 1 {
                    json!({"rule":r.id,"barcode":it.bar_code,"hash":it.price_hash})
                } else {
                    Value::Null
                },
            ));
        }
    }
    if !pages.is_empty() {
        if let Some(area) = crate::printable_area::read(&r.printer, &r.tray, &r.ip, &lbs).await? {
            if let Some(message) = area.warning(&lbs, r.template.starts_with("legacy:"), pages.len()) {
                return Err(message);
            }
        }
    }
    let page_size = if r.ip.is_empty() {
        (lbs.FNLbsCol.max(1) * lbs.FNLbsRow.max(1)) as usize
    } else {
        1
    };
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for (n, chunk) in pages.chunks(page_size).enumerate() {
        let payload = json!({"design":design,"route":route,"template":r.template,"size":lbs,"elements":elements,"legacy":r.template.starts_with("legacy:"),"contexts":chunk.iter().map(|(c,_)|c).collect::<Vec<_>>(),
        "marks":chunk.iter().map(|(_,m)|m).filter(|m|!m.is_null()).collect::<Vec<_>>(),"date":date});
        sqlx::query("INSERT INTO ABPrintUnit(job,seq,kind,payload) VALUES(?,?,'AUTO',?)")
            .bind(id)
            .bind(n as i64)
            .bind(payload.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    sqlx::query("UPDATE ABPrintJob SET status=?,total=? WHERE id=?")
        .bind(if pages.is_empty() { "EMPTY" } else { "QUEUED" })
        .bind(pages.chunks(page_size).len() as i64)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn send_unit(pool: &SqlitePool, kind: &str, p: Value, id: &str, seq: i64) -> Result<()> {
    match kind {
        "WINDOWS" => {
            let body = serde_json::from_value(p).map_err(|e| e.to_string())?;
            let res = crate::routes::post_windows_direct_print(
                State(AppState { pool: pool.clone() }),
                Json(body),
            )
            .await;
            if !res.status().is_success() {
                let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
                    .await
                    .map_err(|e| e.to_string())?;
                return Err(String::from_utf8_lossy(&bytes).into());
            }
            Ok(())
        }
        "RAW" => {
            let (ok, msg, _) = crate::print_drivers::tcp_send(
                p["ip"].as_str().ok_or("ไม่มี IP")?,
                p["port"].as_u64().unwrap_or(9100) as u16,
                p["command"].as_str().ok_or("ไม่มีคำสั่ง")?,
            )
            .await;
            if ok {
                Ok(())
            } else {
                Err(msg)
            }
        }
        "AUTO" => {
            let route: ResolvedPrinterRoute =
                serde_json::from_value(p["route"].clone()).map_err(|e| e.to_string())?;
            let size: TCNMLabelSize =
                serde_json::from_value(p["size"].clone()).map_err(|e| e.to_string())?;
            let elements = serde_json::from_value::<Vec<crate::templates::TemplateElement>>(
                p["elements"].clone(),
            )
            .map_err(|e| e.to_string())?;
            let contexts = p["contexts"].as_array().ok_or("ไม่มีข้อมูลป้าย")?;
            if (p["legacy"] == true || p["design"].is_object()) && route.lan_ip.is_none() {
                #[cfg(target_os = "windows")]
                {
                    let image = crate::legacy_page::render(&size, contexts, &route.printer_name, &p["design"]).await?;
                    let request = crate::routes::WindowsDirectPrintBody {
                        printer_name: route.printer_name, tray_name: Some(route.tray_name),
                        lbs_code: Some(size.FTLbsCode), lbs_name: Some(size.FTLbsName),
                        pri_type_label: None, test_mode: Some(false), page_images: Some(vec![image]),
                        paper_size: size.FTLbsPaperSize, paper_width_mm: size.FCLbsPaperWidth,
                        paper_height_mm: size.FCLbsPaperHeight,
                    };
                    let res = crate::routes::post_windows_direct_print(State(AppState { pool: pool.clone() }), Json(request)).await;
                    if !res.status().is_success() {
                        let bytes=axum::body::to_bytes(res.into_body(),65536).await.map_err(|e|e.to_string())?;
                        return Err(String::from_utf8_lossy(&bytes).into_owned());
                    }
                    return Ok(());
                }
            }
            if let Some(ip) = route.lan_ip {
                let brand = crate::print_drivers::PrinterBrand::from_str(&route.protocol)
                    .ok_or("ไม่รองรับภาษาเครื่องพิมพ์")?;
                let opts = crate::print_drivers::DriverOptions {
                    dpi: route.dpi,
                    label_width_mm: size.FCLbsWidth,
                    label_height_mm: size.FCLbsHeight,
                    gap_mm: size.FCLbsGapV.unwrap_or(2.0),
                    ..Default::default()
                };
                let elements = if p["legacy"] == true {
                    let promo = contexts.first().and_then(|c| c["legacyPriType"].as_str()) == Some("2");
                    let code = if promo {size.FTLbsTmplPmt.as_deref().unwrap_or("SH-100x38-PMT")}
                               else {size.FTLbsTmplNor.as_deref().unwrap_or("SH-100x38-NOR")};
                    crate::templates::get_templates().into_iter().find(|t|t.code == code)
                        .ok_or_else(||format!("ไม่พบแบบพิมพ์ LAN {code}"))?.elements
                } else {elements};
                let labels = contexts.iter().map(|c| (c.clone(), 1)).collect::<Vec<_>>();
                let cmd = crate::print_drivers::generate_commands(brand, &labels, &elements, &opts)?;
                let (ok, msg, _) = crate::print_drivers::tcp_send(&ip, route.lan_port, &cmd).await;
                if ok {
                    Ok(())
                } else {
                    Err(msg)
                }
            } else {
                #[cfg(target_os = "windows")]
                {
                    crate::label_renderer::send_windows_headless_page(
                        &route,
                        Some(&size),
                        &elements,
                        contexts,
                        &format!("AdaBarprint_{}_{}", id, seq),
                    )
                    .await
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = (route, size, elements, contexts, id, seq);
                    Err("Windows printing unavailable".into())
                }
            }
        }
        _ => Err("Unknown unit kind".into()),
    }
}

pub async fn recover(pool: &SqlitePool) -> Result<()> {
    // Called only by the exclusive OS-lock owner, never by a second UI instance.
    sqlx::raw_sql("UPDATE ABPrintUnit SET status='UNKNOWN' WHERE status='SENDING';
      UPDATE ABPrintJob SET status='NEEDS_REVIEW',error='โปรแกรมหยุดขณะส่งพิมพ์ ต้องตรวจหน้าที่ออกแล้วก่อนทำต่อ' WHERE status='SENDING';
      UPDATE ABPrintJob SET status='WAIT_DATA' WHERE status='PREPARING';")
      .execute(pool).await.map_err(|e|e.to_string())?;
    Ok(())
}
async fn worker_tick(pool: &SqlitePool) -> Result<()> {
    let head:Option<(String,String,String,String)>=sqlx::query_as("SELECT id,status,config,source FROM ABPrintJob WHERE status NOT IN ('SUBMITTED','EMPTY','NO_MATCH','ALREADY_SUBMITTED','FAILED','CANCELLED') ORDER BY seq LIMIT 1")
      .fetch_optional(pool).await.map_err(|e|e.to_string())?;
    let Some((id, status, config, source)) = head else {
        return Ok(());
    };
    if ["BLOCKED", "NEEDS_REVIEW"].contains(&status.as_str()) {
        return Ok(());
    }
    if status == "WAIT_DATA" {
        if !fresh(pool).await {
            return Ok(());
        }
        let claim = sqlx::query(
            "UPDATE ABPrintJob SET status='PREPARING' WHERE id=? AND status='WAIT_DATA'",
        )
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        if claim.rows_affected() != 1 {
            return Ok(());
        }
        if let Err(e) = prepare_auto(pool, &id, &config).await {
            sqlx::query("UPDATE ABPrintJob SET status='BLOCKED',error=? WHERE id=?")
                .bind(e)
                .bind(&id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if status != "QUEUED" {
        return Ok(());
    }
    let unit:Option<(i64,String,String)>=sqlx::query_as("SELECT seq,kind,payload FROM ABPrintUnit WHERE job=? AND status='READY' ORDER BY seq LIMIT 1")
      .bind(&id).fetch_optional(pool).await.map_err(|e|e.to_string())?;
    let Some((seq, kind, payload)) = unit else {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        sqlx::query("UPDATE ABPrintJob SET status='SUBMITTED',completed=total WHERE id=? AND status='QUEUED'").bind(&id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
        if source == "MANUAL" {
            if let Some(old_id) = serde_json::from_str::<Value>(&config)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
            {
                sqlx::query("UPDATE TCNTLabelJob SET FTLjbStatus='4',FDLjbTo=? WHERE FTLjbJobID=?")
                    .bind(now())
                    .bind(&old_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query("UPDATE TCNTLabelLog SET FTLlgStatus='4' WHERE FTLjbJobID=?")
                    .bind(&old_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        if source == "AUTO" {
            let r: PrintRule = serde_json::from_str(&config).map_err(|e| e.to_string())?;
            put(
                pool,
                &format!("ruleSince:{}", r.id),
                &Local::now().format("%Y-%m-%d").to_string(),
            )
            .await?;
        }
        return Ok(());
    };
    let p: Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
    if source == "AUTO" && p["date"].as_str() != Some(&Local::now().format("%Y-%m-%d").to_string())
    {
        sqlx::query("UPDATE ABPrintJob SET status='BLOCKED',error='งานข้ามวัน ต้องยกเลิกและสร้างใหม่เพื่อใช้ราคาปัจจุบัน' WHERE id=?").bind(&id).execute(pool).await.map_err(|e|e.to_string())?;
        return Ok(());
    }
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let claim =
        sqlx::query("UPDATE ABPrintJob SET status='SENDING' WHERE id=? AND status='QUEUED'")
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    if claim.rows_affected() != 1 {
        return Ok(());
    }
    sqlx::query("UPDATE ABPrintUnit SET status='SENDING' WHERE job=? AND seq=? AND status='READY'")
        .bind(&id)
        .bind(seq)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    let result = tokio::time::timeout(
        Duration::from_secs(180),
        send_unit(pool, &kind, p.clone(), &id, seq),
    )
    .await
    .unwrap_or_else(|_| Err("หมดเวลารอผล อาจส่งข้อมูลบางส่วนแล้ว".into()));
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    if result.is_ok() {
        sqlx::query("UPDATE ABPrintUnit SET status='SUBMITTED' WHERE job=? AND seq=?")
            .bind(&id)
            .bind(seq)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(marks) = p["marks"].as_array() {
            for m in marks {
                sqlx::query("INSERT INTO ABPrintWatermark(rule,barcode,hash) VALUES(?,?,?) ON CONFLICT(rule,barcode) DO UPDATE SET hash=excluded.hash")
          .bind(m["rule"].as_str()).bind(m["barcode"].as_str()).bind(m["hash"].as_str()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
            }
        }
        sqlx::query("UPDATE ABPrintJob SET status='QUEUED',completed=(SELECT COUNT(*) FROM ABPrintUnit WHERE job=? AND status='SUBMITTED') WHERE id=?")
        .bind(&id).bind(&id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    } else {
        let e = result.unwrap_err();
        sqlx::query("UPDATE ABPrintUnit SET status='UNKNOWN',error=? WHERE job=? AND seq=?")
            .bind(&e)
            .bind(&id)
            .bind(seq)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("UPDATE ABPrintJob SET status='NEEDS_REVIEW',error=? WHERE id=?")
            .bind(&e)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
async fn scheduler_tick_at(pool: &SqlitePool, n: chrono::DateTime<Local>) -> Result<()> {
    let mut rs = rules(pool).await?;
    let prefs:Value=serde_json::from_str(&setting(pool,&format!("bulk:{}",crate::storage::configured_branch(pool).await)).await).unwrap_or(Value::Null);
    let order:Vec<String>=prefs["rows"].as_array().map(|rows|rows.iter().filter_map(|r|r["id"].as_str().map(str::to_owned)).collect()).unwrap_or_default();
    rs.sort_by(|a,b|a.time.cmp(&b.time).then(order.iter().position(|id|id==&a.id).unwrap_or(usize::MAX).cmp(&order.iter().position(|id|id==&b.id).unwrap_or(usize::MAX))).then(a.id.cmp(&b.id)));
    for r in rs.into_iter().filter(|r| r.enabled) {
        if r.days
            .as_bytes()
            .get(n.weekday().num_days_from_monday() as usize)
            != Some(&b'1')
        {
            continue;
        }
        let Ok(t) = NaiveTime::parse_from_str(&r.time, "%H:%M") else {
            continue;
        };
        let elapsed = (n.time() - t).num_seconds();
        if elapsed < 0 || (!r.catch_up && elapsed > 60) {
            continue;
        }
        let dedup = format!("{}:{}", r.id, n.format("%Y-%m-%d"));
        let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ABPrintJob WHERE dedup=?")
            .bind(&dedup)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        if exists > 0 {
            continue;
        }
        let _ = insert_job(
            pool,
            "AUTO",
            &r.name,
            serde_json::to_value(&r).unwrap(),
            Some(&dedup),
            "WAIT_DATA",
            vec![],
        )
        .await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn manual_and_profile_share_fifo_transport_without_duplicate_send() {
        use tokio::io::AsyncReadExt;
        let p = fixture().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let receiver = tokio::spawn(async move {
            let mut received = vec![];
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = vec![];
                stream.read_to_end(&mut bytes).await.unwrap();
                received.push(String::from_utf8(bytes).unwrap());
            }
            assert!(tokio::time::timeout(Duration::from_millis(150), listener.accept()).await.is_err());
            received
        });
        let mut ids = vec![];
        for (source, command) in [("MANUAL", "manual-test"), ("PROFILE", "profile-test")] {
            ids.push(insert_job(&p,source,"test",Value::Null,None,"QUEUED",
                vec![("RAW".into(),json!({"ip":"127.0.0.1","port":port,"command":command}))]).await.unwrap());
        }
        for _ in 0..5 { worker_tick(&p).await.unwrap(); }
        let received = tokio::time::timeout(Duration::from_secs(5),receiver).await.unwrap().unwrap();
        assert_eq!(received,vec!["manual-test","profile-test"]);
        for id in ids { assert_eq!(status(&p,&id).await,"SUBMITTED"); }
        p.close().await;
    }
    async fn fixture() -> SqlitePool {
        let p = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        ensure_schema(&p).await.unwrap();
        p
    }
    async fn bulk_fixture() -> (SqlitePool, Vec<PrintRule>) {
        let p=fixture().await;
        sqlx::raw_sql("CREATE TABLE ABStorage(id INTEGER PRIMARY KEY,branch TEXT);INSERT INTO ABStorage VALUES(1,'B1');").execute(&p).await.unwrap();
        let mut rs=Vec::new();
        for id in ["a","b"] {
            let r:PrintRule=serde_json::from_value(json!({"id":id,"name":id,"enabled":true,"branch":"B1","source":"PRICE_CHG","time":"01:00","days":"1111111","size":"old","template":"old","printer":"old","tray":"","protocol":"WINDOWS","ip":"","port":9100,"dpi":203,"quantity":1,"catchUp":true})).unwrap();
            sqlx::query("INSERT INTO ABPrintRule VALUES(?,?)").bind(id).bind(serde_json::to_string(&r).unwrap()).execute(&p).await.unwrap();
            sqlx::query("INSERT INTO ABPrintWatermark VALUES(?,'barcode','sent-hash')").bind(id).execute(&p).await.unwrap();
            put(&p,&format!("ruleSince:{}",id),"2020-01-01").await.unwrap();
            let mut stopped=r;stopped.enabled=false;rs.push(stopped);
        }
        ensure_schema(&p).await.unwrap();
        (p,rs)
    }
    #[tokio::test]
    async fn legacy_profile_codes_survive_insertion_deletion_and_restart() {
        let (pool,_) = bulk_fixture().await;
        let codes = || sqlx::query_as::<_,(String,String)>("SELECT id,json_extract(config,'$.code') FROM ABPrintRule ORDER BY id");
        assert_eq!(codes().fetch_all(&pool).await.unwrap(),[("a".into(),"001".into()),("b".into(),"002".into())]);
        sqlx::query("DELETE FROM ABPrintRule WHERE id='a'").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO ABPrintRule SELECT '0',json_remove(json_set(config,'$.id','0'),'$.code') FROM ABPrintRule WHERE id='b'")
            .execute(&pool).await.unwrap();
        ensure_schema(&pool).await.unwrap();
        ensure_schema(&pool).await.unwrap();
        assert_eq!(codes().fetch_all(&pool).await.unwrap(),[("0".into(),"001".into()),("b".into(),"002".into())]);
    }

    #[tokio::test]
    async fn auto_rules_ignore_manual_and_retired_shared_settings() {
        let (p, _) = bulk_fixture().await;
        let before = serde_json::to_value(rules(&p).await.unwrap()).unwrap();
        for key in ["eventProfiles:B1", "templateRoutes:LOCAL:old"] {
            put(&p, key, r#"{"PRICE_CHG":{"template":"changed","printer":"changed"}}"#).await.unwrap();
        }
        let after = serde_json::to_value(rules(&p).await.unwrap()).unwrap();
        assert_eq!(before, after);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintJob").fetch_one(&p).await.unwrap(),0);
    }
    #[tokio::test]
    async fn bulk_save_keeps_ids_watermarks_and_does_not_enqueue() {
        let (p,rs)=bulk_fixture().await;
        persist_bulk(&p,BulkRules{branch:"B1".into(),rules:rs,preferences:json!({"branch":"B1","enabled":false})}).await.unwrap();
        assert!(rules(&p).await.unwrap().iter().all(|r|!r.enabled));
        assert_eq!(setting(&p,"ruleSince:a").await,"2020-01-01");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintWatermark").fetch_one(&p).await.unwrap(),2);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintJob").fetch_one(&p).await.unwrap(),0);
    }
    #[tokio::test]
    async fn bulk_write_failure_rolls_back_all_rows_and_preferences() {
        let (p,rs)=bulk_fixture().await;
        sqlx::raw_sql("CREATE TRIGGER fail_second BEFORE UPDATE ON ABPrintRule WHEN NEW.id='b' BEGIN SELECT RAISE(ABORT,'test failure'); END;").execute(&p).await.unwrap();
        assert!(persist_bulk(&p,BulkRules{branch:"B1".into(),rules:rs,preferences:json!({"branch":"B1"})}).await.is_err());
        assert!(rules(&p).await.unwrap().iter().all(|r|r.enabled));
        assert!(setting(&p,"bulk:B1").await.is_empty());
    }
    #[tokio::test]
    async fn bulk_validation_rejects_mixed_branches_before_any_write() {
        let (p,mut rs)=bulk_fixture().await;rs[1].branch="B2".into();
        assert!(persist_bulk(&p,BulkRules{branch:"B1".into(),rules:rs,preferences:json!({"branch":"B1"})}).await.is_err());
        assert!(rules(&p).await.unwrap().iter().all(|r|r.enabled));
    }
    async fn status(p: &SqlitePool, id: &str) -> String {
        sqlx::query_scalar("SELECT status FROM ABPrintJob WHERE id=?")
            .bind(id)
            .fetch_one(p)
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn fifo_blocks_b_until_a_is_explicitly_cancelled() {
        let p = fixture().await;
        let a = insert_job(&p, "AUTO", "A", json!({}), None, "BLOCKED", vec![])
            .await
            .unwrap();
        let b = insert_job(&p, "MANUAL", "B", json!({}), None, "QUEUED", vec![])
            .await
            .unwrap();
        worker_tick(&p).await.unwrap();
        assert_eq!(status(&p, &b).await, "QUEUED");
        let res = queue_action(
            State(AppState { pool: p.clone() }),
            Json(json!({"id":a,"action":"CANCEL"})),
        )
        .await;
        assert!(res.status().is_success());
        worker_tick(&p).await.unwrap();
        assert_eq!(status(&p, &b).await, "SUBMITTED");
    }
    #[tokio::test]
    async fn crash_marks_only_inflight_page_unknown_and_keeps_later_pages_ready() {
        let p = fixture().await;
        let id = insert_job(
            &p,
            "MANUAL",
            "A",
            Value::Null,
            None,
            "SENDING",
            vec![
                ("WINDOWS".into(), json!({})),
                ("WINDOWS".into(), json!({})),
                ("WINDOWS".into(), json!({})),
            ],
        )
        .await
        .unwrap();
        sqlx::query("UPDATE ABPrintUnit SET status=CASE seq WHEN 0 THEN 'SUBMITTED' WHEN 1 THEN 'SENDING' ELSE 'READY' END WHERE job=?").bind(&id).execute(&p).await.unwrap();
        recover(&p).await.unwrap();
        assert_eq!(status(&p, &id).await, "NEEDS_REVIEW");
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT status FROM ABPrintUnit WHERE job=? ORDER BY seq")
                .bind(&id)
                .fetch_all(&p)
                .await
                .unwrap();
        assert_eq!(rows, vec!["SUBMITTED", "UNKNOWN", "READY"]);
        let res = queue_action(
            State(AppState { pool: p.clone() }),
            Json(json!({"id":id,"action":"RETRY"})),
        )
        .await;
        assert!(res.status().is_success());
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT status FROM ABPrintUnit WHERE job=? ORDER BY seq")
                .bind(&id)
                .fetch_all(&p)
                .await
                .unwrap();
        assert_eq!(rows, vec!["SUBMITTED", "READY", "READY"]);
    }
    #[tokio::test]
    async fn daily_dedup_and_manual_units_are_atomic() {
        let p = fixture().await;
        let id = insert_job(
            &p,
            "AUTO",
            "round",
            Value::Null,
            Some("rule:2026-11-20"),
            "WAIT_DATA",
            vec![],
        )
        .await
        .unwrap();
        assert!(insert_job(
            &p,
            "AUTO",
            "round",
            Value::Null,
            Some("rule:2026-11-20"),
            "WAIT_DATA",
            vec![]
        )
        .await
        .is_err());
        assert_eq!(status(&p, &id).await, "WAIT_DATA");
        sqlx::query("CREATE TRIGGER fail_unit BEFORE INSERT ON ABPrintUnit BEGIN SELECT RAISE(ABORT,'test'); END").execute(&p).await.unwrap();
        assert!(insert_job(
            &p,
            "MANUAL",
            "bad",
            Value::Null,
            None,
            "QUEUED",
            vec![("RAW".into(), json!({}))]
        )
        .await
        .is_err());
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ABPrintJob")
            .fetch_one(&p)
            .await
            .unwrap();
        assert_eq!(n, 1);
    }
    #[tokio::test]
    async fn no_fresh_sync_keeps_auto_and_following_manual_waiting() {
        let p = fixture().await;
        let a = insert_job(&p, "AUTO", "A", json!({}), None, "WAIT_DATA", vec![])
            .await
            .unwrap();
        let b = insert_job(&p, "MANUAL", "B", Value::Null, None, "QUEUED", vec![])
            .await
            .unwrap();
        worker_tick(&p).await.unwrap();
        assert_eq!(status(&p, &a).await, "WAIT_DATA");
        assert_eq!(status(&p, &b).await, "QUEUED");
    }
    #[tokio::test]
    async fn sending_page_cannot_be_cancelled_or_retried_concurrently() {
        let p = fixture().await;
        let id = insert_job(&p, "MANUAL", "A", Value::Null, None, "SENDING", vec![])
            .await
            .unwrap();
        for action in ["RETRY", "CANCEL", "CONFIRM_SENT"] {
            let res = queue_action(
                State(AppState { pool: p.clone() }),
                Json(json!({"id":id,"action":action})),
            )
            .await;
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        }
    }
    #[tokio::test]
    async fn morning_catchup_queues_a_before_b_even_while_head_is_blocked() {
        use chrono::TimeZone;
        let p = fixture().await;
        let head = insert_job(&p, "MANUAL", "old", Value::Null, None, "BLOCKED", vec![])
            .await
            .unwrap();
        for (id, time, catch_up) in [
            ("B", "02:00", true),
            ("A", "01:00", true),
            ("C", "03:00", false),
        ] {
            let r = PrintRule {
                id: id.into(),
                code: id.into(),
                name: id.into(),
                enabled: true,
                branch: "B01".into(),
                source: "PRICE_CHG".into(),
                time: time.into(),
                days: "1111111".into(),
                size: "size".into(),
                template: "template".into(),
                printer: "test".into(),
                tray: String::new(),
                protocol: "WINDOWS".into(),
                ip: String::new(),
                port: 9100,
                dpi: 203,
                quantity: 1,
                catch_up,
                offset_days: 0,
                targets: vec![],
            };
            sqlx::query("INSERT INTO ABPrintRule(id,config) VALUES(?,?)")
                .bind(id)
                .bind(serde_json::to_string(&r).unwrap())
                .execute(&p)
                .await
                .unwrap();
        }
        let morning = Local.with_ymd_and_hms(2026, 11, 20, 8, 0, 0).unwrap();
        scheduler_tick_at(&p, morning).await.unwrap();
        scheduler_tick_at(&p, morning).await.unwrap();
        let names: Vec<String> = sqlx::query_scalar("SELECT name FROM ABPrintJob ORDER BY seq")
            .fetch_all(&p)
            .await
            .unwrap();
        assert_eq!(names, vec!["old", "A", "B"]);
        assert_eq!(status(&p, &head).await, "BLOCKED");
    }

    #[tokio::test]
    async fn multi_target_profile_validates_and_normalizes_targets() {
        let p = fixture().await;
        sqlx::raw_sql(
            "CREATE TABLE ABStorage(id INTEGER PRIMARY KEY,branch TEXT);
             INSERT INTO ABStorage VALUES(1,'B01');
             CREATE TABLE TCNMLabelSize(FTLbsCode TEXT PRIMARY KEY, FTLbsGroup TEXT, FTLbsName TEXT, FCLbsWidth REAL, FCLbsHeight REAL, FNLbsCol INTEGER, FNLbsRow INTEGER, FCLbsMagTop REAL, FCLbsMagLeft REAL, FCLbsMagBottom REAL, FCLbsMagRight REAL, FCLbsGapH REAL, FCLbsGapV REAL, FNLbsQtyPageNor INTEGER, FNLbsQtyPagePmt INTEGER, FTLbsTmplNor TEXT, FTLbsTmplPmt TEXT, FTLbsHhtCode TEXT, FTLbsStaUse TEXT, FTLbsPaperSize TEXT, FCLbsPaperWidth REAL, FCLbsPaperHeight REAL, FCLbsOffsetX REAL, FCLbsOffsetY REAL, FTPprCode TEXT, FTLbsStdTmplNor TEXT, FTLbsStdTmplPmt TEXT, FTLbsRmk TEXT, FDCreateOn TEXT, FTCreateBy TEXT, FDLastUpdOn TEXT, FTLastUpdBy TEXT);
             CREATE TABLE TCNMLabelTmpl(FTLtmCode TEXT PRIMARY KEY, FTLtmName TEXT, FTLbsCode TEXT, FTLtmPriType TEXT, FTLtmLayout TEXT, FTLtmStaUse TEXT, FTLtmMode TEXT, FTPprCode TEXT);
             INSERT INTO TCNMLabelSize(FTLbsCode,FTLbsGroup,FTLbsName,FCLbsWidth,FCLbsHeight,FTLbsStaUse) VALUES('S1','1','25x30',25,30,'1'),('S2','1','Small',40,20,'1');
             INSERT INTO TCNMLabelTmpl(FTLtmCode,FTLtmName,FTLbsCode,FTLtmLayout,FTLtmStaUse) VALUES('T1','25x30 Nor','S1','{\"elements\":[{\"type\":\"text\"}]}','1'),('T2','Small Nor','S2','{\"elements\":[{\"type\":\"text\"}]}','1');"
        ).execute(&p).await.unwrap();

        let rule = PrintRule {
            id: "prof-multi".into(),
            code: "001".into(),
            name: "ราคาเปลี่ยน 2 กระดาษ".into(),
            enabled: true,
            branch: "B01".into(),
            source: "PRICE_CHG".into(),
            time: "00:00".into(),
            days: "1111111".into(),
            size: String::new(),
            template: String::new(),
            printer: String::new(),
            tray: String::new(),
            protocol: "WINDOWS".into(),
            ip: String::new(),
            port: 9100,
            dpi: 203,
            quantity: 1,
            catch_up: false,
            offset_days: 0,
            targets: vec![
                PrintProfileTarget {
                    size: "S1".into(),
                    template: "T1".into(),
                    printer: "Printer-A".into(),
                    tray: "Tray 1".into(),
                    protocol: "WINDOWS".into(),
                    ip: String::new(),
                    port: 9100,
                    dpi: 203,
                    quantity: 1,
                },
                PrintProfileTarget {
                    size: "S2".into(),
                    template: "T2".into(),
                    printer: "Printer-B".into(),
                    tray: String::new(),
                    protocol: "WINDOWS".into(),
                    ip: String::new(),
                    port: 9100,
                    dpi: 203,
                    quantity: 2,
                },
            ],
        };

        let res = save_rule(State(AppState { pool: p.clone() }), Json(rule)).await;
        assert!(res.status().is_success());
        let saved = rules(&p).await.unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].code, "001");
        assert_eq!(effective_targets(&saved[0]).len(), 2);
        assert_eq!(saved[0].size, "S1");
        assert_eq!(effective_targets(&saved[0])[1].printer, "Printer-B");
        assert_eq!(effective_targets(&saved[0])[1].quantity, 2);
        let mut legacy = saved[0].clone();
        legacy.targets[1].template = "legacy:S1".into();
        assert!(validate_rule(&p, &legacy).await.is_err());
        legacy.targets[1].template = "legacy:S2".into();
        let res = save_rule(State(AppState { pool: p.clone() }), Json(legacy)).await;
        assert!(res.status().is_success());
        assert_eq!(effective_targets(&rules(&p).await.unwrap()[0])[1].template, "legacy:S2");
    }
}
pub async fn run_profile_api(State(st): State<AppState>, Json(body): Json<Value>) -> Response {
    let key = body["code"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if key.is_empty() {
        return err("กรุณาระบุรหัส Profile ที่ต้องการสั่งพิมพ์");
    }
    match prepare_profile_job(&st.pool, &key, true).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => err(e),
    }
}

pub async fn enqueue_profile(pool: &SqlitePool, profile_code_or_id: &str) -> Result<Value> {
    prepare_profile_job(pool, profile_code_or_id, false).await
}
async fn prepare_profile_job(pool: &SqlitePool, profile_code_or_id: &str, reprint: bool) -> Result<Value> {
    let run_date = Local::now().date_naive();
    let needle = profile_code_or_id.trim();
    if needle.is_empty() {
        return Err("กรุณาระบุรหัส Profile".into());
    }
    let all = rules(pool).await?;
    let r = all
        .into_iter()
        .find(|x| x.code.eq_ignore_ascii_case(needle) || x.id == needle)
        .ok_or_else(|| format!("ไม่พบ Profile รหัส \"{needle}\""))?;

    let target_date = crate::delta_engine::target_date_with_offset(
        run_date,
        r.offset_days,
    );
    let today = run_date.to_string();
    let targets = effective_targets(&r);
    let job_name = format!("[{}] {}", r.code, r.name);
    let mut config = serde_json::to_value(&r).map_err(|e| e.to_string())?;
    config["targetDate"] = json!(target_date);
    let preparation = async {
        validate_rule(pool, &r).await?;
        let mut prepared = Vec::with_capacity(targets.len());
        for target in &targets {
            let layout = load_layout_for(pool, &target.size, &target.template, target.ip.is_empty()).await?;
            let items = candidates_for_target(pool, &r, target, &target_date).await?;
            let area = if items.is_empty() { Ok(None) } else {
                crate::printable_area::read(&target.printer, &target.tray, &target.ip, &layout.0).await
            };
            let design=windows_design(pool,&target.size,&target.template,target.ip.is_empty()).await?;
            prepared.push((layout, items, area,design));
        }
        Ok::<_, String>(prepared)
    }.await;
    let prepared = match preparation {
        Ok(prepared) => prepared,
        Err(e) => {
            let message = format!("เตรียมข้อมูลป้ายวันที่ {target_date} ไม่สำเร็จ: {e}");
            sqlx::query("INSERT INTO ABPrintJob(id,source,name,status,created,config,total,error) VALUES(?,'PROFILE',?,'FAILED',?,?,0,?)")
                .bind(uid()).bind(&job_name).bind(now()).bind(config.to_string()).bind(&message)
                .execute(pool).await.map_err(|log_error| format!("{message} (บันทึกประวัติไม่ได้: {log_error})"))?;
            return Err(message);
        }
    };
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.map_err(|e|e.to_string())?;
    let mut total_items = 0;
    let mut skipped_items = 0;
    let mut units = Vec::new();
    for (idx,(target,((lbs,elements),items,area,design))) in targets.iter().zip(prepared).enumerate() {
        let scope = json!(["profile",r.id,idx]).to_string();
        // Limit the lookup to labels eligible on the target date, even after years of printing.
        let keys: Vec<String> = items.iter().map(|it| json!([it.pdt_code,it.bar_code]).to_string()).collect();
        let previous: std::collections::HashMap<String,String> = sqlx::query_as::<_,(String,String)>(
            "SELECT barcode,hash FROM ABPrintWatermark WHERE rule=? AND barcode IN (SELECT value FROM json_each(?))")
            .bind(&scope).bind(json!(keys).to_string()).fetch_all(&mut *tx).await
            .map_err(|e|e.to_string())?.into_iter().collect();
        let route = ResolvedPrinterRoute {
            lbs_code: target.size.clone(),
            pri_type: "1".into(),
            protocol: target.protocol.clone(),
            printer_name: target.printer.clone(),
            tray_name: target.tray.clone(),
            lan_ip: if target.ip.is_empty() { None } else { Some(target.ip.clone()) },
            lan_port: target.port,
            dpi: target.dpi,
        };
        let mut pages = Vec::new();
        use sha2::{Digest,Sha256};
        let design_hash = if design.is_null() {None} else {Some(format!("{:x}",Sha256::digest(design.to_string())))};
        for it in items {
            let key = json!([it.pdt_code,it.bar_code]).to_string();
            let mut identity=json!([target_date,r.source,target,lbs,it.price_hash]);
            if let Some(hash) = &design_hash { identity.as_array_mut().unwrap().push(json!(hash)); }
            let hash = format!("{:x}",Sha256::digest(identity.to_string()));
            // Each transport acknowledgement records the last copy accepted for this label.
            // A cancelled, partially submitted run can resume without repeating its first copies.
            let sent_copies = if reprint { 0 } else {
                previous.get(&key).and_then(|value|value.rsplit_once(':'))
                    .filter(|(saved,_)|*saved == hash)
                    .and_then(|(_,copies)|copies.parse::<i64>().ok()).unwrap_or(0)
            };
            if sent_copies >= target.quantity { skipped_items += 1; continue; }
            total_items += 1;
            let ctx = it.context.ok_or("ไม่มี snapshot ข้อมูลป้าย")?;
            for copy in sent_copies + 1..=target.quantity {
                let mark = json!({"rule":scope,"barcode":key,"hash":format!("{hash}:{copy}")});
                pages.push((ctx.clone(),mark));
            }
        }
        if !pages.is_empty() {
            let warning = match area {
                Ok(area) => area.and_then(|a| a.warning(&lbs, target.template.starts_with("legacy:"), pages.len())),
                Err(e) => Some(e),
            };
            if let Some(message) = warning {
                let message = format!("{}: {message}", target.printer);
                sqlx::query("INSERT INTO ABPrintJob(id,source,name,status,created,config,total,error) VALUES(?,'PROFILE',?,'FAILED',?,?,0,?)")
                    .bind(uid()).bind(&job_name).bind(now()).bind(config.to_string()).bind(&message)
                    .execute(&mut *tx).await.map_err(|e| format!("{message} ({e})"))?;
                tx.commit().await.map_err(|e| e.to_string())?;
                return Err(message);
            }
        }
        let page_size = if target.ip.is_empty() {
            (lbs.FNLbsCol.max(1) * lbs.FNLbsRow.max(1)) as usize
        } else {
            1
        };
        for chunk in pages.chunks(page_size) {
            let payload = json!({
                "route": route,
                "template": target.template,
                "design":design,
                "size": lbs,
                "elements": elements,
                "legacy": target.template.starts_with("legacy:"),
                "contexts": chunk.iter().map(|(ctx,_)|ctx).collect::<Vec<_>>(),
                "marks": chunk.iter().map(|(_,mark)|mark).collect::<Vec<_>>(),
                "date": today
            });
            units.push(("AUTO".into(), payload));
        }
    }

    let initial_status = if !units.is_empty() { "QUEUED" } else if skipped_items > 0 { "ALREADY_SUBMITTED" } else { "NO_MATCH" };
    let total_units=units.len();
    let job_id=insert_job_rows(&mut tx,"PROFILE",&job_name,config,None,initial_status,units).await?;
    tx.commit().await.map_err(|e|e.to_string())?;
    if total_units==0 {
        return Ok(json!({
            "ok": true,
            "jobId": job_id,
            "profileCode": r.code,
            "profileName": r.name,
            "targetDate": target_date,
            "status": initial_status,
            "itemCount": 0,
            "alreadySubmittedItems": skipped_items,
            "totalUnits": 0,
            "printedUnits": 0
        }));
    }

    Ok(json!({
        "ok": true, "jobId": job_id, "profileCode": r.code, "profileName": r.name,
        "targetDate": target_date, "status": "QUEUED", "itemCount": total_items, "alreadySubmittedItems": skipped_items,
        "totalUnits": total_units, "printedUnits": 0
    }))
}

/// CLI waits for the same persistent FIFO worker used by manual printing.
/// Only the OS-lock owner may send; another desktop/CLI process only watches.
pub async fn execute_profile_cli(pool: &SqlitePool, profile_code_or_id: &str) -> Result<Value> {
    let mut result = enqueue_profile(pool, profile_code_or_id).await?;
    if matches!(result["status"].as_str(), Some("NO_MATCH" | "ALREADY_SUBMITTED")) { return Ok(result); }
    let job = result["jobId"].as_str().ok_or("ไม่มีรหัสงานพิมพ์")?.to_string();
    spawn(pool.clone());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3600);
    loop {
        let (status, completed, error): (String, i64, Option<String>) =
            sqlx::query_as("SELECT status,completed,error FROM ABPrintJob WHERE id=?")
                .bind(&job).fetch_one(pool).await.map_err(|e|e.to_string())?;
        match status.as_str() {
            "SUBMITTED" => {
                result["status"] = json!(status);
                result["printedUnits"] = json!(completed);
                return Ok(result);
            }
            "NEEDS_REVIEW" | "BLOCKED" | "CANCELLED" =>
                return Err(format!("งาน {job}: {status}: {}", error.unwrap_or_default())),
            _ => {}
        }
        let blocker: Option<String> = sqlx::query_scalar("SELECT id FROM ABPrintJob WHERE seq < (SELECT seq FROM ABPrintJob WHERE id=?) AND status IN ('BLOCKED','NEEDS_REVIEW') ORDER BY seq LIMIT 1")
            .bind(&job).fetch_optional(pool).await.map_err(|e|e.to_string())?;
        if let Some(blocker) = blocker {
            return Err(format!("งาน {job} อยู่ในคิวแล้ว แต่รอตรวจสอบงานก่อนหน้า {blocker}; อย่าเรียกซ้ำ ให้จัดการในคิวงานพิมพ์"));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!("หมดเวลารอ งาน {job} ยังอยู่ในคิว ตรวจสอบสถานะก่อนเรียกซ้ำ"));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub fn spawn(pool: SqlitePool) {
    tokio::spawn(async move {
        if let Err(e) = ensure_schema(&pool).await {
            eprintln!("print schema: {e}");
            return;
        }
        // OS releases this exclusive file handle on crash. No stale lock deletion or timeout takeover.
        #[cfg(target_os = "windows")]
        let _owner = {
            use std::os::windows::fs::OpenOptionsExt;
            let path = crate::db::resolve_db_path().with_extension("print-agent.lock");
            loop {
                if pool.is_closed() {
                    return;
                }
                match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .share_mode(0)
                    .open(&path)
                {
                    Ok(f) => break f,
                    Err(_) => tokio::time::sleep(Duration::from_secs(5)).await,
                }
            }
        };

        if let Err(e) = recover(&pool).await {
            eprintln!("print recovery: {e}");
            return;
        }
        let worker_pool = pool.clone();
        let worker = tokio::spawn(async move {
            loop {
                if let Err(e) = worker_tick(&worker_pool).await {
                    eprintln!("print worker: {e}");
                    let _ = recover(&worker_pool).await;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        let mut next_cleanup = tokio::time::Instant::now();
        loop {
            if pool.is_closed() {
                worker.abort();
                break;
            }
            if tokio::time::Instant::now() >= next_cleanup {
                let delay = match archive_finished_jobs(&pool).await {
                    Ok(10) => 5,
                    Ok(_) => 86400,
                    Err(e) => { eprintln!("print history retention: {e}"); 300 }
                };
                next_cleanup = tokio::time::Instant::now() + Duration::from_secs(delay);
            }
            let _ = put(&pool, "heartbeat", &now()).await;
            // In-app timed scheduler loop disabled: scheduled Profiles are triggered externally
            // via Windows Task Scheduler / AdaSchedule passing the Profile Code as a CLI argument.
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
