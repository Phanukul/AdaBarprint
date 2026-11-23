//! Print-owned SQLite main database with a read-only StoreFront attachment.
//! Each pooled connection checks the persisted configuration before reuse so the
//! desktop and standalone agent cannot silently keep different master paths.
use std::{path::{Path, PathBuf}, sync::OnceLock, time::Duration};
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode}, Connection, SqliteConnection, SqlitePool};
use crate::routes::AppState;

static APP_DATA_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Sets the application data root directory from the runtime (used on mobile platforms).
pub fn set_app_data_root(path: PathBuf) {
    let _ = APP_DATA_ROOT.set(path);
}

pub fn data_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("ADABARPRINT_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = APP_DATA_ROOT.get() {
        return dir.clone();
    }
    #[cfg(target_os = "windows")]
    {
        #[cfg(not(test))]
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                // The installed worker lives in <application>/agent, not a
                // separate installation. Share Data, Logs and the queue lock.
                if exe.file_stem().is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case("autoprint_agent"))
                    && dir.file_name().is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case("agent")) {
                    if let Some(root) = dir.parent() { return root.to_path_buf(); }
                }
                return dir.to_path_buf();
            }
        }
        PathBuf::from(std::env::var_os("ProgramData").expect("Windows ProgramData is required"))
            .join("AdaPos6.0")
            .join("AdaBarprint")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::temp_dir().join("AdaBarprint")
    }
}

pub fn is_dir_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(".adabarprint-write-probe-{}", uuid::Uuid::new_v4()));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(file) => {
            drop(file);
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}
pub fn print_db_path() -> PathBuf { std::env::var_os("ADABARPRINT_DB_DIR").map(PathBuf::from).unwrap_or_else(||data_root().join("Data")).join("AdaBarprint.db") }
pub fn default_master() -> PathBuf {
    PathBuf::from(std::env::var_os("ProgramData").unwrap_or_else(|| "C:\\ProgramData".into()))
        .join("AdaPos6.0").join("AdaStoreFront").join("Data").join("AdaAccD.db")
}
pub fn default_local_master() -> PathBuf {
    data_root().join("Data").join("AdaAccD.db")
}

pub fn readonly_uri(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let raw = raw.strip_prefix("//?/").unwrap_or(&raw);
    let mut result = String::from("file:");
    for b in raw.bytes() {
        if b.is_ascii_alphanumeric() || b"/:._-".contains(&b) { result.push(b as char); }
        else { result.push_str(&format!("%{b:02X}")); }
    }
    result.push_str("?mode=ro");
    result
}

pub async fn connect() -> Result<SqlitePool, sqlx::Error> {
    for sub in ["Data", "Exports", "Logs", "Backups"] {
        std::fs::create_dir_all(data_root().join(sub)).map_err(sqlx::Error::Io)?;
    }
    std::fs::create_dir_all(print_db_path().parent().unwrap()).map_err(sqlx::Error::Io)?;
    backup_before_upgrade(&print_db_path()).await?;
    connect_path(&print_db_path()).await
}

pub const PRINT_SCHEMA_REVISION: i64 = 20261121;

/// Snapshot only the print-owned database; VACUUM INTO includes committed WAL
/// data and does not require copying a live SQLite file or touching StoreFront.
pub async fn backup_before_upgrade(path: &Path) -> Result<(), sqlx::Error> {
    if !path.is_file() || std::fs::metadata(path).map_err(sqlx::Error::Io)?.len()==0 {return Ok(());}
    let options=SqliteConnectOptions::new().filename(path).create_if_missing(false).busy_timeout(Duration::from_secs(15));
    let mut connection=SqliteConnection::connect_with(&options).await?;
    let revision:i64=sqlx::query_scalar("PRAGMA user_version").fetch_one(&mut connection).await?;
    if revision>=PRINT_SCHEMA_REVISION {connection.close().await?;return Ok(());}
    let tables:i64=sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table'").fetch_one(&mut connection).await?;
    if tables>0 {
        let directory=path.parent().ok_or_else(||sqlx::Error::Protocol("ไม่พบโฟลเดอร์ฐานข้อมูลการพิมพ์".into()))?.join("Backups");
        std::fs::create_dir_all(&directory).map_err(sqlx::Error::Io)?;
        let destination=directory.join(format!("AdaBarprint-before-{}-{}.db",PRINT_SCHEMA_REVISION,uuid::Uuid::new_v4()));
        sqlx::query("VACUUM main INTO ?").bind(destination.to_string_lossy().as_ref()).execute(&mut connection).await?;
        let mut snapshot=SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&destination).read_only(true)).await?;
        let integrity:String=sqlx::query_scalar("PRAGMA integrity_check").fetch_one(&mut snapshot).await?;
        snapshot.close().await?;
        if integrity!="ok" {return Err(sqlx::Error::Protocol("สำรองฐานข้อมูลก่อนอัปเกรดไม่สำเร็จ กรุณาตรวจสอบพื้นที่จัดเก็บ".into()));}
    }
    connection.close().await?;
    Ok(())
}

pub async fn connect_path(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal).busy_timeout(Duration::from_secs(5));
    // Bootstrap only print-owned tables, before attaching any Master.
    let mut bootstrap = SqliteConnection::connect_with(&options).await?;
    // StoreFront's installed SQLite master uses UTF-16LE. SQLite ATTACH requires
    // all files in a connection to use the same encoding. Set before first DDL.
    sqlx::query("PRAGMA encoding='UTF-16le'").execute(&mut bootstrap).await?;
    let encoding:String=sqlx::query_scalar("PRAGMA encoding").fetch_one(&mut bootstrap).await?;
    if !encoding.eq_ignore_ascii_case("UTF-16le") {
        return Err(sqlx::Error::Protocol("AdaBarprint.db ต้องย้ายเป็น UTF-16LE ก่อนใช้ Master ของ StoreFront".into()));
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS ABStorage(id INTEGER PRIMARY KEY CHECK(id=1), path TEXT NOT NULL, branch TEXT NOT NULL, max_age_minutes INTEGER NOT NULL DEFAULT 1440)").execute(&mut bootstrap).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS ABSource(id INTEGER PRIMARY KEY CHECK(id=1), mode TEXT NOT NULL DEFAULT 'POS_SHARED', config TEXT NOT NULL DEFAULT '{}', status TEXT NOT NULL DEFAULT '', last_success TEXT NOT NULL DEFAULT '', lease_until INTEGER NOT NULL DEFAULT 0)").execute(&mut bootstrap).await?;
    bootstrap.close().await?;
    SqlitePoolOptions::new().max_connections(8)
        .after_connect(|c, _| Box::pin(async move { refresh_attachment(c).await }))
        .before_acquire(|c, _| Box::pin(async move { refresh_attachment(c).await?; Ok(true) }))
        .connect_with(options).await
}

async fn refresh_attachment(c: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    let cfg: Option<(String, String)> = sqlx::query_as("SELECT path,branch FROM main.ABStorage WHERE id=1").fetch_optional(&mut *c).await?;
    let attached: Vec<(i64,String,String)> = sqlx::query_as("PRAGMA database_list").fetch_all(&mut *c).await?;
    let current = attached.iter().find(|r| r.1 == "master").map(|r| r.2.as_str());
    let standalone_ready:Option<(String,String)>=sqlx::query_as("SELECT mode,last_success FROM main.ABSource WHERE id=1").fetch_optional(&mut *c).await?;
    let ready=standalone_ready.map(|(m,last)|m!="STANDALONE"||!last.is_empty()).unwrap_or(true);
    let wanted = cfg.as_ref().map(|r| r.0.as_str()).filter(|p| ready && Path::new(p).is_file());
    let same = match (current,wanted) {
        (Some(a),Some(b)) => std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok(),
        (None,None) => true, _ => false,
    };
    if !same {
        // Views are connection-local adapters, never tables copied into main.
        let views:Vec<String>=sqlx::query_scalar("SELECT name FROM sqlite_temp_master WHERE type='view'").fetch_all(&mut *c).await?;
        for view in views { sqlx::query(&format!("DROP VIEW temp.\"{}\"",view.replace('"',"\"\""))).execute(&mut *c).await?; }
        if current.is_some() { sqlx::query("DETACH DATABASE master").execute(&mut *c).await?; }
        if let Some(path) = wanted {
            // No immutable=1: StoreFront may concurrently update its WAL.
            if let Err(e) = sqlx::query("ATTACH DATABASE ? AS master").bind(readonly_uri(Path::new(path))).execute(&mut *c).await {
                eprintln!("StoreFront master unavailable: {e}");
            }
        }
    }
    let present: i64=sqlx::query_scalar("SELECT COUNT(*) FROM pragma_database_list WHERE name='master'").fetch_one(&mut *c).await?;
    let adapted:i64=sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_temp_master WHERE type='view' AND name='TCNTPdtPrice4PDT'").fetch_one(&mut *c).await?;
    if present>0 && adapted==0 {
        // Scope branch-bearing business rows. Branch selection is stored in main,
        // so existing pooled views also see changes without rebuilding the pool.
        let branch_has_agency: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('TCNMBranch','master') WHERE name='FTAgnCode'").fetch_one(&mut *c).await?;
        for table in ["TCNTPdtPrice4PDT","TCNTPdtAdjPriHD","TCNTPdtAdjPriDT","TCNTPdtPmtHD","TCNTPdtPmtDT","TCNTPdtPmtHD_L","TCNTPdtPmtCG","TCNTPmtHD"] {
            let columns:Vec<(i64,String,String,i64,Option<String>,i64)>=sqlx::query_as(&format!("PRAGMA master.table_info(\"{table}\")")).fetch_all(&mut *c).await?;
            if columns.is_empty() {continue;}
            let has=|name:&str|columns.iter().any(|col|col.1==name);
            let projection=if table=="TCNTPdtPrice4PDT" {
                columns.iter().map(|col| match col.1.as_str() {
                    "FDPghDStart" if has("FDPghEffectiveStart") => "COALESCE(NULLIF(FDPghEffectiveStart,''),FDPghDStart) AS FDPghDStart".into(),
                    "FDPghDStop" if has("FDPghEffectiveStop") => "COALESCE(NULLIF(FDPghEffectiveStop,''),FDPghDStop) AS FDPghDStop".into(),
                    "FCPgdPriceRet" if has("FCPgdPrice") => "COALESCE(FCPgdPrice,FCPgdPriceRet) AS FCPgdPriceRet".into(),
                    _=>format!("\"{}\"",col.1.replace('"',"\"\"")),
                }).collect::<Vec<_>>().join(",")
            } else {"*".into()};
            let filter=if table=="TCNTPmtHD" && has("FTAgnCode") && branch_has_agency>0 {
                " WHERE COALESCE(FTAgnCode,'')='' OR FTAgnCode=COALESCE((SELECT NULLIF(TRIM(b.FTAgnCode),'') FROM master.TCNMBranch b JOIN main.ABStorage s ON s.branch=b.FTBchCode WHERE s.id=1),FTAgnCode)"
            } else if has("FTBchCode") {" WHERE COALESCE(FTBchCode,'')='' OR FTBchCode=(SELECT branch FROM main.ABStorage WHERE id=1)"} else {""};
            sqlx::query(&format!("CREATE TEMP VIEW IF NOT EXISTS \"{table}\" AS SELECT {projection} FROM master.\"{table}\"{filter}")).execute(&mut *c).await?;
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Config { #[serde(default,skip_serializing)] pub operator_password: String, #[serde(default)] pub path: String, #[serde(default="unified_mode")] pub mode:String, #[serde(default)] pub sync:Option<crate::standalone::SyncConfig>, #[serde(default)] pub branch: String,
    #[serde(default="default_age")] pub max_age_minutes: i64, #[serde(default,skip_serializing)] pub pin: String, #[serde(default,skip_serializing)] pub new_pin: String }
fn unified_mode()->String{"UNIFIED".into()}
fn default_age() -> i64 { 1440 }

pub async fn validate(path: &str) -> Result<Value, String> {
    let p = Path::new(path.trim());
    if !p.is_absolute() || !p.is_file() { return Err("เลือกไฟล์ AdaAccD.db ที่มีอยู่จริงด้วยพาธเต็ม".into()); }
    if std::fs::canonicalize(p).ok() == std::fs::canonicalize(print_db_path()).ok() { return Err("ฐานข้อมูล Master ต้องไม่ใช่ฐานข้อมูล AdaBarprint".into()); }
    let options = SqliteConnectOptions::new().filename(p).read_only(true).create_if_missing(false)
        .busy_timeout(Duration::from_secs(3));
    let mut c = SqliteConnection::connect_with(&options).await.map_err(|_| "เปิดฐานข้อมูล StoreFront แบบอ่านอย่างเดียวไม่ได้")?;
    let encoding:String=sqlx::query_scalar("PRAGMA encoding").fetch_one(&mut c).await.map_err(|e|e.to_string())?;
    if !encoding.eq_ignore_ascii_case("UTF-16le") { return Err("ฐาน StoreFront ต้องใช้ UTF-16LE ตามรูปแบบ POS รุ่นนี้ กรุณาตรวจว่าเลือกไฟล์ใช้งานจริง".into()); }
    for query in [
        "SELECT FTPdtCode FROM TCNMPdt LIMIT 0",
        "SELECT FTBarCode,FTPdtCode FROM TCNMPdtBar LIMIT 0",
        "SELECT FTPdtCode,FCPgdPriceRet,FDPghDStart,FDPghDStop FROM TCNTPdtPrice4PDT LIMIT 0",
        "SELECT FTUsrCode,FTUsrLoginPwd FROM TCNMUsrLogin LIMIT 0",
        "SELECT FTUsrCode FROM TCNMUser LIMIT 0",
        "SELECT FTBchCode FROM TCNMBranch LIMIT 0",
    ] { sqlx::query(query).execute(&mut c).await.map_err(|_| "โครงสร้าง Master ไม่รองรับ หรือยังไม่ได้เตรียมข้อมูลจาก StoreFront")?; }
    let branches:Vec<(String,Option<String>)> = sqlx::query_as("SELECT b.FTBchCode,(SELECT l.FTBchName FROM TCNMBranch_L l WHERE l.FTBchCode=b.FTBchCode ORDER BY CASE WHEN l.FNLngID=1 THEN 0 ELSE 1 END LIMIT 1) FROM TCNMBranch b ORDER BY b.FTBchCode").fetch_all(&mut c).await.map_err(|e|e.to_string())?;
    let products:i64=sqlx::query_scalar("SELECT COUNT(*) FROM TCNMPdt").fetch_one(&mut c).await.map_err(|e|e.to_string())?;
    let users:i64=sqlx::query_scalar("SELECT COUNT(*) FROM TCNMUser").fetch_one(&mut c).await.map_err(|e|e.to_string())?;
    c.close().await.map_err(|e|e.to_string())?;
    Ok(json!({"ok":true,"productCount":products,"userCount":users,"branches":branches.into_iter().map(|(code,name)|json!({"code":code,"name":name})).collect::<Vec<_>>(),"readOnly":true}))
}

pub async fn configured_branch(pool: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT branch FROM main.ABStorage WHERE id=1").fetch_optional(pool).await.ok().flatten().unwrap_or_default()
}

pub async fn master_revision(pool: &SqlitePool) -> String {
    let rows:Vec<(String,String)> = sqlx::query_as("SELECT FTSynTable,FDSynLastSuccess FROM master.TSysSyncTaskDownload ORDER BY FTSynTable").fetch_all(pool).await.unwrap_or_default();
    serde_json::to_string(&rows).unwrap_or_default()
}

pub async fn auto_ready(pool: &SqlitePool) -> bool {
    let age:i64 = sqlx::query_scalar("SELECT max_age_minutes FROM main.ABStorage WHERE id=1").fetch_optional(pool).await.ok().flatten().unwrap_or(1440);
    let rows:Vec<(String,String)> = sqlx::query_as("SELECT FTSynTable,FDSynLastSuccess FROM master.TSysSyncTaskDownload").fetch_all(pool).await.unwrap_or_default();
    // Missing checkpoint is unknown, not successful. StoreFront owns these records.
    ["TCNMPdt", "TCNTPdtPrice4PDT"].iter().all(|table| rows.iter().any(|(t,s)| {
        if t != table { return false; }
        let seconds = chrono::DateTime::parse_from_rfc3339(s).ok().map(|d|(chrono::Utc::now()-d.with_timezone(&chrono::Utc)).num_seconds())
            .or_else(|| chrono::NaiveDateTime::parse_from_str(s,"%Y-%m-%d %H:%M:%S").ok().map(|d|(chrono::Local::now().naive_local()-d).num_seconds()));
        seconds.map(|n|n>=0 && n<=age*60).unwrap_or(false)
    })) && !configured_branch(pool).await.is_empty()
}

pub async fn promo_ready(pool: &SqlitePool) -> bool {
    let age:i64=sqlx::query_scalar("SELECT max_age_minutes FROM main.ABStorage WHERE id=1").fetch_optional(pool).await.ok().flatten().unwrap_or(1440);
    let rows:Vec<String>=sqlx::query_scalar("SELECT FDSynLastSuccess FROM master.TSysSyncTaskDownload WHERE FTSynTable IN ('TCNTPmtHD','TCNTPdtPmtHD')").fetch_all(pool).await.unwrap_or_default();
    rows.iter().any(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|d|(chrono::Utc::now()-d.with_timezone(&chrono::Utc)).num_seconds())
        .or_else(||chrono::NaiveDateTime::parse_from_str(s,"%Y-%m-%d %H:%M:%S").ok().map(|d|(chrono::Local::now().naive_local()-d).num_seconds()))
        .map(|n|n>=0 && n<=age*60).unwrap_or(false))
}

/// Native picker is only invoked by an explicit click in the desktop UI.
#[tauri::command]
pub async fn choose_master_database() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os="windows")]
        {
            use std::os::windows::process::CommandExt;
            let output=std::process::Command::new("powershell.exe")
                .creation_flags(0x08000000)
                .args(["-NoProfile","-STA","-Command", "$OutputEncoding=[Console]::OutputEncoding=[Text.UTF8Encoding]::new(); Add-Type -AssemblyName System.Windows.Forms; $picker=New-Object System.Windows.Forms.OpenFileDialog; $picker.Title='StoreFront Master Database'; $picker.Filter='SQLite database (*.db)|*.db'; $picker.InitialDirectory=Join-Path $env:ProgramData 'AdaPos6.0\\AdaStoreFront\\Data'; if($picker.ShowDialog() -eq 'OK'){[Console]::Write($picker.FileName)}"])
                .output().map_err(|e|e.to_string())?;
            if !output.status.success() {return Err("ไม่สามารถเปิดหน้าต่างเลือกไฟล์ กรุณาวางพาธเต็มแทน".into());}
            let path=String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if path.is_empty(){None}else{Some(path)})
        }
        #[cfg(not(target_os="windows"))]
        { Ok(None) }
    }).await.map_err(|e|e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn upgrade_backup_includes_committed_wal_and_preserves_source() {
        let directory=std::env::temp_dir().join(format!("barprint-upgrade-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path=directory.join("AdaBarprint.db");
        let mut connection=SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path).create_if_missing(true).journal_mode(SqliteJournalMode::Wal)).await.unwrap();
        sqlx::raw_sql("CREATE TABLE PrintSettings(value TEXT); INSERT INTO PrintSettings VALUES('saved')").execute(&mut connection).await.unwrap();
        backup_before_upgrade(&path).await.unwrap();
        let snapshots:Vec<_>=std::fs::read_dir(directory.join("Backups")).unwrap().map(|e|e.unwrap().path()).collect();
        assert_eq!(snapshots.len(),1);
        let mut snapshot=SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&snapshots[0]).read_only(true)).await.unwrap();
        let value:String=sqlx::query_scalar("SELECT value FROM PrintSettings").fetch_one(&mut snapshot).await.unwrap();
        assert_eq!(value,"saved");
        let count:i64=sqlx::query_scalar("SELECT COUNT(*) FROM PrintSettings").fetch_one(&mut connection).await.unwrap();
        assert_eq!(count,1);
        snapshot.close().await.unwrap();connection.close().await.unwrap();
    }
    async fn fixture() -> (PathBuf,PathBuf,SqlitePool) {
        let root=std::env::temp_dir().join(format!("barprint-storage-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let master=root.join("Master # ข้อมูล.db");
        let options=SqliteConnectOptions::new().filename(&master).create_if_missing(true).journal_mode(SqliteJournalMode::Wal);
        let mut c=SqliteConnection::connect_with(&options).await.unwrap();
        sqlx::query("PRAGMA encoding='UTF-16le'").execute(&mut c).await.unwrap();
        sqlx::raw_sql("CREATE TABLE TCNMPdt(FTPdtCode TEXT); INSERT INTO TCNMPdt VALUES('P1'); CREATE TABLE TCNMPdtBar(FTBarCode TEXT,FTPdtCode TEXT); CREATE TABLE TCNTPdtPrice4PDT(FTPdtCode TEXT,FCPgdPriceRet REAL,FDPghDStart TEXT,FDPghDStop TEXT); CREATE TABLE TCNMUsrLogin(FTUsrCode TEXT,FTUsrLoginPwd TEXT); CREATE TABLE TCNMUser(FTUsrCode TEXT); CREATE TABLE TCNMBranch(FTBchCode TEXT); INSERT INTO TCNMBranch VALUES('00001'); CREATE TABLE TCNMBranch_L(FTBchCode TEXT,FNLngID INTEGER,FTBchName TEXT);").execute(&mut c).await.unwrap();
        c.close().await.unwrap();
        let pool=connect_path(&root.join("AdaBarprint.db")).await.unwrap();
        (root,master,pool)
    }
    #[tokio::test]
    async fn master_is_readonly_and_print_tables_remain_local() {
        let (_root,master,pool)=fixture().await;
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'00001',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
        crate::db::ensure_label_schema(&pool).await.unwrap();
        crate::print_center::ensure_schema(&pool).await.unwrap();
        let product:String=sqlx::query_scalar("SELECT FTPdtCode FROM TCNMPdt").fetch_one(&pool).await.unwrap();
        assert_eq!(product,"P1");
        assert!(sqlx::query("DELETE FROM TCNMPdt").execute(&pool).await.is_err());
        assert!(sqlx::query("CREATE TABLE master.Unwanted(x)").execute(&pool).await.is_err());
        sqlx::query("INSERT INTO ABPrintSetting VALUES('test','ok')").execute(&pool).await.unwrap();
        let copied:i64=sqlx::query_scalar("SELECT COUNT(*) FROM main.sqlite_master WHERE name='TCNMPdt'").fetch_one(&pool).await.unwrap();
        let leaked:i64=sqlx::query_scalar("SELECT COUNT(*) FROM master.sqlite_master WHERE name LIKE 'ABPrint%' OR name LIKE 'TCNMLabel%'").fetch_one(&pool).await.unwrap();
        assert_eq!((copied,leaked),(0,0));
        assert!(!auto_ready(&pool).await,"No checkpoint must not authorize auto");
        pool.close().await;
    }
    #[tokio::test]
    async fn missing_master_does_not_create_file_or_block_local_settings() {
        let (root,_master,pool)=fixture().await;
        let missing=root.join("missing.db");
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'00001',1440)").bind(missing.to_str().unwrap()).execute(&pool).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABStorage").fetch_one(&pool).await.unwrap(),1);
        assert!(!missing.exists());
        assert!(validate(missing.to_str().unwrap()).await.is_err());
        pool.close().await;
    }
    #[tokio::test]
    async fn master_path_with_unicode_and_uri_characters_validates() {
        let (_root,master,pool)=fixture().await;
        assert!(validate(master.to_str().unwrap()).await.is_ok());
        pool.close().await;
    }
    #[tokio::test]
    async fn print_connections_observe_storefront_updates_without_copying_master() {
        let (_root,master,pool)=fixture().await;
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'00001',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMPdt").fetch_one(&pool).await.unwrap(),1);
        let mut writer=SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&master)).await.unwrap();
        sqlx::query("INSERT INTO TCNMPdt VALUES('P2')").execute(&mut writer).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMPdt").fetch_one(&pool).await.unwrap(),2);
        writer.close().await.unwrap();pool.close().await;
    }
    #[tokio::test]
    async fn branch_and_effective_dates_are_adapted_without_changing_master() {
        let (_root,master,pool)=fixture().await;
        let mut writer=SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&master)).await.unwrap();
        sqlx::raw_sql("ALTER TABLE TCNTPdtPrice4PDT ADD COLUMN FTBchCode TEXT; ALTER TABLE TCNTPdtPrice4PDT ADD COLUMN FCPgdPrice REAL; ALTER TABLE TCNTPdtPrice4PDT ADD COLUMN FDPghEffectiveStart TEXT; ALTER TABLE TCNTPdtPrice4PDT ADD COLUMN FDPghEffectiveStop TEXT; INSERT INTO TCNTPdtPrice4PDT VALUES('P1',10,'2020-01-01','2099-01-01','00001',12,'2026-11-20','2026-11-21'); INSERT INTO TCNTPdtPrice4PDT VALUES('P1',20,'2020-01-01','2099-01-01','00002',22,'2026-11-20','2026-11-21');").execute(&mut writer).await.unwrap();
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'00001',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
        let rows:Vec<(f64,String)>=sqlx::query_as("SELECT FCPgdPriceRet,FDPghDStart FROM TCNTPdtPrice4PDT").fetch_all(&pool).await.unwrap();
        assert_eq!(rows,vec![(12.0,"2026-11-20".into())]);
        assert!(sqlx::query("UPDATE TCNTPdtPrice4PDT SET FCPgdPriceRet=99").execute(&pool).await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNTPdtPrice4PDT").fetch_one(&mut writer).await.unwrap(),2);
        pool.close().await;writer.close().await.unwrap();
    }

    #[tokio::test]
    async fn changing_master_requires_pin_and_an_empty_active_queue() {
        let (_root,master,pool)=fixture().await;
        crate::db::ensure_label_schema(&pool).await.unwrap();
        crate::print_center::ensure_schema(&pool).await.unwrap();
        crate::auth::ensure_auth_schema(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNSSysOption(FTSopCode,FTSopApp,FTSopName,FTSopValue) VALUES('AutoPrintAdminPin','PrintBar','PIN','test-pin')").execute(&pool).await.unwrap();
        let config=|pin:&str|Config{operator_password:String::new(),mode:"POS_SHARED".into(),sync:None,path:master.to_string_lossy().into_owned(),branch:"00001".into(),max_age_minutes:1440,pin:pin.into(),new_pin:String::new()};
        let state=||State(AppState{pool:pool.clone()});
        assert_eq!(save_config(state(),Json(config("wrong"))).await.status(),StatusCode::UNAUTHORIZED);
        sqlx::query("INSERT INTO ABPrintJob(id,source,name,status,created,config) VALUES('hold','MANUAL','hold','QUEUED','now','{}')").execute(&pool).await.unwrap();
        assert_eq!(save_config(state(),Json(config("test-pin"))).await.status(),StatusCode::BAD_REQUEST);
        assert!(configured_branch(&pool).await.is_empty());
        sqlx::query("UPDATE ABPrintJob SET status='CANCELLED'").execute(&pool).await.unwrap();
        assert!(save_config(state(),Json(config("test-pin"))).await.status().is_success());
        assert_eq!(configured_branch(&pool).await,"00001");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM master.TCNMPdt").fetch_one(&pool).await.unwrap(),1);
        pool.close().await;
    }

    #[tokio::test]
    async fn unified_save_handles_local_and_pos_db_without_mode_dropdown() {
        let (root, master, pool) = fixture().await;
        crate::db::ensure_label_schema(&pool).await.unwrap();
        crate::print_center::ensure_schema(&pool).await.unwrap();
        crate::auth::ensure_auth_schema(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNSSysOption(FTSopCode,FTSopApp,FTSopName,FTSopValue) VALUES('AutoPrintAdminPin','PrintBar','PIN','1234')").execute(&pool).await.unwrap();

        // First-time Admin Gate verification must require setting a new PIN when current PIN is 1234
        let err_missing = verify_and_update_admin_pin(&pool, "1234", "").await.unwrap_err();
        assert_eq!(err_missing.0, StatusCode::BAD_REQUEST);
        assert!(err_missing.2);
        assert!(!verify_and_update_admin_pin(&pool, "1234", "246810").await.unwrap());
        assert!(!admin_pin_change_required(&pool).await);

        let state = || State(AppState { pool: pool.clone() });
        // 1) Unified save pointing to new local DB without API URL -> initializes local owned AdaAccD.db automatically
        let local_db = root.join("LocalData").join("AdaAccD.db");
        let res_local = save_config(state(), Json(Config {
            operator_password:"sample-password".into(),
            mode: "UNIFIED".into(),
            sync: None,
            path: local_db.to_string_lossy().into_owned(),
            branch: "00002".into(),
            max_age_minutes: 1440,
            pin: "246810".into(),
            new_pin: String::new(),
        })).await;
        assert!(res_local.status().is_success());
        assert!(local_db.is_file());
        let login=crate::auth::post_auth_login(state(),Json(crate::auth::LoginRequest {
            user_id:Some("LOCAL".into()),credential_type:Some("1".into()),credential:Some("sample-password".into())
        })).await;
        assert!(login.status().is_success());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMUsrLogin WHERE FTUsrCode='LOCAL' AND FTUsrLogType='1'").fetch_one(&pool).await.unwrap(),1);
        assert_eq!(crate::standalone::mode(&pool).await, "STANDALONE");
        assert_eq!(configured_branch(&pool).await, "00002");

        // 2) Unified save pointing to existing StoreFront POS DB without API URL -> read-only POS attachment automatically
        let res_pos = save_config(state(), Json(Config {
            operator_password:String::new(),
            mode: "UNIFIED".into(),
            sync: None,
            path: master.to_string_lossy().into_owned(),
            branch: String::new(), // single branch auto-detected
            max_age_minutes: 1440,
            pin: "246810".into(),
            new_pin: String::new(),
        })).await;
        assert!(res_pos.status().is_success());
        assert_eq!(crate::standalone::mode(&pool).await, "POS_SHARED");
        assert_eq!(configured_branch(&pool).await, "00001");
        pool.close().await;
    }

    #[tokio::test]
    async fn manual_event_lookup_does_not_create_jobs_or_rules() {
        let (root,_,pool)=fixture().await;
        let master=root.join("event-master.db");
        crate::standalone::create_master(&master).await.unwrap();
        crate::db::ensure_label_schema(&pool).await.unwrap();
        crate::print_center::ensure_schema(&pool).await.unwrap();
        sqlx::query("INSERT INTO ABStorage VALUES(1,?,'00001',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
        for source in ["PRICE_CHG","PROMO_START","PROMO_END_REVERT","NEW_PDT"] {
            let body=crate::print_center::ManualCandidates{source:source.into(),date:"2026-11-20".into(),size:"SH-100x38".into(),quantity:1};
            let response=crate::print_center::manual_candidates(State(AppState{pool:pool.clone()}),Json(body)).await;
            let status=response.status();let bytes=axum::body::to_bytes(response.into_body(),1024*1024).await.unwrap();
            assert!(status.is_success(),"{source}: {}",String::from_utf8_lossy(&bytes));
            assert!(serde_json::from_slice::<Value>(&bytes).unwrap()["lines"].is_array());
        }
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintJob").fetch_one(&pool).await.unwrap(),0);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintRule").fetch_one(&pool).await.unwrap(),0);
        pool.close().await;
    }

    /// Explicit opt-in integration check. Reads the installed POS master only;
    /// writes a disposable copy of print data and never starts a scheduler.
    #[tokio::test]
    #[ignore]
    async fn installed_storefront_manual_read_endpoints() {
        use tower::ServiceExt;
        let master=std::env::var_os("ADABARPRINT_VERIFY_MASTER").map(PathBuf::from).unwrap_or_else(default_master);
        assert!(validate(master.to_str().unwrap()).await.is_ok());
        let root=std::env::temp_dir().join(format!("barprint-readcheck-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::copy(print_db_path(),root.join("AdaBarprint.db")).unwrap();
        let pool=connect_path(&root.join("AdaBarprint.db")).await.unwrap();
        crate::db::ensure_label_schema(&pool).await.unwrap();
        crate::print_center::ensure_schema(&pool).await.unwrap();
        crate::auth::ensure_auth_schema(&pool).await.unwrap();
        sqlx::query("INSERT OR REPLACE INTO ABStorage VALUES(1,?,'00005',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
        let router=crate::build_router(pool.clone());
        for path in ["/api/seed","/api/branches","/api/config/all","/api/templates","/api/products?limit=5","/api/promotions","/api/auth/login-options","/api/auth/users","/api/print-center"] {
            let response=router.clone().oneshot(axum::http::Request::builder().uri(path).body(axum::body::Body::empty()).unwrap()).await.unwrap();
            let code=response.status();
            let bytes=axum::body::to_bytes(response.into_body(),16*1024*1024).await.unwrap();
            assert!(code.is_success(),"{path}: {code}: {}",String::from_utf8_lossy(&bytes));
            let parsed:Value=serde_json::from_slice(&bytes).unwrap();
            assert!(parsed.get("error").is_none(),"{path}: {}",parsed["error"]);
        }
        let barcode:Option<String>=sqlx::query_scalar("SELECT b.FTBarCode FROM TCNMPdtBar b JOIN TCNMPdt p ON p.FTPdtCode=b.FTPdtCode WHERE COALESCE(b.FTBarStaUse,'1')='1' LIMIT 1").fetch_optional(&pool).await.unwrap();
        let Some(barcode)=barcode else {
            eprintln!("Master has no products: verified empty-state read endpoints only; manual/auto data preparation requires StoreFront Sync.");
            pool.close().await;return;
        };
        let request=json!({"barcode":barcode,"qty":1,"bchCode":"00005","priType":"1","verify":true,"overrideLbs":"ADA-100x38"});
        let response=router.clone().oneshot(axum::http::Request::builder().method("POST").uri("/api/print/build-line").header("content-type","application/json").body(axum::body::Body::from(request.to_string())).unwrap()).await.unwrap();
        let code=response.status();let bytes=axum::body::to_bytes(response.into_body(),1024*1024).await.unwrap();
        assert!(code.is_success(),"manual preparation: {}",String::from_utf8_lossy(&bytes));
        let raw:String=sqlx::query_scalar("SELECT config FROM ABPrintRule ORDER BY id LIMIT 1").fetch_one(&pool).await.unwrap();
        let mut rule:Value=serde_json::from_str(&raw).unwrap();rule["branch"]=json!("00005");
        let response=router.clone().oneshot(axum::http::Request::builder().method("POST").uri("/api/print-center/preview").header("content-type","application/json").body(axum::body::Body::from(rule.to_string())).unwrap()).await.unwrap();
        let code=response.status();let bytes=axum::body::to_bytes(response.into_body(),16*1024*1024).await.unwrap();
        assert!(code.is_success(),"auto preview: {}",String::from_utf8_lossy(&bytes));
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintJob").fetch_one(&pool).await.unwrap(),0,"read/preview tests must not enqueue jobs");
        pool.close().await;
    }
}

pub async fn status(pool: &SqlitePool) -> Value {
    let cfg:Option<(String,String,i64)> = sqlx::query_as("SELECT path,branch,max_age_minutes FROM main.ABStorage WHERE id=1").fetch_optional(pool).await.ok().flatten();
    let default_path = default_local_master();
    let (path,branch,age)=cfg.clone().unwrap_or((default_path.to_string_lossy().into_owned(),String::new(),1440));
    let checked=validate(&path).await;
    let attached=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM master.TCNMBranch").fetch_one(pool).await.is_ok();
    let connected=cfg.is_some() && checked.is_ok() && attached;
    let root_dir = data_root();
    let sync_status = crate::standalone::status(pool).await;
    let api_configured = sync_status["config"]["baseUrl"].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false);
    json!({"sourceVersion":4,"platform":std::env::consts::OS,"supportsPosShared":cfg!(target_os="windows"),"supportsWindowsPrint":cfg!(target_os="windows"),"adminPinChangeRequired":admin_pin_change_required(pool).await,"mode":crate::standalone::mode(pool).await,"apiConfigured":api_configured,"defaultPath":default_path,"standalonePath":if crate::standalone::mode(pool).await=="STANDALONE" {PathBuf::from(&path)}else{crate::standalone::own_path()},"sync":sync_status,"configured":cfg.is_some(),"connected":connected,"path":path,"branch":branch,"maxAgeMinutes":age,
        "dataRoot":root_dir,"dataRootWritable":is_dir_writable(&root_dir),
        "printDb":print_db_path(),"readOnly":true,"autoReady":connected && auto_ready(pool).await,
        "error":checked.err(),"message":"พร้อมใช้งานกับแหล่งข้อมูลสินค้า และรัน Profile ผ่าน Windows Task Scheduler"})
}

fn error(message: impl ToString) -> Response { (StatusCode::BAD_REQUEST,Json(json!({"error":message.to_string()}))).into_response() }
pub async fn get_status(State(st):State<AppState>) -> Response { Json(status(&st.pool).await).into_response() }
pub async fn check_config(State(st):State<AppState>,Json(cfg):Json<Config>) -> Response {
    if !matches_admin_pin(&st.pool,&cfg.pin).await{return (StatusCode::UNAUTHORIZED,Json(json!({"error":"กรุณาระบุ PIN ผู้ดูแลระบบให้ถูกต้อง"}))).into_response();}
    match validate(&cfg.path).await { Ok(v)=>Json(v).into_response(), Err(e)=>error(e) }
}
async fn saved_admin_pin(pool:&SqlitePool)->String {
    sqlx::query_scalar("SELECT FTSopValue FROM main.TCNSSysOption WHERE FTSopCode='AutoPrintAdminPin'").fetch_optional(pool).await.ok().flatten().unwrap_or_default()
}
pub async fn admin_pin_change_required(pool:&SqlitePool)->bool {
    let pin=saved_admin_pin(pool).await;
    pin.is_empty() || pin=="1234"
}
async fn matches_admin_pin(pool:&SqlitePool,pin:&str)->bool {
    if pin.trim().is_empty(){return false;}
    let saved=saved_admin_pin(pool).await;
    let expected=if let Some(protected)=saved.strip_prefix("dpapi:") {
        match crate::standalone::protect(protected,true).await {Ok(v)=>v,Err(_)=>return false}
    }else if saved.starts_with("local:"){
        match crate::standalone::protect(&saved,true).await {Ok(v)=>v,Err(_)=>return false}
    }else{saved};
    pin.trim()==expected
}
pub async fn valid_pin(pool:&SqlitePool,pin:&str)->bool {
    !admin_pin_change_required(pool).await && matches_admin_pin(pool,pin).await
}
fn validate_new_pin_value(current_pin: &str, new_pin: &str) -> Result<(), &'static str> {
    let np = new_pin.trim();
    if !(6..=12).contains(&np.len())
        || !np.bytes().all(|b| b.is_ascii_digit())
        || np == current_pin.trim()
        || np.bytes().all(|b| Some(b) == np.bytes().next())
    {
        return Err("PIN ใหม่ต้องเป็นตัวเลข 6–12 หลัก ไม่ซ้ำรหัสเดิมและไม่เป็นตัวเลขเดียวกันทั้งหมด");
    }
    Ok(())
}
async fn encrypt_admin_pin(new_pin: &str) -> Result<String, String> {
    let encrypted = crate::standalone::protect(new_pin.trim(), false).await?;
    Ok(if encrypted.starts_with("local:") { encrypted } else { format!("dpapi:{encrypted}") })
}
pub async fn verify_and_update_admin_pin(pool: &SqlitePool, pin: &str, new_pin: &str) -> Result<bool, (StatusCode, &'static str, bool)> {
    if !matches_admin_pin(pool, pin).await {
        return Err((StatusCode::UNAUTHORIZED, "รหัส PIN ผู้ดูแลระบบไม่ถูกต้อง", false));
    }
    let must_change = admin_pin_change_required(pool).await;
    let trimmed_new = new_pin.trim();
    if must_change && trimmed_new.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "กรุณาตั้ง PIN ผู้ดูแลระบบใหม่ (ตัวเลข 6–12 หลัก) ก่อนเข้าใช้งาน", true));
    }
    if !trimmed_new.is_empty() {
        validate_new_pin_value(pin, trimmed_new).map_err(|msg| (StatusCode::BAD_REQUEST, msg, must_change))?;
        let protected = encrypt_admin_pin(trimmed_new)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "ไม่สามารถเข้ารหัส PIN ใหม่ได้", must_change))?;
        sqlx::query("INSERT INTO TCNSSysOption(FTSopCode,FTSopApp,FTSopName,FTSopValue) VALUES('AutoPrintAdminPin','PrintBar','Admin PIN',?) ON CONFLICT(FTSopCode) DO UPDATE SET FTSopValue=excluded.FTSopValue")
            .bind(protected)
            .execute(pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "บันทึก PIN ใหม่ไม่สำเร็จ", must_change))?;
        return Ok(false);
    }
    Ok(must_change)
}
async fn has_barprint_identity(path: &Path) -> Result<bool, String> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .create_if_missing(false)
        .busy_timeout(Duration::from_secs(3));
    let mut c = SqliteConnection::connect_with(&options)
        .await
        .map_err(|_| "เปิดไฟล์ฐานข้อมูลเพื่อตรวจสอบไม่ได้".to_string())?;
    let owned: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ABMasterIdentity'",
    )
    .fetch_one(&mut c)
    .await
    .map_err(|e| e.to_string())?;
    let _ = c.close().await;
    Ok(owned > 0)
}
pub async fn save_config(State(st):State<AppState>,Json(mut cfg):Json<Config>) -> Response {
    if !matches_admin_pin(&st.pool,&cfg.pin).await{return (StatusCode::UNAUTHORIZED,Json(json!({"error":"กรุณาระบุ PIN ผู้ดูแลระบบให้ถูกต้อง"}))).into_response();}
    let must_change=admin_pin_change_required(&st.pool).await;
    if must_change && cfg.new_pin.trim().is_empty(){return error("กรุณาตั้ง PIN ผู้ดูแลระบบใหม่ก่อนเริ่มใช้งาน");}
    let new_pin=cfg.new_pin.trim();
    if !new_pin.is_empty() {
        if let Err(msg) = validate_new_pin_value(&cfg.pin, new_pin) {
            return error(msg);
        }
    }
    let protected_pin=if !new_pin.is_empty(){
        match encrypt_admin_pin(new_pin).await {Ok(v)=>Some(v),Err(e)=>return error(e)}
    }else{None};
    if !["UNIFIED","POS_SHARED","STANDALONE"].contains(&cfg.mode.as_str()){return error("โหมดแหล่งข้อมูลไม่ถูกต้อง");}
    if cfg.path.trim().is_empty() {
        cfg.path = default_local_master().to_string_lossy().into_owned();
    }
    let has_api = cfg
        .sync
        .as_ref()
        .map(|s| !s.base_url.trim().is_empty())
        .unwrap_or(false);
    let mut sync_json = String::from("{}");
    let effective_mode: String;
    if has_api {
        let Some(mut sync) = cfg.sync.take() else { return error("ระบุการเชื่อมต่อ API2PSMaster"); };
        sync.base_url = sync.base_url.trim().to_string();
        sync.agency = sync.agency.trim().to_string();
        sync.pos = sync.pos.trim().to_string();
        if sync.branch.trim().is_empty() {
            sync.branch = cfg.branch.trim().to_string();
        } else {
            sync.branch = sync.branch.trim().to_string();
        }
        if let Err(e) = crate::standalone::validate_config(&sync) {
            return error(e);
        }
        cfg.branch = sync.branch.clone();
        if sync.api_key.is_empty() {
            let previous: String = sqlx::query_scalar("SELECT config FROM ABSource WHERE id=1")
                .fetch_optional(&st.pool)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            let old: Option<crate::standalone::SyncConfig> = serde_json::from_str(&previous).ok();
            if let Some(old) = old {
                if old.base_url.trim() != sync.base_url || old.agency.trim() != sync.agency || old.branch.trim() != sync.branch {
                    return error("การเปลี่ยนปลายทางหรือสาขาต้องระบุ API Key ใหม่");
                }
                sync.api_key = old.api_key;
            }
            if sync.api_key.is_empty() {
                return error("ระบุ API Key");
            }
        } else {
            sync.api_key = match crate::standalone::protect(&sync.api_key, false).await {
                Ok(k) => k,
                Err(e) => return error(e),
            };
        }
        sync_json = serde_json::to_string(&sync).unwrap();
        if let Err(e) = crate::standalone::prepare_destination(Path::new(cfg.path.trim())).await {
            return error(e);
        }
        effective_mode = "STANDALONE".into();
    } else {
        let target_path = Path::new(cfg.path.trim());
        if !target_path.is_absolute() {
            return error("ระบุพาธเต็มของไฟล์ AdaAccD.db");
        }
        if target_path.is_file() {
            let is_owned = match has_barprint_identity(target_path).await {
                Ok(v) => v,
                Err(e) => return error(e),
            };
            if is_owned {
                effective_mode = "STANDALONE".into();
                if cfg.branch.trim().is_empty() {
                    cfg.branch = "00001".into();
                }
            } else {
                let check = match validate(&cfg.path).await {
                    Ok(v) => v,
                    Err(e) => return error(e),
                };
                let branch_arr = check["branches"].as_array().cloned().unwrap_or_default();
                if cfg.branch.trim().is_empty() && branch_arr.len() == 1 {
                    cfg.branch = branch_arr[0]["code"].as_str().unwrap_or("").to_string();
                }
                if !branch_arr.iter().any(|b| b["code"].as_str() == Some(cfg.branch.trim())) {
                    return error("เลือกสาขาที่มีอยู่ในฐานข้อมูล Master (กดตรวจสอบไฟล์ DB ก่อนบันทึก)");
                }
                effective_mode = "POS_SHARED".into();
            }
        } else {
            if cfg.mode == "POS_SHARED" {
                return error("เลือกไฟล์ AdaAccD.db ที่มีอยู่จริงด้วยพาธเต็ม");
            }
            if let Err(e) = crate::standalone::prepare_destination(target_path).await {
                return error(e);
            }
            if cfg.branch.trim().is_empty() {
                cfg.branch = "00001".into();
            }
            effective_mode = "STANDALONE".into();
        }
    }
    if !(1..=10080).contains(&cfg.max_age_minutes) { return error("อายุข้อมูลต้องอยู่ระหว่าง 1 ถึง 10080 นาที"); }
    let bootstrap = !cfg.operator_password.is_empty();
    if bootstrap && (has_api || effective_mode != "STANDALONE" || !(8..=64).contains(&cfg.operator_password.chars().count())) {
        return error("สร้างผู้ใช้แรกได้เฉพาะฐานภายในเครื่องที่ไม่มี API และต้องตั้งรหัสผ่าน 8–64 ตัวอักษร");
    }
    if !has_api && effective_mode == "STANDALONE" && !bootstrap {
        let options=SqliteConnectOptions::new().filename(cfg.path.trim()).read_only(true);
        let mut local=match SqliteConnection::connect_with(&options).await {Ok(c)=>c,Err(e)=>return error(e)};
        match sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMUsrLogin").fetch_one(&mut local).await {
            Ok(0)=>return error("ฐานภายในเครื่องยังไม่มีผู้ใช้ กรุณาตั้งรหัสผ่านสำหรับผู้ใช้ LOCAL ก่อนบันทึก"),
            Err(e)=>return error(e),
            _=>{}
        }
    }
    let canonical=match std::fs::canonicalize(cfg.path.trim()) { Ok(p)=>p.to_string_lossy().trim_start_matches("\\\\?\\").to_string(),Err(e)=>return error(e)};
    let result:Result<(),sqlx::Error>=async {
        let mut tx=st.pool.begin_with("BEGIN IMMEDIATE").await?;
        let pending:i64=sqlx::query_scalar("SELECT COUNT(*) FROM ABPrintJob WHERE status NOT IN ('SUBMITTED','EMPTY','CANCELLED')").fetch_one(&mut *tx).await?;
        if pending>0 { return Err(sqlx::Error::Protocol("ยังมีงานค้าง กรุณาจัดการคิวก่อนเปลี่ยนแหล่งข้อมูล".into())); }
        let source:Option<(String,String,i64)>=sqlx::query_as("SELECT mode,config,lease_until FROM ABSource WHERE id=1").fetch_optional(&mut *tx).await?;
        if source.as_ref().map(|v|v.2>chrono::Utc::now().timestamp()).unwrap_or(false){return Err(sqlx::Error::Protocol("รอการดาวน์โหลดที่กำลังทำงานให้เสร็จก่อนเปลี่ยนแหล่งข้อมูล".into()));}
        // Connection credentials and polling frequency are not a new dataset.
        // Preserve schedules, sessions and deduplication when only those change.
        let mut source_changed=source.as_ref().map(|v| {
            let before:Value=serde_json::from_str(&v.1).unwrap_or(Value::Null);
            let after:Value=serde_json::from_str(&sync_json).unwrap_or(Value::Null);
            v.0!=effective_mode || ["baseUrl","agency","branch","pos","shop"].iter().any(|key|before[*key]!=after[*key])
        }).unwrap_or(true);
        let old:Option<(String,String)>=sqlx::query_as("SELECT path,branch FROM ABStorage WHERE id=1").fetch_optional(&mut *tx).await?;
        source_changed |= old.as_ref().map(|v|v.0!=canonical || v.1!=cfg.branch.trim()).unwrap_or(true);
        if source_changed || old.as_ref().map(|v|v.0!=canonical || v.1!=cfg.branch.trim()).unwrap_or(true) {
            // Do not carry deduplication or enabled schedules across branch/database boundaries.
            sqlx::query("UPDATE ABPrintRule SET config=json_set(config,'$.enabled',json('false'))").execute(&mut *tx).await?;
            sqlx::query("DELETE FROM ABPrintWatermark").execute(&mut *tx).await?;
            sqlx::query("DELETE FROM ABPrintSetting WHERE id LIKE 'ruleSince:%' OR id LIKE 'run:%'").execute(&mut *tx).await?;
            sqlx::query("DELETE FROM TCNTLabelAuthSession").execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO ABStorage(id,path,branch,max_age_minutes) VALUES(1,?,?,?) ON CONFLICT(id) DO UPDATE SET path=excluded.path,branch=excluded.branch,max_age_minutes=excluded.max_age_minutes")
            .bind(&canonical).bind(cfg.branch.trim()).bind(cfg.max_age_minutes).execute(&mut *tx).await?;
        sqlx::query("UPDATE TCNSSysOption SET FTSopValue=? WHERE FTSopCode='DefaultBranch'").bind(cfg.branch.trim()).execute(&mut *tx).await?;
        let next_last_success_sql = if has_api {
            "CASE WHEN ? THEN '' ELSE last_success END"
        } else {
            "CASE WHEN NULLIF(last_success,'') IS NULL THEN datetime('now') ELSE last_success END"
        };
        let next_status = if has_api {
            "รอดาวน์โหลดครั้งแรก"
        } else if effective_mode == "POS_SHARED" {
            "เชื่อมต่อฐานข้อมูล Master แล้ว (อ่านอย่างเดียว)"
        } else {
            "พร้อมใช้งานฐานข้อมูลภายในเครื่อง (Manual / Import)"
        };
        let query_sql = format!(
            "INSERT INTO ABSource(id,mode,config,status,last_success) VALUES(1,?,?,?,{}) ON CONFLICT(id) DO UPDATE SET mode=excluded.mode,config=excluded.config,last_success={},status=CASE WHEN ? OR ?='' THEN ? ELSE status END",
            if has_api { "''" } else { "datetime('now')" },
            next_last_success_sql
        );
        let mut q = sqlx::query(&query_sql)
            .bind(&effective_mode)
            .bind(&sync_json)
            .bind(next_status);
        if has_api {
            q = q.bind(source_changed);
        }
        q.bind(source_changed)
            .bind(if has_api { "api" } else { "" })
            .bind(next_status)
            .execute(&mut *tx)
            .await?;
        if let Some(pin)=protected_pin {
            sqlx::query("UPDATE TCNSSysOption SET FTSopValue=? WHERE FTSopCode='AutoPrintAdminPin'").bind(pin).execute(&mut *tx).await?;
        }
        if bootstrap {
            let mut local = SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&canonical).create_if_missing(false)).await?;
            let mut write = local.begin_with("BEGIN IMMEDIATE").await?;
            sqlx::query("SELECT id FROM ABMasterIdentity LIMIT 0").execute(&mut *write).await?;
            let count:i64 = sqlx::query_scalar("SELECT COUNT(*) FROM TCNMUsrLogin").fetch_one(&mut *write).await?;
            if count != 0 { return Err(sqlx::Error::Protocol("ฐานนี้มีบัญชีผู้ใช้แล้ว ให้เข้าสู่ระบบด้วยบัญชีเดิม".into())); }
            let password=crate::auth::encrypt_pos_aes_b64(&cfg.operator_password).ok_or_else(||sqlx::Error::Protocol("สร้างรหัสผ่านไม่ได้".into()))?;
            sqlx::query("INSERT INTO TCNMUser(FTUsrCode) VALUES('LOCAL')").execute(&mut *write).await?;
            sqlx::query("INSERT INTO TCNMUser_L(FTUsrCode,FNLngID,FTUsrName) VALUES('LOCAL',1,'ผู้ใช้งานภายในเครื่อง')").execute(&mut *write).await?;
            sqlx::query("INSERT INTO TCNMUsrLogin(FTUsrCode,FTUsrLogType,FDUsrPwdStart,FTUsrLogin,FTUsrLoginPwd,FTUsrStaActive) VALUES('LOCAL','1',date('now','localtime'),'LOCAL',?,'1')").bind(password).execute(&mut *write).await?;
            write.commit().await?;
        }
        tx.commit().await
    }.await;
    match result {Ok(())=>Json(json!({"ok":true,"message":"บันทึกแหล่งข้อมูลแล้ว กรุณาเข้าสู่ระบบเพื่อใช้งาน"})).into_response(),Err(e)=>error(e)}
}

#[cfg(test)]
pub async fn test_master_promotion_schema(pool: &SqlitePool) {
    sqlx::raw_sql(r#"CREATE TABLE IF NOT EXISTS "TCNTPmtHD" (
            "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FNPmhVersion" INTEGER NOT NULL DEFAULT 0,
            "FTPmhStaActive" TEXT COLLATE NOCASE NOT NULL DEFAULT '1',
            "FTPmhName" TEXT COLLATE NOCASE,
            "FTPmhRmk" TEXT COLLATE NOCASE,
            "FTPmhStaDoc" TEXT COLLATE NOCASE,
            "FTPmhStaPrcDoc" TEXT COLLATE NOCASE,
            "FDPmhDStart" TEXT,
            "FDPmhDStop" TEXT,
            "FNPmhPriority" INTEGER NOT NULL DEFAULT 100,
            "FTPmhStaStack" TEXT COLLATE NOCASE,
            "FTPmhFmtCode" TEXT COLLATE NOCASE,
            "FNPmhSchemaVer" INTEGER NOT NULL DEFAULT 1,
            "FTPmhRuleJson" TEXT,
            "FDLastUpdOn" TEXT,
            "FTLastUpdBy" TEXT COLLATE NOCASE,
            "FDCreateOn" TEXT,
            "FTCreateBy" TEXT COLLATE NOCASE,
            PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmhVersion")
        );
CREATE TABLE IF NOT EXISTS "TCNTPmtHD_L" (
            "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FNPmhVersion" INTEGER NOT NULL DEFAULT 0,
            "FNLngID" INTEGER NOT NULL DEFAULT 0,
            "FTPmhName" TEXT COLLATE NOCASE,
            "FTPmhRmk" TEXT COLLATE NOCASE,
            "FDLastUpdOn" TEXT,
            "FTLastUpdBy" TEXT COLLATE NOCASE,
            "FDCreateOn" TEXT,
            "FTCreateBy" TEXT COLLATE NOCASE,
            PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmhVersion", "FNLngID")
        );
CREATE TABLE IF NOT EXISTS "TCNTPmtGrpDT" (
            "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FTPmdGrpName" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FNPmdSeqNo" INTEGER NOT NULL DEFAULT 0,
            "FTPmdStaType" TEXT COLLATE NOCASE,
            "FTPmdRefType" TEXT COLLATE NOCASE,
            "FTPmdRefCode" TEXT COLLATE NOCASE,
            "FCPmdQty" REAL,
            "FTPmdJson" TEXT,
            "FDLastUpdOn" TEXT,
            "FTLastUpdBy" TEXT COLLATE NOCASE,
            "FDCreateOn" TEXT,
            "FTCreateBy" TEXT COLLATE NOCASE,
            PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FTPmdGrpName", "FNPmdSeqNo")
        );
CREATE TABLE IF NOT EXISTS "TCNTPmtCond" (
            "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
            "FNPmcSeqNo" INTEGER NOT NULL DEFAULT 0,
            "FTPmcCondType" TEXT COLLATE NOCASE,
            "FTPmcRefCode" TEXT COLLATE NOCASE,
            "FCPmcValue" REAL,
            "FTPmcJson" TEXT,
            "FDLastUpdOn" TEXT,
            "FTLastUpdBy" TEXT COLLATE NOCASE,
            "FDCreateOn" TEXT,
            "FTCreateBy" TEXT COLLATE NOCASE,
            PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmcSeqNo")
        )"#).execute(pool).await.unwrap();
}
