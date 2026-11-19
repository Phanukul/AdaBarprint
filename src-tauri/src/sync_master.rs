use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterSyncConfig {
    pub api_url: String,
    pub api_key: String,
    pub agn_code: String,
    pub bch_code: String,
    pub retry_count: u32,
    pub retry_wait_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncExecutionProof {
    pub ok: bool,
    pub code: String,
    pub attempts: u32,
    pub synced_at: String,
    pub last_sync_date: String,
    pub items_upserted: usize,
    pub prices_upserted: usize,
    pub promos_upserted: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub code: String,
    pub attempts: u32,
    pub message: String,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} (attempts: {})", self.code, self.message, self.attempts)
    }
}

async fn get_option_val(pool: &SqlitePool, code: &str) -> String {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = ? LIMIT 1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default()
    .trim()
    .to_string()
}

pub async fn load_sync_config(pool: &SqlitePool, retry_count: u32, retry_wait_sec_override: Option<u64>) -> MasterSyncConfig {
    let api_url = {
        let ui = get_option_val(pool, "MasterApiUrl").await;
        if ui.is_empty() {
            std::env::var("API2PSMASTER_URL").unwrap_or_default().trim().to_string()
        } else {
            ui
        }
    };
    let api_key = {
        let ui = get_option_val(pool, "MasterApiKey").await;
        if ui.is_empty() {
            std::env::var("API2PSMASTER_KEY").unwrap_or_default().trim().to_string()
        } else {
            ui
        }
    };
    let agn_code = get_option_val(pool, "MasterAgnCode").await;
    let bch_code = {
        let b = get_option_val(pool, "MasterBchCode").await;
        if b.is_empty() {
            get_option_val(pool, "DefaultBranch").await
        } else {
            b
        }
    };
    let retry_wait_sec = retry_wait_sec_override.unwrap_or_else(|| {
        std::env::var("AUTOPRINT_RETRY_WAIT_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(300)
    });

    MasterSyncConfig {
        api_url: api_url.trim_end_matches('/').to_string(),
        api_key,
        agn_code,
        bch_code,
        retry_count: retry_count.max(1),
        retry_wait_sec,
    }
}

pub async fn get_last_sync_date(pool: &SqlitePool) -> String {
    sqlx::query_scalar::<_, String>(
        "SELECT FDSynLast FROM TCNTSyncLocalState WHERE FTSynTable = 'API2PSMaster_All' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "1900-01-01 00:00:00".to_string())
}

#[cfg(target_os = "windows")]
fn new_curl_cmd() -> tokio::process::Command {
    let mut c = tokio::process::Command::new("curl.exe");
    c.creation_flags(0x08000000); // CREATE_NO_WINDOW
    c
}

#[cfg(not(target_os = "windows"))]
fn new_curl_cmd() -> tokio::process::Command {
    tokio::process::Command::new("curl")
}

/// Perform HTTP POST JSON using system curl + temp config file (keeps API key off process cmdline).
async fn post_json_endpoint(url: &str, api_key: &str, payload: &Value, timeout_sec: u64) -> Result<Value, String> {
    let tmp_dir = std::env::temp_dir().join(format!("ada_sync_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&tmp_dir);
    let body_path = tmp_dir.join("req.json");
    let cfg_path = tmp_dir.join("curl.cfg");

    let body_str = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    std::fs::write(&body_path, &body_str).map_err(|e| e.to_string())?;

    let mut cfg = format!(
        "url = \"{}\"\nrequest = \"POST\"\nsilent\nshow-error\nfail\nmax-time = {}\nheader = \"Content-Type: application/json\"\ndata-binary = \"@{}\"\n",
        url.replace('"', ""),
        timeout_sec.max(5),
        body_path.to_string_lossy().replace('\\', "/")
    );
    if !api_key.is_empty() {
        cfg.push_str(&format!("header = \"X-API-Key: {}\"\n", api_key.replace('"', "")));
    }
    std::fs::write(&cfg_path, cfg).map_err(|e| e.to_string())?;

    let output = new_curl_cmd()
        .arg("-K")
        .arg(&cfg_path)
        .output()
        .await;

    let _ = std::fs::remove_dir_all(&tmp_dir);

    match output {
        Ok(o) if o.status.success() => {
            let txt = String::from_utf8_lossy(&o.stdout);
            if txt.trim().is_empty() {
                return Ok(json!({ "rtCode": "001" }));
            }
            serde_json::from_str::<Value>(&txt)
                .map_err(|e| format!("Invalid JSON from {}: {}", url, e))
        }
        Ok(o) => {
            let err_txt = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Err(format!("HTTP error calling {}: {}", url, if err_txt.is_empty() { o.status.to_string() } else { err_txt }))
        }
        Err(e) => Err(format!("Failed spawning curl for {}: {}", url, e)),
    }
}

/// Helper to read JSON field supporting both Ada POS short prefix (`rtPdtCode`) and Pascal DB column (`FTPdtCode`).
fn j_str(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(val) = v.get(*k) {
            if let Some(s) = val.as_str() {
                return Some(s.to_string());
            }
            if let Some(n) = val.as_i64() {
                return Some(n.to_string());
            }
            if let Some(n) = val.as_f64() {
                return Some(n.to_string());
            }
        }
    }
    None
}

fn j_f64(v: &Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        if let Some(val) = v.get(*k) {
            if let Some(f) = val.as_f64() {
                return Some(f);
            }
            if let Some(s) = val.as_str() {
                if let Ok(parsed) = s.trim().parse::<f64>() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn j_i64(v: &Value, keys: &[&str]) -> Option<i64> {
    for k in keys {
        if let Some(val) = v.get(*k) {
            if let Some(n) = val.as_i64() {
                return Some(n);
            }
            if let Some(s) = val.as_str() {
                if let Ok(parsed) = s.trim().parse::<i64>() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn extract_array<'a>(root: &'a Value, keys: &[&str]) -> Vec<&'a Value> {
    let item_root = root.get("roItem").unwrap_or(root);
    for k in keys {
        if let Some(arr) = item_root.get(*k).and_then(|v| v.as_array()) {
            return arr.iter().collect();
        }
        if let Some(arr) = root.get(*k).and_then(|v| v.as_array()) {
            return arr.iter().collect();
        }
    }
    Vec::new()
}

/// Apply incoming V6 payload JSON directly into `AdaAccD.db` inside a single SQLite transaction.
/// Strictly touches ONLY product/price/promo master tables (never `TCNMLabelSize/Tmpl/Printer`).
pub async fn apply_master_payload_tx(
    tx: &mut Transaction<'_, Sqlite>,
    payload: &Value,
    now_iso: &str,
) -> Result<(usize, usize, usize), sqlx::Error> {
    let mut items_cnt = 0usize;
    let mut prices_cnt = 0usize;
    let mut promos_cnt = 0usize;

    // 1. TCNMPdt
    for row in extract_array(payload, &["raPdt", "aoTCNMPdt", "TCNMPdt", "products"]) {
        let Some(pdt_code) = j_str(row, &["FTPdtCode", "rtPdtCode"]) else { continue };
        sqlx::query(
            r#"
            INSERT INTO TCNMPdt (
                FTPdtCode, FTPdtStaActive, FTPgpChain, FTPtyCode, FTPbnCode, FTPmoCode,
                FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')
            ON CONFLICT(FTPdtCode) DO UPDATE SET
                FTPdtStaActive = COALESCE(excluded.FTPdtStaActive, TCNMPdt.FTPdtStaActive),
                FTPgpChain = COALESCE(excluded.FTPgpChain, TCNMPdt.FTPgpChain),
                FTPtyCode = COALESCE(excluded.FTPtyCode, TCNMPdt.FTPtyCode),
                FTPbnCode = COALESCE(excluded.FTPbnCode, TCNMPdt.FTPbnCode),
                FTPmoCode = COALESCE(excluded.FTPmoCode, TCNMPdt.FTPmoCode),
                FDLastUpdOn = excluded.FDLastUpdOn,
                FTLastUpdBy = 'API2PSMASTER'
            "#,
        )
        .bind(&pdt_code)
        .bind(j_str(row, &["FTPdtStaActive", "rtPdtStaActive"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTPgpChain", "rtPgpChain"]))
        .bind(j_str(row, &["FTPtyCode", "rtPtyCode"]))
        .bind(j_str(row, &["FTPbnCode", "rtPbnCode"]))
        .bind(j_str(row, &["FTPmoCode", "rtPmoCode"]))
        .bind(j_str(row, &["FDCreateOn", "rdCreateOn"]).unwrap_or_else(|| now_iso.to_string()))
        .bind(j_str(row, &["FDLastUpdOn", "rdLastUpdOn"]).unwrap_or_else(|| now_iso.to_string()))
        .execute(&mut **tx)
        .await?;
        items_cnt += 1;
    }

    // 2. TCNMPdt_L
    for row in extract_array(payload, &["raPdtLng", "aoTCNMPdt_L", "TCNMPdt_L", "productNames"]) {
        let Some(pdt_code) = j_str(row, &["FTPdtCode", "rtPdtCode"]) else { continue };
        let lng_id = j_i64(row, &["FNLngID", "rnLngID"]).unwrap_or(1);
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO TCNMPdt_L (FTPdtCode, FNLngID, FTPdtName, FTPdtNameOth, FTPdtNameABB, FTPdtRmk)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&pdt_code)
        .bind(lng_id)
        .bind(j_str(row, &["FTPdtName", "rtPdtName"]).unwrap_or_else(|| pdt_code.clone()))
        .bind(j_str(row, &["FTPdtNameOth", "rtPdtNameOth"]))
        .bind(j_str(row, &["FTPdtNameABB", "rtPdtNameABB"]))
        .bind(j_str(row, &["FTPdtRmk", "rtPdtRmk"]))
        .execute(&mut **tx)
        .await?;
    }

    // 3. TCNMPdtBar
    for row in extract_array(payload, &["raPdtBar", "aoTCNMPdtBar", "TCNMPdtBar", "barcodes"]) {
        let Some(pdt_code) = j_str(row, &["FTPdtCode", "rtPdtCode"]) else { continue };
        let Some(bar_code) = j_str(row, &["FTBarCode", "rtBarCode"]) else { continue };
        let pun_code = j_str(row, &["FTPunCode", "rtPunCode"]).unwrap_or_else(|| "001".to_string());
        let ref_seq = j_i64(row, &["FNBarRefSeq", "rnBarRefSeq"]).unwrap_or(1);
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO TCNMPdtBar (
                FTPdtCode, FTBarCode, FTPunCode, FTBarStaUse, FTBarStaAlwSale,
                FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy, FNBarRefSeq
            ) VALUES (?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER', ?)
            "#,
        )
        .bind(&pdt_code)
        .bind(&bar_code)
        .bind(&pun_code)
        .bind(j_str(row, &["FTBarStaUse", "rtBarStaUse"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTBarStaAlwSale", "rtBarStaAlwSale"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FDCreateOn", "rdCreateOn"]).unwrap_or_else(|| now_iso.to_string()))
        .bind(j_str(row, &["FDLastUpdOn", "rdLastUpdOn"]).unwrap_or_else(|| now_iso.to_string()))
        .bind(ref_seq)
        .execute(&mut **tx)
        .await?;
    }

    // 4. TCNTPdtPrice4PDT
    for row in extract_array(payload, &["raPdtPrice4PDT", "aoTCNTPdtPrice4PDT", "TCNTPdtPrice4PDT", "prices"]) {
        let Some(pdt_code) = j_str(row, &["FTPdtCode", "rtPdtCode"]) else { continue };
        let agn = j_str(row, &["FTAgnCode", "rtAgnCode"]).unwrap_or_default();
        let pun = j_str(row, &["FTPunCode", "rtPunCode"]).unwrap_or_else(|| "001".to_string());
        let dstart = j_str(row, &["FDPghDStart", "rdPghDStart"]).unwrap_or_else(|| now_iso[..10].to_string());
        let tstart = j_str(row, &["FTPghTStart", "rtPghTStart"]).unwrap_or_else(|| "00:00:00".to_string());
        let ppl = j_str(row, &["FTPplCode", "rtPplCode"]).unwrap_or_default();
        let doc_no = j_str(row, &["FTPghDocNo", "rtPghDocNo"]).unwrap_or_else(|| "API-SYNC".to_string());
        let doc_type = j_str(row, &["FTPghDocType", "rtPghDocType"]).unwrap_or_else(|| "1".to_string());
        let ret = j_f64(row, &["FCPgdPriceRet", "rcPgdPriceRet"]).unwrap_or(0.0);

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO TCNTPdtPrice4PDT (
                FTAgnCode, FTPdtCode, FTPunCode, FDPghDStart, FTPghTStart,
                FTPplCode, FDPghDStop, FTPghTStop, FTPghDocNo, FTPghDocType,
                FTPghStaAdj, FCPgdPriceRet, FCPgdPriceWhs, FCPgdPriceNet,
                FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')
            "#,
        )
        .bind(&agn).bind(&pdt_code).bind(&pun).bind(&dstart).bind(&tstart)
        .bind(&ppl)
        .bind(j_str(row, &["FDPghDStop", "rdPghDStop"]))
        .bind(j_str(row, &["FTPghTStop", "rtPghTStop"]))
        .bind(&doc_no).bind(&doc_type)
        .bind(j_str(row, &["FTPghStaAdj", "rtPghStaAdj"]).unwrap_or_else(|| "1".to_string()))
        .bind(ret)
        .bind(j_f64(row, &["FCPgdPriceWhs", "rcPgdPriceWhs"]).unwrap_or(ret))
        .bind(j_f64(row, &["FCPgdPriceNet", "rcPgdPriceNet"]).unwrap_or(ret))
        .bind(j_str(row, &["FDLastUpdOn", "rdLastUpdOn"]).unwrap_or_else(|| now_iso.to_string()))
        .bind(j_str(row, &["FDCreateOn", "rdCreateOn"]).unwrap_or_else(|| now_iso.to_string()))
        .execute(&mut **tx)
        .await?;
        prices_cnt += 1;
    }

    // 5. V6 Promotion tables (TCNTPmtHD, TCNTPmtHD_L, TCNTPmtGrpDT, TCNTPmtCond)
    for row in extract_array(payload, &["raPmtHD", "aoTCNTPmtHD", "TCNTPmtHD", "promotions"]) {
        let Some(doc_no) = j_str(row, &["FTPmhDocNo", "rtPmhDocNo"]) else { continue };
        let agn = j_str(row, &["FTAgnCode", "rtAgnCode"]).unwrap_or_default();
        let ver = j_i64(row, &["FNPmhVersion", "rnPmhVersion"]).unwrap_or(1);
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO TCNTPmtHD (
                FTAgnCode, FTPmhDocNo, FNPmhVersion, FTPmhStaActive, FTPmhName, FTPmhRmk,
                FTPmhStaDoc, FTPmhStaPrcDoc, FDPmhDStart, FDPmhDStop, FNPmhPriority,
                FTPmhRuleJson, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')
            "#,
        )
        .bind(&agn).bind(&doc_no).bind(ver)
        .bind(j_str(row, &["FTPmhStaActive", "rtPmhStaActive"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTPmhName", "rtPmhName"]))
        .bind(j_str(row, &["FTPmhRmk", "rtPmhRmk"]))
        .bind(j_str(row, &["FTPmhStaDoc", "rtPmhStaDoc"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTPmhStaPrcDoc", "rtPmhStaPrcDoc"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FDPmhDStart", "rdPmhDStart"]))
        .bind(j_str(row, &["FDPmhDStop", "rdPmhDStop"]))
        .bind(j_i64(row, &["FNPmhPriority", "rnPmhPriority"]).unwrap_or(100))
        .bind(j_str(row, &["FTPmhRuleJson", "rtPmhRuleJson"]))
        .bind(j_str(row, &["FDLastUpdOn", "rdLastUpdOn"]).unwrap_or_else(|| now_iso.to_string()))
        .bind(j_str(row, &["FDCreateOn", "rdCreateOn"]).unwrap_or_else(|| now_iso.to_string()))
        .execute(&mut **tx)
        .await?;
        promos_cnt += 1;
    }

    for row in extract_array(payload, &["raPmtHDLng", "aoTCNTPmtHD_L", "TCNTPmtHD_L"]) {
        let Some(doc_no) = j_str(row, &["FTPmhDocNo", "rtPmhDocNo"]) else { continue };
        let agn = j_str(row, &["FTAgnCode", "rtAgnCode"]).unwrap_or_default();
        let ver = j_i64(row, &["FNPmhVersion", "rnPmhVersion"]).unwrap_or(1);
        let lng = j_i64(row, &["FNLngID", "rnLngID"]).unwrap_or(1);
        sqlx::query(
            "INSERT OR REPLACE INTO TCNTPmtHD_L (FTAgnCode, FTPmhDocNo, FNPmhVersion, FNLngID, FTPmhName, FTPmhRmk, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy) VALUES (?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')",
        )
        .bind(&agn).bind(&doc_no).bind(ver).bind(lng)
        .bind(j_str(row, &["FTPmhName", "rtPmhName"]))
        .bind(j_str(row, &["FTPmhRmk", "rtPmhRmk"]))
        .bind(now_iso).bind(now_iso)
        .execute(&mut **tx)
        .await?;
    }

    for row in extract_array(payload, &["raPmtGrpDT", "aoTCNTPmtGrpDT", "TCNTPmtGrpDT", "promotionItems"]) {
        let Some(doc_no) = j_str(row, &["FTPmhDocNo", "rtPmhDocNo"]) else { continue };
        let agn = j_str(row, &["FTAgnCode", "rtAgnCode"]).unwrap_or_default();
        let grp = j_str(row, &["FTPmdGrpName", "rtPmdGrpName"]).unwrap_or_else(|| "DEFAULT".to_string());
        let seq = j_i64(row, &["FNPmdSeqNo", "rnPmdSeqNo"]).unwrap_or(1);
        sqlx::query(
            "INSERT OR REPLACE INTO TCNTPmtGrpDT (FTAgnCode, FTPmhDocNo, FTPmdGrpName, FNPmdSeqNo, FTPmdStaType, FTPmdRefType, FTPmdRefCode, FCPmdQty, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')",
        )
        .bind(&agn).bind(&doc_no).bind(&grp).bind(seq)
        .bind(j_str(row, &["FTPmdStaType", "rtPmdStaType"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTPmdRefType", "rtPmdRefType"]).unwrap_or_else(|| "1".to_string()))
        .bind(j_str(row, &["FTPmdRefCode", "rtPmdRefCode"]))
        .bind(j_f64(row, &["FCPmdQty", "rcPmdQty"]).unwrap_or(1.0))
        .bind(now_iso).bind(now_iso)
        .execute(&mut **tx)
        .await?;
    }

    for row in extract_array(payload, &["raPmtCond", "aoTCNTPmtCond", "TCNTPmtCond", "promotionConds"]) {
        let Some(doc_no) = j_str(row, &["FTPmhDocNo", "rtPmhDocNo"]) else { continue };
        let agn = j_str(row, &["FTAgnCode", "rtAgnCode"]).unwrap_or_default();
        let seq = j_i64(row, &["FNPmcSeqNo", "rnPmcSeqNo"]).unwrap_or(1);
        sqlx::query(
            "INSERT OR REPLACE INTO TCNTPmtCond (FTAgnCode, FTPmhDocNo, FNPmcSeqNo, FTPmcCondType, FTPmcRefCode, FCPmcValue, FTPmcJson, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'API2PSMASTER', ?, 'API2PSMASTER')",
        )
        .bind(&agn).bind(&doc_no).bind(seq)
        .bind(j_str(row, &["FTPmcCondType", "rtPmcCondType"]))
        .bind(j_str(row, &["FTPmcRefCode", "rtPmcRefCode"]))
        .bind(j_f64(row, &["FCPmcValue", "rcPmcValue"]))
        .bind(j_str(row, &["FTPmcJson", "rtPmcJson"]))
        .bind(now_iso).bind(now_iso)
        .execute(&mut **tx)
        .await?;
    }

    Ok((items_cnt, prices_cnt, promos_cnt))
}

async fn execute_single_sync_attempt(
    pool: &SqlitePool,
    cfg: &MasterSyncConfig,
) -> Result<SyncExecutionProof, String> {
    if cfg.api_url.is_empty() {
        return Err("ยังไม่ได้ตั้งค่า URL ของ API2PSMaster ในแท็บ Admin".to_string());
    }

    let last_sync = get_last_sync_date(pool).await;
    let req_body = json!({
        "ptAgnCode": cfg.agn_code,
        "ptBchCode": cfg.bch_code,
        "pdDate": last_sync,
    });

    // Step 1: CheckTaskDownload
    let check_url = format!("{}/CheckTaskDownload", cfg.api_url);
    let _check_res = post_json_endpoint(&check_url, &cfg.api_key, &req_body, 20).await?;

    let mut total_items = 0usize;
    let mut total_prices = 0usize;
    let mut total_promos = 0usize;

    // Step 2: Pull endpoints sequentially (Products -> Price4Pdt -> Promotion)
    let endpoints = [
        format!("{}/Product/Download", cfg.api_url),
        format!("{}/Price4Pdt/Download", cfg.api_url),
        format!("{}/Promotion/Download", cfg.api_url),
    ];

    let now_iso = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut payloads = Vec::new();
    for ep in &endpoints {
        match post_json_endpoint(ep, &cfg.api_key, &req_body, 30).await {
            Ok(v) => payloads.push(v),
            Err(e) => {
                // Allow a unified /Sync/LabelMaster endpoint or require core endpoints
                if ep.ends_with("/Product/Download") {
                    let fallback = format!("{}/Product/Item/Download", cfg.api_url);
                    let v = post_json_endpoint(&fallback, &cfg.api_key, &req_body, 30)
                        .await
                        .map_err(|_| e)?;
                    payloads.push(v);
                } else {
                    return Err(e);
                }
            }
        }
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for pl in &payloads {
        let (i, p, pm) = apply_master_payload_tx(&mut tx, pl, &now_iso)
            .await
            .map_err(|e| format!("DB transaction error: {}", e))?;
        total_items += i;
        total_prices += p;
        total_promos += pm;
    }

    sqlx::query(
        r#"
        INSERT INTO TCNTSyncLocalState (FTSynTable, FDSynLast, FTSynSnapshotToken, FTSynLastChecksum, FDLastSyncAt, FTSynLastStatus)
        VALUES ('API2PSMaster_All', ?, ?, ?, ?, 'OK')
        ON CONFLICT(FTSynTable) DO UPDATE SET
            FDSynLast = excluded.FDSynLast,
            FDLastSyncAt = excluded.FDLastSyncAt,
            FTSynLastStatus = 'OK'
        "#,
    )
    .bind(&now_iso)
    .bind("")
    .bind("")
    .bind(&now_iso)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(SyncExecutionProof {
        ok: true,
        code: "OK".to_string(),
        attempts: 1,
        synced_at: now_iso.clone(),
        last_sync_date: now_iso,
        items_upserted: total_items,
        prices_upserted: total_prices,
        promos_upserted: total_promos,
        message: format!(
            "ซิงค์ตรง API2PSMaster สำเร็จ (สินค้า {} | ราคา {} | โปรโมชัน {})",
            total_items, total_prices, total_promos
        ),
    })
}

/// Run direct sync with configurable retry rounds; fails with `E-SYNC-001` when all retries fail.
pub async fn sync_master_with_retry(
    pool: &SqlitePool,
    cfg: &MasterSyncConfig,
) -> Result<SyncExecutionProof, SyncError> {
    if cfg.api_url.trim().is_empty() {
        return Err(SyncError {
            code: "E-SYNC-001".to_string(),
            attempts: 0,
            message: "ยังไม่ได้กรอกที่อยู่ API2PSMaster ในส่วนผู้ดูแลระบบ (IT Admin)".to_string(),
        });
    }

    let mut last_err = String::new();
    let max_tries = cfg.retry_count.max(1);

    for attempt in 1..=max_tries {
        match execute_single_sync_attempt(pool, cfg).await {
            Ok(mut proof) => {
                proof.attempts = attempt;
                return Ok(proof);
            }
            Err(e) => {
                last_err = e;
                if attempt < max_tries && cfg.retry_wait_sec > 0 {
                    tokio::time::sleep(Duration::from_secs(cfg.retry_wait_sec)).await;
                }
            }
        }
    }

    let now_iso = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query(
        r#"
        INSERT INTO TCNTSyncLocalState (FTSynTable, FDSynLast, FDLastSyncAt, FTSynLastStatus)
        VALUES ('API2PSMaster_All', '1900-01-01 00:00:00', ?, 'ERROR')
        ON CONFLICT(FTSynTable) DO UPDATE SET
            FDLastSyncAt = excluded.FDLastSyncAt,
            FTSynLastStatus = 'ERROR'
        "#,
    )
    .bind(&now_iso)
    .execute(pool)
    .await;

    Err(SyncError {
        code: "E-SYNC-001".to_string(),
        attempts: max_tries,
        message: format!("ซิงค์ API2PSMaster ไม่สำเร็จหลังลอง {} รอบ: {}", max_tries, last_err),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_label_schema;
    use axum::{routing::post, Json, Router};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMPdt (FTPdtCode TEXT PRIMARY KEY, FTPdtStaActive TEXT, FTPgpChain TEXT, FTPtyCode TEXT, FTPbnCode TEXT, FTPmoCode TEXT, FDCreateOn TEXT, FTCreateBy TEXT, FDLastUpdOn TEXT, FTLastUpdBy TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPdt_L (FTPdtCode TEXT, FNLngID INTEGER, FTPdtName TEXT, FTPdtNameOth TEXT, FTPdtNameABB TEXT, FTPdtRmk TEXT, PRIMARY KEY (FTPdtCode, FNLngID))",
            "CREATE TABLE IF NOT EXISTS TCNMPdtBar (FTPdtCode TEXT, FTBarCode TEXT, FTPunCode TEXT, FTBarStaUse TEXT, FTBarStaAlwSale TEXT, FDCreateOn TEXT, FTCreateBy TEXT, FDLastUpdOn TEXT, FTLastUpdBy TEXT, FNBarRefSeq INTEGER, PRIMARY KEY (FTPdtCode, FTBarCode, FNBarRefSeq))",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTPdtCode TEXT, FTPunCode TEXT, FDPghDStart TEXT, FTPghTStart TEXT, FTPplCode TEXT, FDPghDStop TEXT, FTPghTStop TEXT, FTPghDocNo TEXT, FTPghDocType TEXT, FTPghStaAdj TEXT, FCPgdPriceRet REAL, FCPgdPriceWhs REAL, FCPgdPriceNet REAL, FDLastUpdOn TEXT, FTLastUpdBy TEXT, FDCreateOn TEXT, FTCreateBy TEXT, PRIMARY KEY (FTAgnCode, FTPdtCode, FTPunCode, FDPghDStart, FTPghTStart, FTPplCode, FTPghDocNo, FTPghDocType))",
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
        pool
    }

    #[tokio::test]
    async fn tc04_and_tc05_direct_sync_and_retry_abort() {
        let pool = setup_test_db().await;

        let mock_app = Router::new()
            .route("/CheckTaskDownload", post(|| async { Json(json!({ "rtCode": "001" })) }))
            .route("/Product/Download", post(|| async {
                Json(json!({
                    "raPdt": [{ "FTPdtCode": "P100", "FTPdtStaActive": "1" }],
                    "raPdtLng": [{ "FTPdtCode": "P100", "FNLngID": 1, "FTPdtName": "น้ำดื่มซิงค์ใหม่" }],
                    "raPdtBar": [{ "FTPdtCode": "P100", "FTBarCode": "8859990001001", "FTPunCode": "001" }]
                }))
            }))
            .route("/Price4Pdt/Download", post(|| async {
                Json(json!({
                    "raPdtPrice4PDT": [{ "FTPdtCode": "P100", "FTPghDocNo": "P-20261119", "FDPghDStart": "2026-11-19", "FCPgdPriceRet": 25.0 }]
                }))
            }))
            .route("/Promotion/Download", post(|| async {
                Json(json!({ "raPmtHD": [] }))
            }));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, mock_app).await;
        });

        let ok_cfg = MasterSyncConfig {
            api_url: format!("http://{}", addr),
            api_key: "secret-key".to_string(),
            agn_code: "001".to_string(),
            bch_code: "00001".to_string(),
            retry_count: 3,
            retry_wait_sec: 0,
        };

        let proof = sync_master_with_retry(&pool, &ok_cfg).await.unwrap();
        assert!(proof.ok);
        assert_eq!(proof.items_upserted, 1);
        assert_eq!(proof.prices_upserted, 1);

        let ret_price: f64 = sqlx::query_scalar("SELECT FCPgdPriceRet FROM TCNTPdtPrice4PDT WHERE FTPdtCode = 'P100'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!((ret_price - 25.0).abs() < 0.01);

        // TC-05: Unreachable port -> Retry 3x -> E-SYNC-001
        let bad_cfg = MasterSyncConfig {
            api_url: "http://127.0.0.1:1".to_string(),
            api_key: "".to_string(),
            agn_code: "".to_string(),
            bch_code: "".to_string(),
            retry_count: 3,
            retry_wait_sec: 0,
        };
        let err = sync_master_with_retry(&pool, &bad_cfg).await.unwrap_err();
        assert_eq!(err.code, "E-SYNC-001");
        assert_eq!(err.attempts, 3);
    }
}
