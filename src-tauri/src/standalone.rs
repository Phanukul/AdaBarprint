//! Standalone V6 master snapshots. Never writes to a POS-owned database or dispatches print jobs.
use crate::routes::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqlitePool};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
#[cfg(target_os = "windows")]
use std::process::Stdio;
#[cfg(target_os = "windows")]
use tokio::io::AsyncWriteExt;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub base_url: String,
    pub header_name: String,
    pub agency: String,
    pub branch: String,
    pub pos: String,
    #[serde(default)]
    pub shop: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "interval")]
    pub interval_minutes: u64,
}
fn interval() -> u64 {
    60
}
#[derive(Deserialize)]
struct Task {
    table: String,
    uri: String,
    mapping: BTreeMap<String, String>,
}
fn tasks() -> Vec<Task> {
    serde_json::from_str(include_str!("../resources/master-tasks.json")).unwrap()
}
pub fn own_path() -> PathBuf {
    crate::storage::print_db_path()
        .parent()
        .unwrap()
        .join("AdaAccD.db")
}
pub async fn ensure(pool: &SqlitePool) -> Result<()> {
    sqlx::query("CREATE TABLE IF NOT EXISTS ABSource(id INTEGER PRIMARY KEY CHECK(id=1), mode TEXT NOT NULL DEFAULT 'POS_SHARED', config TEXT NOT NULL DEFAULT '{}', status TEXT NOT NULL DEFAULT '', last_success TEXT NOT NULL DEFAULT '', lease_until INTEGER NOT NULL DEFAULT 0)").execute(pool).await.map_err(|e|e.to_string())?;
    Ok(())
}
pub async fn mode(pool: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT mode FROM main.ABSource WHERE id=1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "POS_SHARED".into())
}
pub async fn status(pool: &SqlitePool) -> Value {
    let row: Option<(String, String, String)> =
        sqlx::query_as("SELECT config,status,last_success FROM main.ABSource WHERE id=1")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    let (raw, state, last) = row.unwrap_or_default();
    let mut cfg: Value = serde_json::from_str(&raw).unwrap_or(json!({}));
    let key = cfg["apiKey"]
        .as_str()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    cfg["apiKey"] = json!("");
    json!({"config":cfg,"hasApiKey":key,"status":state,"lastSuccess":last})
}
pub fn validate_config(c: &SyncConfig) -> Result<()> {
    if c.api_key.contains(['\r', '\n']) {
        return Err("API Key ไม่ถูกต้อง".into());
    }
    if !(c.base_url.starts_with("https://") || c.base_url.starts_with("http://"))
        || c.base_url.contains(['\n', '\r', '"', '\\', '@', '?', '#'])
    {
        return Err("ระบุ URL ของ API2PSMaster ให้ถูกต้อง".into());
    }
    if c.branch.trim().is_empty()
        || c.header_name.is_empty()
        || !c
            .header_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("ระบุสาขาและชื่อ Header ให้ถูกต้อง".into());
    }
    Ok(())
}
// Windows DPAPI binds the stored credential to the Windows account running the service.
// Non-Windows (Android/iOS/Unix) stores an app-sandboxed AES-128-CBC secret key under `data_root()`.
fn load_or_create_local_key_at(root: &Path) -> Result<[u8; 16]> {
    let key_path = root.join(".secret.key");
    if key_path.is_file() {
        let raw = std::fs::read(&key_path).map_err(|_| "ไม่สามารถอ่านคีย์เข้ารหัสภายในแอป".to_string())?;
        if raw.len() == 16 {
            let mut key = [0u8; 16];
            key.copy_from_slice(&raw);
            return Ok(key);
        }
    }
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let key = *uuid::Uuid::new_v4().as_bytes();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&key_path)
            .map_err(|e| e.to_string())?;
        f.write_all(&key).map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&key_path, &key).map_err(|e| e.to_string())?;
    }
    Ok(key)
}

fn protect_local_at(root: &Path, secret: &str, decrypt: bool) -> Result<String> {
    use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    let key = load_or_create_local_key_at(root)?;
    if decrypt {
        let encoded = secret
            .strip_prefix("local:")
            .ok_or_else(|| "ข้อมูลยืนยันตัวตนไม่รองรับบนระบบปฏิบัติการนี้ กรุณาระบุใหม่".to_string())?;
        let raw = B64
            .decode(encoded.trim())
            .map_err(|_| "อ่านข้อมูลยืนยันตัวตนไม่สำเร็จ".to_string())?;
        if raw.len() < 17 {
            return Err("อ่านข้อมูลยืนยันตัวตนไม่สำเร็จ".into());
        }
        let (iv, ct) = raw.split_at(16);
        let dec = Aes128CbcDec::new_from_slices(&key, iv)
            .map_err(|_| "ไม่สามารถถอดรหัสข้อมูลยืนยันตัวตน".to_string())?;
        let plain = dec
            .decrypt_padded_vec_mut::<Pkcs7>(ct)
            .map_err(|_| "อ่านข้อมูลยืนยันตัวตนไม่ได้ กรุณาระบุใหม่".to_string())?;
        String::from_utf8(plain).map_err(|_| "ข้อมูลยืนยันตัวตนไม่ถูกต้อง".to_string())
    } else {
        let iv = *uuid::Uuid::new_v4().as_bytes();
        let enc = Aes128CbcEnc::new_from_slices(&key, &iv)
            .map_err(|_| "ไม่สามารถเข้ารหัสข้อมูลยืนยันตัวตน".to_string())?;
        let ct = enc.encrypt_padded_vec_mut::<Pkcs7>(secret.as_bytes());
        let mut combined = Vec::with_capacity(16 + ct.len());
        combined.extend_from_slice(&iv);
        combined.extend_from_slice(&ct);
        Ok(format!("local:{}", B64.encode(combined)))
    }
}

pub async fn protect(secret: &str, decrypt: bool) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if decrypt && secret.starts_with("local:") {
            return protect_local_at(&crate::storage::data_root(), secret, true);
        }
        let script = if decrypt {
            "[void][Reflection.Assembly]::LoadWithPartialName('System.Security'); $s=[Console]::In.ReadToEnd(); $b=[Security.Cryptography.ProtectedData]::Unprotect([Convert]::FromBase64String($s),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser); [Console]::OutputEncoding=[Text.UTF8Encoding]::new(); [Console]::Write([Text.Encoding]::UTF8.GetString($b))"
        } else {
            "[void][Reflection.Assembly]::LoadWithPartialName('System.Security'); $s=[Console]::In.ReadToEnd(); $b=[Security.Cryptography.ProtectedData]::Protect([Text.Encoding]::UTF8.GetBytes($s),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser); [Console]::Write([Convert]::ToBase64String($b))"
        };
        let mut cmd = tokio::process::Command::new("powershell.exe");
        cmd.creation_flags(0x08000000);
        let mut child = cmd
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "ไม่สามารถใช้ Windows Credential Protection".to_string())?;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(secret.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("อ่านข้อมูลยืนยันตัวตนไม่ได้ กรุณาระบุ API Key ใหม่ด้วยบัญชี Windows ที่ใช้รันบริการ".into());
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().into())
    }
    #[cfg(not(target_os = "windows"))]
    {
        protect_local_at(&crate::storage::data_root(), secret, decrypt)
    }
}
/// A standalone destination must be a local, print-owned master; never adopt a POS database.
pub async fn prepare_destination(path: &Path) -> Result<()> {
    if !path.is_absolute() || path.to_string_lossy().starts_with("\\\\") || path.file_name().and_then(|v|v.to_str()).map(|v|!v.eq_ignore_ascii_case("AdaAccD.db")).unwrap_or(true) {
        return Err("ระบุพาธเต็มของ AdaAccD.db บนเครื่องนี้".into());
    }
    if path.exists() {
        let mut c=sqlx::SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path).read_only(true)).await.map_err(|_|"เปิดไฟล์ปลายทางไม่ได้")?;
        let owned:i64=sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ABMasterIdentity'").fetch_one(&mut c).await.map_err(|e|e.to_string())?;
        c.close().await.map_err(|e|e.to_string())?;
        if owned==0 {return Err("ไฟล์นี้ไม่ใช่ฐานดาวน์โหลดของ Barprint กรุณาเลือกโฟลเดอร์ใหม่ หรือใช้โหมดอ่านข้อมูลจาก StoreFront".into());}
    } else {
        std::fs::create_dir_all(path.parent().ok_or("พาธไม่ถูกต้อง")?).map_err(|_|"สร้างโฟลเดอร์ปลายทางไม่ได้")?;
    }
    create_master(path).await
}

pub async fn create_master(path: &Path) -> Result<()> {
    let mut c = sqlx::SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    )
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query("PRAGMA encoding='UTF-16le'")
        .execute(&mut c)
        .await
        .map_err(|e| e.to_string())?;
    for statement in include_str!("../resources/master-schema.sql").split(";\n") {
        if !statement.trim().is_empty() {
            sqlx::query(statement)
                .execute(&mut c)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS TSysSyncTaskDownload(FTSynTable TEXT PRIMARY KEY,FDSynLastSuccess TEXT)").execute(&mut c).await.map_err(|e|e.to_string())?;
    sqlx::query("CREATE TABLE IF NOT EXISTS ABMasterIdentity(id INTEGER PRIMARY KEY,agency TEXT,branch TEXT,pos TEXT)").execute(&mut c).await.map_err(|e|e.to_string())?;
    c.close().await.map_err(|e| e.to_string())
}
fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn base(c: &SyncConfig) -> String {
    let s = c.base_url.trim_end_matches('/');
    if s.to_ascii_lowercase().ends_with("/v6") {
        s.into()
    } else {
        format!("{s}/V6")
    }
}
async fn http(c: &SyncConfig, url: &str, body: Option<Value>) -> Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "ไม่สามารถสร้างตัวเชื่อมต่อ API".to_string())?;
    let mut req = if let Some(b) = body {
        client.post(url).json(&b)
    } else {
        client.get(url)
    };
    req = req
        .header(&c.header_name, &c.api_key)
        .header("Content-Type", "application/json");
    let resp = req
        .send()
        .await
        .map_err(|_| "เชื่อม API ไม่สำเร็จ ตรวจ URL เครือข่าย และสิทธิ์ API Key".to_string())?;
    if !resp.status().is_success() {
        return Err("เชื่อม API ไม่สำเร็จ ตรวจ URL เครือข่าย และสิทธิ์ API Key".into());
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|_| "เชื่อม API ไม่สำเร็จ ตรวจ URL เครือข่าย และสิทธิ์ API Key".to_string())?;
    let v: Value =
        serde_json::from_slice(&bytes).map_err(|_| "API ไม่ได้ตอบกลับเป็น JSON".to_string())?;
    if !["001", "1"].contains(&v["rtCode"].as_str().unwrap_or("")) {
        return Err(format!("API ไม่ยืนยันข้อมูลครบ (รหัส {})", v["rtCode"]));
    }
    Ok(v)
}
fn payload(v: Value) -> Result<Value> {
    let p = v.get("roItem").cloned().unwrap_or(v);
    if !p.is_object() {
        return Err("API ไม่มีชุดข้อมูล roItem".into());
    }
    Ok(p)
}
fn product_body(c: &SyncConfig) -> Value {
    let mut body = json!({"ptAgnCode":c.agency,"ptBchCode":c.branch,"ptShpCode":c.shop,"pdDate":"1999-01-01 00:00:00","pnRow":0,"pnStartRow":0});
    if !c.pos.trim().is_empty() {
        body["ptPosCode"] = json!(c.pos.trim());
    }
    body
}
async fn download(c: &SyncConfig, t: &Task) -> Result<Value> {
    let mut uri = t.uri.clone();
    for (k, v) in [
        ("{pdDate}", "1999-01-01 00:00:00"),
        ("{ptAgnCode}", &c.agency),
        ("{ptBchCode}", &c.branch),
        ("{ptPosCode}", &c.pos),
    ] {
        uri = uri.replace(k, &enc(v));
    }
    let pos_qs = if c.pos.trim().is_empty() {
        String::new()
    } else {
        format!("&ptPosCode={}", enc(c.pos.trim()))
    };
    let url = format!(
        "{}{}{}ptBchCode={}{}",
        base(c),
        uri,
        if uri.contains('?') { "&" } else { "?" },
        enc(&c.branch),
        pos_qs
    );
    if t.table == "TCNMPdt" {
        let count = http(
            c,
            &format!("{}/Product/Check", base(c)),
            Some(product_body(c)),
        )
        .await?;
        let total = count["rnCount"].as_u64().ok_or("ไม่พบจำนวนสินค้า")?;
        let size = count["rnPageSize"]
            .as_u64()
            .filter(|v| *v > 0 && *v <= 10000)
            .ok_or("ขนาดหน้าสินค้าไม่ถูกต้อง")?;
        let token = count["rtSnapshotToken"]
            .as_str()
            .filter(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or("ไม่พบ snapshot token")?;
        let mut all = json!({});
        for key in t.mapping.keys() {
            all[key] = json!([]);
        }
        let mut start = 0;
        while start < total {
            let n = size.min(total - start);
            let mut body = product_body(c);
            body["pnRow"] = json!(n);
            body["pnStartRow"] = json!(start);
            body["ptSnapshotToken"] = json!(token);
            let page = http(c, &url, Some(body)).await?;
            if page["rtSnapshotToken"] != token
                || page["rnStartRow"] != start
                || page["rnRowCount"] != n
            {
                return Err("หน้าสินค้าไม่ตรงกับ snapshot".into());
            }
            let p = payload(page)?;
            for key in t.mapping.keys() {
                let rows = p[key].as_array().ok_or(format!("ข้อมูลสินค้าไม่ครบ: {key}"))?;
                all[key].as_array_mut().unwrap().extend(rows.clone());
            }
            start += n;
        }
        if all["raPdt"].as_array().unwrap().len() as u64 != total {
            return Err("จำนวนสินค้าที่ดาวน์โหลดไม่ครบ".into());
        }
        let mut seen = std::collections::HashSet::new();
        for r in all["raPdt"].as_array().unwrap() {
            let code = r
                .get("rtPdtCode")
                .or(r.get("FTPdtCode"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if code.is_empty() || !seen.insert(code) {
                return Err("รหัสสินค้าใน snapshot ว่างหรือซ้ำ".into());
            }
        }
        return Ok(all);
    }
    if t.table == "TCNTPdtPrice4PDT" {
        let mut req_body = json!({"ptAgnCode":c.agency,"ptBchCode":c.branch,"ptContractVersion":"3","ptSyncMode":"FULL"});
        if !c.pos.trim().is_empty() {
            req_body["ptPosCode"] = json!(c.pos.trim());
        }
        let mut p=payload(http(c,&url,Some(req_body)).await?)?;
        if p.get("raPdtPrice4PDT").is_none() {
            p["raPdtPrice4PDT"] = p["raItems"].clone();
        }
        let meta = &p["roPricingProjectionMeta"];
        if meta["complete"] != true
            || meta["contractVersion"] != "3"
            || meta["syncMode"] != "FULL"
            || meta["reasonCode"] != "PRICING_PROJECTION_READY"
        {
            return Err("ข้อมูลราคา V3 ยังไม่ครบหรือไม่ได้รับสิทธิ์สาขานี้".into());
        }
        for k in [
            "raCstPriList",
            "raPdtPriList",
            "raPdtPriListLng",
            "raChannel",
            "raChannelLng",
            "raPdtPrice4PDT",
            "raPdtPrice4PdtStep",
            "raPdtPrice4PdtScope",
            "raSuggestedPrice4Pdt",
            "raPdtPriCalHD",
            "raPdtPriCalDT",
            "raPdtPriCalHDBch",
            "raPdtPriCalApplicability",
            "raPdtProfRef",
            "raPdtSrnCtrlProf",
            "raPdtSrnCtrlProfLng",
            "raSerialControlStatus",
            "raProjectionInvalidation",
        ] {
            let rows = p[k].as_array().ok_or(format!("ข้อมูลราคาขาด {k}"))?;
            let ck = if k == "raPdtPrice4PDT" { "raItems" } else { k };
            if meta["rowCounts"][ck].as_u64() != Some(rows.len() as u64) {
                return Err("จำนวนรายการราคาไม่ตรง".into());
            }
        }
        return Ok(p);
    }
    payload(http(c, &url, None).await?)
}
fn column(k: &str) -> String {
    if k.len() > 2 {
        let prefix = match &k[..2] {
            "rt" => "FT",
            "rn" => "FN",
            "rd" => "FD",
            "rc" => "FC",
            "rl" => "FL",
            "rm" => "FM",
            _ => return k.into(),
        };
        format!("{prefix}{}", &k[2..])
    } else {
        k.into()
    }
}
fn quote(k: &str) -> String {
    format!("\"{}\"", k.replace('"', "\"\""))
}
pub async fn apply_snapshots(
    path: &Path,
    cfg: &SyncConfig,
    bundles: &[(String, Value)],
) -> Result<()> {
    let mut c = sqlx::SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .busy_timeout(Duration::from_secs(10)),
    )
    .await
    .map_err(|e| e.to_string())?;
    let result:Result<()>=async {
    let mut tx = c.begin().await.map_err(|e| e.to_string())?;
    let mut effective_agency = cfg.agency.trim().to_string();
    let mut cleared = std::collections::HashSet::new();
    for t in tasks() {
        let data = &bundles
            .iter()
            .find(|(name, _)| name == &t.table)
            .ok_or(format!("ชุดข้อมูลขาด {}", t.table))?
            .1;
        for (key, table) in &t.mapping {
            let rows = data[key]
                .as_array()
                .ok_or(format!("API ไม่ส่ง {key} จึงไม่ล้างข้อมูลเดิม"))?;
            let cols: Vec<(i64, String, String, i64, Option<String>, i64)> =
                sqlx::query_as(&format!("PRAGMA table_info({})", quote(table)))
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            if cols.is_empty() {
                return Err(format!("ฐานข้อมูลยังไม่รองรับ {table}"));
            }
            if cleared.insert(table.clone()) {
                sqlx::query(&format!("DELETE FROM {}", quote(table)))
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            for row in rows {
                let mut mapped: BTreeMap<String, Value> = row
                    .as_object()
                    .ok_or("แถวข้อมูลไม่ถูกต้อง")?
                    .iter()
                    .map(|(k, v)| (column(k), v.clone()))
                    .collect();
                if t.table == "TCNTPdtPrice4PDT"
                    && [
                        "raCstPriList",
                        "raPdtPriList",
                        "raPdtPriListLng",
                        "raPdtPrice4PDT",
                        "raPdtPrice4PdtStep",
                        "raPdtPrice4PdtScope",
                    ]
                    .contains(&key.as_str())
                {
                    if let Some(agency) = mapped.get("FTAgnCode").and_then(Value::as_str) {
                        if effective_agency.is_empty() { effective_agency = agency.trim().to_string(); }
                        if !agency.is_empty() && agency != effective_agency {
                            return Err(format!("ข้อมูลราคา API ข้าม Agency: {table}"));
                        }
                    }
                }
                if table == "TCNTPdtPrice4PDT" {
                    if let Some(v) = mapped.get("FCPgdPrice").cloned() {
                        mapped.entry("FCPgdPriceRet".into()).or_insert(v);
                    }
                    for (from, to) in [
                        ("FDPghEffectiveStart", "FDPghDStart"),
                        ("FDPghEffectiveStop", "FDPghDStop"),
                    ] {
                        if let Some(v) = mapped.get(from).cloned() {
                            mapped.entry(to.into()).or_insert(v);
                        }
                    }
                }
                let fields: Vec<_> = cols
                    .iter()
                    .filter_map(|col| mapped.get(&col.1).map(|v| (&col.1, v)))
                    .collect();
                if fields.is_empty() {
                    return Err(format!("ไม่พบคอลัมน์ที่รองรับใน {table}"));
                }
                let sql = format!(
                    "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
                    quote(table),
                    fields
                        .iter()
                        .map(|(k, _)| quote(k))
                        .collect::<Vec<_>>()
                        .join(","),
                    vec!["?"; fields.len()].join(",")
                );
                let mut q = sqlx::query(&sql);
                for (_, v) in &fields {
                    q = match v {
                        Value::Null => q.bind(Option::<String>::None),
                        Value::Bool(b) => q.bind(i64::from(*b)),
                        Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                q.bind(i)
                            } else {
                                q.bind(n.as_f64().unwrap_or_default())
                            }
                        }
                        Value::String(s) => q.bind(s.clone()),
                        _ => q.bind(v.to_string()),
                    };
                }
                q.execute(&mut *tx)
                    .await
                    .map_err(|e| format!("เขียน {table} ไม่สำเร็จ: {e}"))?;
            }
        }
    }
    let branch: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM TCNMBranch WHERE FTBchCode=?")
        .bind(&cfg.branch)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    if branch == 0 {
        return Err("API ไม่ส่งสาขาที่ระบุมาให้".into());
    }
    for t in tasks() {
        sqlx::query(
            "INSERT OR REPLACE INTO TSysSyncTaskDownload(FTSynTable,FDSynLastSuccess) VALUES(?,?)",
        )
        .bind(t.table)
        .bind(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    sqlx::query("INSERT OR REPLACE INTO ABMasterIdentity VALUES(1,?,?,?)")
        .bind(&effective_agency)
        .bind(&cfg.branch)
        .bind(&cfg.pos)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())
    }.await;
    c.close().await.map_err(|e|e.to_string())?;
    result
}
pub async fn sync_now(pool: &SqlitePool) -> Result<()> {
    let source_cfg: Option<(String, String)> = sqlx::query_as("SELECT mode, config FROM ABSource WHERE id=1")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    let has_api = source_cfg
        .as_ref()
        .and_then(|(m, raw)| {
            if m != "STANDALONE" {
                return None;
            }
            serde_json::from_str::<SyncConfig>(raw)
                .ok()
                .filter(|c| !c.base_url.trim().is_empty())
        })
        .is_some();
    if !has_api {
        return Err("ยังไม่ได้ตั้งค่าการเชื่อมต่อ API2PSMaster สำหรับดาวน์โหลด (เครื่องนี้ใช้งานฐานข้อมูลโดยตรง หรือนำเข้า Master)".into());
    }
    let locked=sqlx::query("UPDATE ABSource SET lease_until=unixepoch()+1800,status='กำลังดาวน์โหลด' WHERE id=1 AND mode='STANDALONE' AND lease_until<unixepoch()").execute(pool).await.map_err(|e|e.to_string())?.rows_affected();
    if locked == 0 {
        return Err("มีการดาวน์โหลดข้อมูลกำลังทำงานอยู่ กรุณารอสักครู่".into());
    }
    let result = tokio::time::timeout(Duration::from_secs(1500), async {
        let raw: String = sqlx::query_scalar("SELECT config FROM ABSource WHERE id=1")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        let mut cfg: SyncConfig = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        validate_config(&cfg)?;
        cfg.api_key = protect(&cfg.api_key, true).await?;
        let mut bundles = vec![];
        for task in tasks() {
            let value = download(&cfg, &task)
                .await
                .map_err(|e| format!("{}: {e}", task.table))?;
            bundles.push((task.table, value));
        }
        let path:String=sqlx::query_scalar("SELECT path FROM ABStorage WHERE id=1").fetch_one(pool).await.map_err(|e|e.to_string())?;
        prepare_destination(Path::new(&path)).await?;
        apply_snapshots(Path::new(&path), &cfg, &bundles).await
    })
    .await
    .unwrap_or_else(|_| Err("ดาวน์โหลดเกินเวลาที่กำหนด กรุณาลองใหม่".into()));
    let message = match &result {
        Ok(()) => "ดาวน์โหลดครบแล้ว".into(),
        Err(e) => e.clone(),
    };
    sqlx::query("UPDATE ABSource SET lease_until=0,status=?,last_success=CASE WHEN ? THEN datetime('now') ELSE last_success END WHERE id=1").bind(message).bind(result.is_ok()).execute(pool).await.map_err(|e|e.to_string())?;
    result
}
#[derive(Deserialize)]
pub struct Trigger {
    #[serde(default)]
    pub pin: String,
}
pub async fn trigger(
    State(st): State<AppState>,
    headers: axum::http::HeaderMap,
    body: Option<Json<Trigger>>,
) -> Response {
    let pin = body.map(|b| b.pin.clone()).unwrap_or_default();
    let authorized = if !pin.trim().is_empty() {
        crate::storage::valid_pin(&st.pool, &pin).await
    } else {
        crate::auth::has_active_session(&st.pool, &headers).await
    };
    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"PIN ผู้ดูแลระบบไม่ถูกต้อง หรือกรุณาเข้าสู่ระบบก่อนดาวน์โหลด"})),
        )
            .into_response();
    }
    match sync_now(&st.pool).await {
        Ok(()) => Json(json!({"ok":true,"message":"ดาวน์โหลดครบแล้ว"})).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error":e}))).into_response(),
    }
}
pub fn spawn(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut failures: u32 = 0;
        loop {
            if pool.is_closed() { break; }
            tokio::time::sleep(Duration::from_secs(30)).await;
            let due:bool=sqlx::query_scalar("SELECT COALESCE(mode='STANDALONE' AND lease_until<unixepoch() AND (last_success='' OR unixepoch()-unixepoch(last_success)>=COALESCE(json_extract(config,'$.intervalMinutes'),60)*60),0) FROM ABSource WHERE id=1").fetch_optional(&pool).await.ok().flatten().unwrap_or(false);
            if due {
                if sync_now(&pool).await.is_ok() { failures=0; }
                else { failures=failures.saturating_add(1); }
                let delay=if failures==0 {60} else {(60_u64 * (1_u64 << failures.min(5))).min(1800)};
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> SyncConfig {
        SyncConfig {
            base_url: "http://127.0.0.1".into(),
            header_name: "X-API-Key".into(),
            branch: "B1".into(),
            interval_minutes: 15,
            ..Default::default()
        }
    }
    #[tokio::test]
    async fn atomic_snapshot_and_missing_arrays() {
        let root =
            std::env::temp_dir().join(format!("barprint-sync-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("AdaAccD.db");
        prepare_destination(&path).await.unwrap();
        let pos_dir=root.join("pos");std::fs::create_dir_all(&pos_dir).unwrap();let pos_path=pos_dir.join("AdaAccD.db");
        let mut pos=sqlx::SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&pos_path).create_if_missing(true)).await.unwrap();
        sqlx::query("CREATE TABLE PosOnly(id INTEGER)").execute(&mut pos).await.unwrap();pos.close().await.unwrap();
        assert!(prepare_destination(&pos_path).await.is_err(),"must reject existing non-owned POS database");
        assert!(prepare_destination(Path::new("relative/AdaAccD.db")).await.is_err());
        let mut bundles: Vec<(String, Value)> = tasks()
            .into_iter()
            .map(|t| {
                let mut v = json!({});
                for k in t.mapping.keys() {
                    v[k] = json!([]);
                }
                (t.table, v)
            })
            .collect();
        bundles[0].1["raBch"] = json!([{"rtBchCode":"B1","rtPplCode":"","rtWahCode":""}]);
        apply_snapshots(&path, &config(), &bundles).await.unwrap();
        let mut c =
            sqlx::SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path))
                .await
                .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM TSysSyncTaskDownload")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            8
        );
        bundles[0].1["raBch"] = json!([{"rtBchCode":"BAD","rtPplCode":"","rtWahCode":""}]);
        bundles[1].1.as_object_mut().unwrap().remove("raUser");
        assert!(apply_snapshots(&path, &config(), &bundles).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT FTBchCode FROM TCNMBranch")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            "B1"
        );
        c.close().await.unwrap();
        let pool = crate::storage::connect_path(&root.join("AdaBarprint.db"))
            .await
            .unwrap();
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'B1',1440)")
            .bind(path.to_str().unwrap())
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO ABSource(id,mode) VALUES(1,'STANDALONE')")
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM master.TCNMBranch")
                .fetch_one(&pool)
                .await
                .is_err(),
            "standalone must not expose old master before first completed sync"
        );
        sqlx::query("UPDATE ABSource SET last_success=datetime('now')")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM master.TCNMBranch")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
        assert!(sqlx::query("DELETE FROM master.TCNMBranch")
            .execute(&pool)
            .await
            .is_err());
        pool.close().await;
        // sqlx can still be joining a discarded SQLite worker after pool.close,
        // especially after the intentional readonly-write failure above.
        let resolved=root.canonicalize().unwrap();
        let temporary=std::env::temp_dir().canonicalize().unwrap();
        assert!(resolved.starts_with(&temporary) && resolved!=temporary);
        let mut cleanup=None;
        for _ in 0..20 {
            match std::fs::remove_dir_all(&resolved) {
                Ok(())=>{cleanup=None;break;},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{cleanup=None;break;},
                Err(e)=>cleanup=Some(e),
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if let Some(error)=cleanup {panic!("Temporary SQLite cleanup failed: {error}");}
    }
    #[cfg(windows)]
    #[tokio::test]
    async fn credentials_roundtrip_without_plaintext_storage() {
        let encrypted = protect("test-only-secret", false).await.unwrap();
        assert_ne!(encrypted, "test-only-secret");
        assert_eq!(protect(&encrypted, true).await.unwrap(), "test-only-secret");
    }
    #[test]
    fn local_credentials_roundtrip_without_plaintext_storage() {
        let root = std::env::temp_dir().join(format!("barprint-secret-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let encrypted = protect_local_at(&root, "test-mobile-secret", false).unwrap();
        assert!(encrypted.starts_with("local:"));
        assert_ne!(encrypted, "test-mobile-secret");
        assert_eq!(protect_local_at(&root, &encrypted, true).unwrap(), "test-mobile-secret");
        let _ = std::fs::remove_dir_all(&root);
    }
    #[tokio::test]
    #[ignore]
    async fn live_storefront_download_without_printing() {
        let cfg: SyncConfig =
            serde_json::from_slice(&std::fs::read("../.env.standalone-test.json").unwrap())
                .unwrap();
        validate_config(&cfg).unwrap();
        let root =
            std::env::temp_dir().join(format!("barprint-live-sync-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("AdaAccD.db");
        create_master(&path).await.unwrap();
        let mut bundles = vec![];
        for task in tasks() {
            eprintln!("Checking {}", task.table);
            let value = download(&cfg, &task)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", task.table));
            bundles.push((task.table, value));
        }
        apply_snapshots(&path, &cfg, &bundles).await.unwrap();
        eprintln!(
            "Live snapshot validated at {}; no router, scheduler, queue or printer was started.",
            path.display()
        );
    }
}
