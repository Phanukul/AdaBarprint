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
use chrono::{Datelike, Local, NaiveTime};
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

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrintRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub branch: String,
    pub source: String,
    pub time: String,
    pub days: String,
    pub size: String,
    pub template: String,
    pub printer: String,
    pub tray: String,
    pub protocol: String,
    pub ip: String,
    pub port: u16,
    pub dpi: u32,
    pub quantity: i64,
    pub catch_up: bool,
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
      CREATE TABLE IF NOT EXISTS ABPrintAudit(id INTEGER PRIMARY KEY AUTOINCREMENT,job TEXT,action TEXT,created TEXT);")
      .execute(pool).await?;
    Ok(())
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
    let mut result=Vec::new();
    for raw in rows {let r:PrintRule=serde_json::from_str(&raw).map_err(|e|e.to_string())?;result.push(r);}Ok(result)
}
async fn rule(pool: &SqlitePool, id: &str) -> Result<PrintRule> {
    rules(pool)
        .await?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or("ไม่พบกฎพิมพ์".into())
}
pub async fn get_center(State(st): State<AppState>) -> Response {
    let jobs=sqlx::query_as::<_,QueueJob>("SELECT id,seq,source,name,status,created,error,completed,total FROM ABPrintJob WHERE status NOT IN ('SUBMITTED','EMPTY','CANCELLED') OR id IN (SELECT id FROM ABPrintJob WHERE status IN ('SUBMITTED','EMPTY','CANCELLED') ORDER BY seq DESC LIMIT 100) ORDER BY seq DESC").fetch_all(&st.pool).await;
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
        return Err("กรุณาระบุชื่องานและสาขา".into());
    }
    if ![
        "PRICE_CHG",
        "PROMO_START",
        "PROMO_END_REVERT",
        "NEW_PDT",
        "ALL_PRODUCTS",
    ]
    .contains(&r.source.as_str())
    {
        return Err("ประเภทข้อมูลไม่รองรับ".into());
    }
    NaiveTime::parse_from_str(&r.time, "%H:%M").map_err(|_| "เวลาไม่ถูกต้อง")?;
    if r.days.len() != 7 || !r.days.bytes().all(|b| b == b'0' || b == b'1') || !r.days.contains('1')
    {
        return Err("เลือกวันอย่างน้อยหนึ่งวัน".into());
    }
    if r.printer.trim().is_empty() || !(1..=999).contains(&r.quantity) {
        return Err("ระบุเครื่องพิมพ์และจำนวน 1–999".into());
    }
    let branch = crate::storage::configured_branch(pool).await;
    if branch.is_empty() || branch != r.branch {
        return Err("สาขาของตารางพิมพ์ต้องตรงกับแหล่งข้อมูลที่ตั้งไว้".into());
    }
    if !r.ip.is_empty() {
        r.ip.parse::<std::net::IpAddr>()
            .map_err(|_| "IP เครื่องพิมพ์ไม่ถูกต้อง")?;
        if r.port == 0 {
            return Err("Port ไม่ถูกต้อง".into());
        }
    }
    load_layout(pool, r).await?;
    Ok(())
}
async fn load_layout(
    pool: &SqlitePool,
    r: &PrintRule,
) -> Result<(TCNMLabelSize, Vec<crate::templates::TemplateElement>)> {
    let size = sqlx::query_as::<_, TCNMLabelSize>(
        "SELECT * FROM TCNMLabelSize WHERE FTLbsCode=? AND FTLbsStaUse='1'",
    )
    .bind(&r.size)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or("ขนาดป้ายไม่พบหรือปิดใช้งาน")?;
    let raw:Option<String>=sqlx::query_scalar("SELECT FTLtmLayout FROM TCNMLabelTmpl WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmStaUse='1'")
      .bind(&r.template).bind(&r.size).fetch_optional(pool).await.map_err(|e|e.to_string())?.flatten();
    let raw = raw.ok_or("ไม่พบ Template ที่เลือกสำหรับขนาดนี้ กรุณาบันทึกแบบป้ายก่อน")?;
    let layout: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if layout["designMode"] == "preprint" {
        return Err("แบบกระดาษพิมพ์พื้นหลังสำเร็จให้ใช้ Manual ก่อน Auto ยังไม่รองรับการชดเชยแบบนี้".into());
    }
    let elements: Vec<crate::templates::TemplateElement> =
        serde_json::from_value(layout["elements"].clone()).map_err(|e| e.to_string())?;
    if elements.is_empty() {
        return Err("Template ไม่มีองค์ประกอบ".into());
    }
    // Refuse unsupported headless elements instead of silently losing parts of a label.
    for el in layout["elements"].as_array().ok_or("Template ไม่ถูกต้อง")? {
        let kind = el["type"].as_str().unwrap_or("");
        if !["text", "static", "price", "date", "barcode", "rect", "line"].contains(&kind)
            || el["rotation"].as_f64().unwrap_or(0.0) != 0.0
            || el["opacity"].as_f64().unwrap_or(1.0) != 1.0
        {
            return Err("แบบนี้มีองค์ประกอบที่ Auto headless ยังรองรับไม่ครบ ให้ใช้ Manual หรือเลือกแบบ text/price/barcode/rect/line ที่ไม่หมุน".into());
        }
    }
    Ok((size, elements))
}
#[derive(Deserialize)]
pub struct BulkRules {
    branch: String,
    rules: Vec<PrintRule>,
    preferences: Value,
}
async fn persist_bulk(pool: &SqlitePool, body: BulkRules) -> Result<()> {
    let branch=crate::storage::configured_branch(pool).await;
    if branch.is_empty() || body.branch!=branch || body.preferences["branch"].as_str()!=Some(branch.as_str()) {
        return Err("สาขาของการตั้งค่าต้องตรงกับแหล่งข้อมูล StoreFront".into());
    }
    if body.rules.len()>100 { return Err("บันทึกได้ไม่เกิน 100 ตารางต่อครั้ง".into()); }
    let old=rules(pool).await?;
    let mut ids=std::collections::HashSet::new();
    for r in &body.rules {
        if !ids.insert(r.id.clone()) {return Err("พบรหัสตารางซ้ำในชุดบันทึก".into());}
        if r.branch!=branch {return Err("พบตารางของสาขาอื่นในชุดบันทึก".into());}
        if let Some(previous)=old.iter().find(|x|x.id==r.id) {
            if previous.branch!=branch {return Err("ไม่สามารถแก้ไขตารางของสาขาอื่น".into());}
            // Allow stopping an existing rule even if its template/printer was removed.
            let mut stopped=previous.clone();stopped.enabled=false;
            if !r.enabled && serde_json::to_value(&stopped).unwrap()==serde_json::to_value(r).unwrap() {continue;}
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

pub async fn save_rule(State(st): State<AppState>, Json(r): Json<PrintRule>) -> Response {
    if let Err(e) = validate_rule(&st.pool, &r).await {
        return err(e);
    }
    match sqlx::query("INSERT INTO ABPrintRule(id,config) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET config=excluded.config")
      .bind(&r.id).bind(serde_json::to_string(&r).unwrap()).execute(&st.pool).await {
        Ok(_)=>{
          let key=format!("ruleSince:{}",r.id);
          if setting(&st.pool,&key).await.is_empty(){let _=put(&st.pool,&key,&Local::now().format("%Y-%m-%d").to_string()).await;}
          Json(json!({"ok":true})).into_response()
        },Err(e)=>err(e)
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
    let today = Local::now().format("%Y-%m-%d").to_string();
    let since = setting(pool, &format!("ruleSince:{}", r.id)).await;
    let since = if since.is_empty() {
        today.as_str()
    } else {
        since.as_str()
    };
    let mut items =
        crate::delta_engine::compute_delta_candidates_since(pool, &schedule(r), &today, since)
            .await
            .map_err(|e| e.to_string())?
            .items;
    let (_, els) = load_layout(pool, r).await?;
    for it in &mut items {
        it.lbs_code = r.size.clone();
        it.qty = r.quantity;
        // Share the existing manual data/price builder; only scheduling and transport differ.
        let request = crate::routes::BuildLineBody {
            barcode: it.bar_code.clone(),
            pdt_code: Some(it.pdt_code.clone()),
            qty: Some(r.quantity),
            bch_code: r.branch.clone(),
            pri_type: "1".into(),
            verify: Some(true),
            override_lbs: Some(r.size.clone()),
            effective_date: Some(today.clone()),
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
                    r.quantity,
                    serde_json::to_string(&els).unwrap()
                )
                .as_bytes()
            )
        );
        it.context = Some(ctx);
        let old: Option<String> =
            sqlx::query_scalar("SELECT hash FROM ABPrintWatermark WHERE rule=? AND barcode=?")
                .bind(&r.id)
                .bind(&it.bar_code)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;
        it.skipped_dup = old.as_deref() == Some(&it.price_hash);
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
// Read only: the same event selector as Auto, without a schedule watermark or enqueue.
pub async fn manual_candidates(State(st):State<AppState>,Json(body):Json<ManualCandidates>)->Response {
    let result=async {
        if !["PRICE_CHG","PROMO_START","PROMO_END_REVERT","NEW_PDT"].contains(&body.source.as_str()) {return Err("เหตุการณ์ไม่ถูกต้อง".to_string());}
        chrono::NaiveDate::parse_from_str(&body.date,"%Y-%m-%d").map_err(|_|"วันที่ไม่ถูกต้อง".to_string())?;
        if !(1..=999).contains(&body.quantity){return Err("จำนวนต้องอยู่ระหว่าง 1–999".into());}
        let branch=crate::storage::configured_branch(&st.pool).await;
        if branch.is_empty(){return Err("กรุณาตั้งค่าแหล่งข้อมูล StoreFront".into());}
        let r=PrintRule{id:"manual-preview".into(),name:"ค้นหาสินค้า".into(),enabled:false,branch:branch.clone(),source:body.source.clone(),time:"00:00".into(),days:"1111111".into(),size:body.size.clone(),template:String::new(),printer:String::new(),tray:String::new(),protocol:"WINDOWS".into(),ip:String::new(),port:9100,dpi:203,quantity:body.quantity,catch_up:false};
        let delta=crate::delta_engine::compute_delta_candidates_since(&st.pool,&schedule(&r),&body.date,&body.date).await.map_err(|e|e.to_string())?;
        let mut lines=vec![];
        for item in delta.items {
            let response=crate::routes::post_build_line(State(st.clone()),Json(crate::routes::BuildLineBody{barcode:item.bar_code,pdt_code:Some(item.pdt_code),qty:Some(body.quantity),bch_code:branch.clone(),pri_type:"1".into(),verify:Some(true),override_lbs:Some(body.size.clone()),effective_date:Some(body.date.clone()),price_group:None})).await;
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
    if let Err(e) = validate_rule(&st.pool, &r).await {
        return err(e);
    }
    match candidates(&st.pool, &r).await {
        Ok(items) => Json(
            json!({"items":items,"asOf":now(),"master":crate::storage::status(&st.pool).await}),
        )
        .into_response(),
        Err(e) => err(e),
    }
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
    let id = uid();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let inserted=sqlx::query("INSERT OR IGNORE INTO ABPrintJob(id,dedup,source,name,status,created,config,total) VALUES(?,?,?,?,?,?,?,?)")
      .bind(&id).bind(dedup).bind(source).bind(name).bind(status).bind(now()).bind(config.to_string()).bind(units.len() as i64)
      .execute(&mut *tx).await.map_err(|e|e.to_string())?;
    if inserted.rows_affected() == 0 {
        return Err("รอบนี้มีอยู่ในคิวแล้ว".into());
    }
    for (n, (kind, payload)) in units.into_iter().enumerate() {
        sqlx::query("INSERT INTO ABPrintUnit(job,seq,kind,payload) VALUES(?,?,?,?)")
            .bind(&id)
            .bind(n as i64)
            .bind(kind)
            .bind(payload.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
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
              request["previewOnly"]=json!(true);
              let response=crate::routes::post_print_send(State(st.clone()),Json(request)).await;
              let ok=response.status().is_success();
              let bytes=axum::body::to_bytes(response.into_body(),32*1024*1024).await.map_err(|e|e.to_string())?;
              let v:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
              if !ok || v["success"]!=true {return Err(v.to_string())}
              for _ in 0..qty {units.push(("RAW".into(),json!({"ip":payload["printerIp"],"port":payload["printerPort"],"command":v["command"]})));}
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
        "CANCEL"=>{sqlx::query("UPDATE ABPrintJob SET status='CANCELLED' WHERE id=? AND status NOT IN ('SUBMITTED','EMPTY')").bind(id).execute(&mut *tx).await.map_err(|e|e.to_string())?;},
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
    let page_size = if r.ip.is_empty() {
        (lbs.FNLbsCol.max(1) * lbs.FNLbsRow.max(1)) as usize
    } else {
        1
    };
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for (n, chunk) in pages.chunks(page_size).enumerate() {
        let payload = json!({"route":route,"size":lbs,"elements":elements,"contexts":chunk.iter().map(|(c,_)|c).collect::<Vec<_>>(),
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
                let labels = contexts.iter().map(|c| (c.clone(), 1)).collect::<Vec<_>>();
                let cmd = crate::print_drivers::generate_commands(brand, &labels, &elements, &opts);
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
    let head:Option<(String,String,String,String)>=sqlx::query_as("SELECT id,status,config,source FROM ABPrintJob WHERE status NOT IN ('SUBMITTED','EMPTY','CANCELLED') ORDER BY seq LIMIT 1")
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
async fn scheduler_tick(pool: &SqlitePool) -> Result<()> {
    scheduler_tick_at(pool, Local::now()).await
}
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
        (p,rs)
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
        loop {
            if pool.is_closed() {
                worker.abort();

                break;
            }
            let _ = put(&pool, "heartbeat", &now()).await;
            if let Err(e) = scheduler_tick(&pool).await {
                eprintln!("print schedule: {e}");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
