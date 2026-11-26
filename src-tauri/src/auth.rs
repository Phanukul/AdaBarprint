#![allow(non_snake_case)]

use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use axum::{
    extract::{Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine as _};
use chrono::{Duration as ChronoDuration, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::routes::AppState;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Exact POS5/POS6 AES-128-CBC key & IV matching `AdaPos6.0StoreFront` (`auth.service.ts`).
const POS_AES_KEY: &[u8; 16] = b"5YpPTypXtwMML$u@";
const POS_AES_IV: &[u8; 16] = b"zNhQ$D%arP6U8waL";
const SESSION_TTL_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodOption {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub label: String,
    pub requires_user_selection: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserDto {
    pub id: String,
    pub name: String,
    pub login_name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSessionUser {
    pub id: String,
    pub name: String,
    pub role: String,
    pub roles: Vec<String>,
    pub branch_id: String,
    pub branch_name: String,
    pub locked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersQuery {
    pub credential_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub user_id: Option<String>,
    pub credential: Option<String>,
    pub credential_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    pub credential: Option<String>,
}

#[derive(Debug, FromRow)]
struct UsrLoginRow {
    FTUsrCode: String,
    FTUsrLogin: Option<String>,
    FTUsrLoginPwd: Option<String>,
    FDUsrPwdStart: Option<String>,
    FDUsrPwdExpired: Option<String>,
}

#[derive(Debug, FromRow)]
struct SessionRow {
    FTSesTokenHash: String,
    FTUsrCode: String,
    FTUsrName: String,
    FTRolesJson: String,
    FTBchCode: String,
    FTBchName: String,
    FTSesLocked: i64,
    FTSesExpiresAt: String,
}

fn err_json(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(json!({ "error": msg.into() }))).into_response()
}

pub async fn ensure_auth_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "TCNTLabelAuthSession" (
            "FTSesTokenHash"  TEXT NOT NULL PRIMARY KEY,
            "FTUsrCode"       TEXT NOT NULL,
            "FTUsrName"       TEXT NOT NULL,
            "FTRolesJson"     TEXT NOT NULL DEFAULT '[]',
            "FTBchCode"       TEXT NOT NULL DEFAULT '',
            "FTBchName"       TEXT NOT NULL DEFAULT '',
            "FTSesLocked"     INTEGER NOT NULL DEFAULT 0,
            "FTSesExpiresAt"  TEXT NOT NULL,
            "FDCreateOn"      TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn normalize_credential_type(raw: Option<&str>) -> &'static str {
    match raw.map(str::trim) {
        Some("1") => "1",
        Some("3") => "3",
        _ => "2",
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.is_empty() || a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn encrypt_pos_aes_b64(plain: &str) -> Option<String> {
    let enc = Aes128CbcEnc::new_from_slices(POS_AES_KEY, POS_AES_IV).ok()?;
    let bytes = plain.as_bytes();
    let mut buf = vec![0u8; bytes.len() + 16];
    buf[..bytes.len()].copy_from_slice(bytes);
    let ct = enc.encrypt_padded_mut::<Pkcs7>(&mut buf, bytes.len()).ok()?;
    Some(B64_STANDARD.encode(ct))
}

pub fn matches_database_credential(input: &str, stored: &str) -> bool {
    let stored_trimmed = stored.trim();
    if input.is_empty() || stored_trimmed.is_empty() {
        return false;
    }
    // 1) Optional plain PIN (legacy dev seeds <= 12 chars)
    if stored_trimmed.len() <= 12 && constant_time_eq(input.as_bytes(), stored_trimmed.as_bytes()) {
        return true;
    }
    // 2) Standard POS5/POS6 AES-128-CBC Base64
    if let Some(encrypted_b64) = encrypt_pos_aes_b64(input) {
        return constant_time_eq(encrypted_b64.as_bytes(), stored_trimmed.as_bytes());
    }
    false
}

fn parse_dt_loose(raw: &str) -> Option<NaiveDateTime> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    for fmt in [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }
    if s.len() >= 10 {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d") {
            return d.and_hms_opt(0, 0, 0);
        }
    }
    None
}

fn is_not_started(raw: Option<&str>) -> bool {
    match raw.and_then(parse_dt_loose) {
        Some(dt) => dt > Local::now().naive_local(),
        None => false,
    }
}

fn is_expired(raw: Option<&str>) -> bool {
    match raw.and_then(parse_dt_loose) {
        Some(dt) => dt < Local::now().naive_local(),
        None => false,
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode_bytes(&hasher.finalize())
}

mod hex {
    pub fn encode_bytes(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let val = headers.get(AUTHORIZATION)?.to_str().ok()?.trim();
    if let Some(rest) = val.strip_prefix("Bearer ").or_else(|| val.strip_prefix("bearer ")) {
        let t = rest.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

/// Enforce sessions on business APIs, including requests made outside the UI.
/// Source setup remains available before login.
pub async fn require_api_session(
    State(pool): State<SqlitePool>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let path = request.uri().path();
    if let Some(origin) = request.headers().get("origin") {
        let origin = origin.to_str().unwrap_or("");
        let host = request.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("");
        let same_origin = origin == format!("http://{host}") || origin == format!("https://{host}");
        let dev_origin = cfg!(debug_assertions) && matches!(origin, "http://localhost:5173" | "http://127.0.0.1:5173");
        if !same_origin && !dev_origin {
            return (StatusCode::FORBIDDEN, Json(json!({"error":"ไม่อนุญาตให้เรียกบริการจากเว็บไซต์อื่น"}))).into_response();
        }
    }
    let public = matches!(path,
        "/api/auth/login-options" | "/api/auth/users" | "/api/auth/login" |
        "/api/auth/me" | "/api/auth/lock" | "/api/auth/unlock" | "/api/auth/logout" |
        "/api/storage" | "/api/storage/check" | "/api/storage/sync");
    if path.starts_with("/api/") && !public {
        let Some(token) = bearer_token(request.headers()) else {
            return (StatusCode::UNAUTHORIZED, Json(json!({"error":"กรุณาเข้าสู่ระบบก่อนใช้งาน"}))).into_response();
        };
        match load_valid_session(&pool, &token).await {
            Ok(session) if session.FTSesLocked == 0 => {
                if session.FTBchCode != crate::storage::configured_branch(&pool).await {
                    return (StatusCode::UNAUTHORIZED, Json(json!({"error":"แหล่งข้อมูลเปลี่ยนแล้ว กรุณาเข้าสู่ระบบใหม่"}))).into_response();
                }
            }
            Ok(_) => return (StatusCode::LOCKED, Json(json!({"error":"กรุณาปลดล็อกหน้าจอก่อนใช้งาน"}))).into_response(),
            Err((status, message)) => return (status, Json(json!({"error":message}))).into_response(),
        }
    }
    next.run(request).await
}

async fn resolve_user_display_name(pool: &SqlitePool, usr_code: &str, fallback_login: Option<&str>) -> String {
    let name_opt: Option<String> = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT COALESCE(NULLIF(TRIM(UL.FTUsrName), ''), U.FTUsrCode)
        FROM TCNMUser U
        LEFT JOIN TCNMUser_L UL ON UL.FTUsrCode = U.FTUsrCode AND UL.FNLngID = 1
        WHERE U.FTUsrCode = ?
        LIMIT 1
        "#,
    )
    .bind(usr_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten();

    name_opt
        .filter(|s| !s.trim().is_empty())
        .or_else(|| fallback_login.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
        .unwrap_or_else(|| usr_code.to_string())
}

async fn resolve_user_roles(pool: &SqlitePool, usr_code: &str) -> (Vec<String>, String) {
    let roles: Vec<String> = sqlx::query_scalar::<_, Option<String>>(
        "SELECT DISTINCT TRIM(FTRolCode) FROM TCNMUsrActRole WHERE FTUsrCode = ? ORDER BY FTRolCode",
    )
    .bind(usr_code)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .flatten()
    .filter(|s| !s.is_empty())
    .collect();

    let display_role = if roles.is_empty() {
        "Staff".to_string()
    } else {
        format!("Role {}", roles[0])
    };
    (roles, display_role)
}

async fn resolve_default_branch(pool: &SqlitePool) -> (String, String) {
    let configured = crate::storage::configured_branch(pool).await;
    let bch_code = if configured.is_empty() { sqlx::query_scalar::<_, Option<String>>(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'DefaultBranch' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default()
    .trim()
    .to_string() } else { configured };

    if bch_code.is_empty() {
        return (String::new(), String::new());
    }

    let bch_name: String = sqlx::query_scalar::<_, Option<String>>(
        "SELECT FTBchName FROM TCNMBranch_L WHERE FTBchCode = ? AND NULLIF(TRIM(FTBchName), '') IS NOT NULL ORDER BY CASE WHEN FNLngID = 1 THEN 0 ELSE 1 END LIMIT 1",
    )
    .bind(&bch_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_else(|| bch_code.clone());

    (bch_code, bch_name)
}

async fn load_valid_session(pool: &SqlitePool, token: &str) -> Result<SessionRow, (StatusCode, &'static str)> {
    let token_hash = hash_token(token);
    let row: Option<SessionRow> = sqlx::query_as(
        "SELECT FTSesTokenHash, FTUsrCode, FTUsrName, FTRolesJson, FTBchCode, FTBchName, FTSesLocked, FTSesExpiresAt FROM TCNTLabelAuthSession WHERE FTSesTokenHash = ? LIMIT 1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "อ่านข้อมูล session ไม่สำเร็จ"))?;

    let Some(s) = row else {
        return Err((StatusCode::UNAUTHORIZED, "Session หมดอายุหรือไม่ได้รับอนุญาต"));
    };

    if let Some(exp_dt) = parse_dt_loose(&s.FTSesExpiresAt) {
        if exp_dt <= Local::now().naive_local() {
            let _ = sqlx::query("DELETE FROM TCNTLabelAuthSession WHERE FTSesTokenHash = ?")
                .bind(&token_hash)
                .execute(pool)
                .await;
            return Err((StatusCode::UNAUTHORIZED, "Session หมดอายุแล้ว กรุณาเข้าสู่ระบบใหม่"));
        }
    }

    Ok(s)
}

fn session_row_to_user(s: &SessionRow) -> AuthSessionUser {
    let roles: Vec<String> = serde_json::from_str(&s.FTRolesJson).unwrap_or_default();
    let role = if roles.is_empty() {
        "Staff".to_string()
    } else {
        format!("Role {}", roles[0])
    };
    AuthSessionUser {
        id: s.FTUsrCode.clone(),
        name: s.FTUsrName.clone(),
        role,
        roles,
        branch_id: s.FTBchCode.clone(),
        branch_name: s.FTBchName.clone(),
        locked: s.FTSesLocked != 0,
    }
}

/// GET /api/auth/login-options — mirrors StoreFront `getLoginOptions` from `TCNMAppModule`.
pub async fn get_login_options(State(st): State<AppState>) -> Response {
    let app_rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT TRIM(FTAppSignType)
        FROM TCNMAppModule
        WHERE CAST(FTAppModule AS TEXT) = '1'
          AND TRIM(FTAppSignType) IN ('1', '2', '3')
        ORDER BY FNAppSeqNo ASC
        "#,
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let (source, mut types): (&str, Vec<String>) = if !app_rows.is_empty() {
        ("TCNMAppModule", app_rows.into_iter().map(|r| r.0).collect())
    } else {
        // Default POS6 StoreFront fallback: Password ('1') + PIN ('2')
        ("fallback", vec!["2".to_string(), "1".to_string()])
    };

    // Ensure at least types present in TCNMUsrLogin are offered if using fallback
    if source == "fallback" {
        let existing_types: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT FTUsrLogType FROM TCNMUsrLogin WHERE COALESCE(NULLIF(TRIM(FTUsrStaActive), ''), '1') <> '2' AND FTUsrLogType IN ('1', '2', '3') ORDER BY FTUsrLogType DESC",
        )
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default();
        if !existing_types.is_empty() {
            types = existing_types.into_iter().map(|r| r.0).collect();
        }
    }

    let methods: Vec<AuthMethodOption> = types
        .into_iter()
        .enumerate()
        .map(|(idx, t)| {
            let label = match t.as_str() {
                "1" => "Password".to_string(),
                "3" => "RFID".to_string(),
                _ => "PIN".to_string(),
            };
            AuthMethodOption {
                requires_user_selection: t != "3",
                is_default: idx == 0,
                cred_type: t,
                label,
            }
        })
        .collect();

    Json(json!({
        "appCode": "PS",
        "source": source,
        "methods": methods,
    }))
    .into_response()
}

/// GET /api/auth/users?credentialType=1|2|3 — mirrors StoreFront `listCashiers` (never exposes passwords).
pub async fn get_auth_users(
    State(st): State<AppState>,
    Query(q): Query<UsersQuery>,
) -> Response {
    let cred_type = normalize_credential_type(q.credential_type.as_deref());
    if cred_type == "3" {
        return Json(json!({ "users": Vec::<AuthUserDto>::new() })).into_response();
    }

    let rows: Vec<(String, Option<String>, Option<String>)> = match sqlx::query_as(
        r#"
        SELECT U.FTUsrCode AS id,
               MAX(NULLIF(TRIM(UL.FTUsrName), '')) AS name,
               MAX(NULLIF(TRIM(L.FTUsrLogin), '')) AS login_name
        FROM TCNMUser U
        JOIN TCNMUsrLogin L
          ON L.FTUsrCode = U.FTUsrCode
         AND L.FTUsrLogType = ?
         AND COALESCE(NULLIF(TRIM(L.FTUsrStaActive), ''), '1') <> '2'
        LEFT JOIN TCNMUser_L UL
          ON UL.FTUsrCode = U.FTUsrCode
         AND UL.FNLngID = 1
        WHERE (L.FDUsrPwdStart IS NULL OR datetime(L.FDUsrPwdStart) <= datetime('now', 'localtime'))
          AND (L.FDUsrPwdExpired IS NULL OR datetime(L.FDUsrPwdExpired) >= datetime('now', 'localtime'))
        GROUP BY U.FTUsrCode
        ORDER BY U.FTUsrCode
        "#,
    )
    .bind(cred_type)
    .fetch_all(&st.pool)
    .await {
        Ok(rows) => rows,
        Err(_) => return err_json(StatusCode::SERVICE_UNAVAILABLE, "อ่านรายชื่อผู้ใช้ไม่ได้ กรุณาตรวจแหล่งข้อมูลและผลการดาวน์โหลดล่าสุด"),
    };

    let mut users = Vec::with_capacity(rows.len());
    for (id, name_opt, login_opt) in rows {
        let (_, role) = resolve_user_roles(&st.pool, &id).await;
        let login_name = login_opt.unwrap_or_default();
        let name = name_opt
            .filter(|s| !s.trim().is_empty())
            .or_else(|| if login_name.is_empty() { None } else { Some(login_name.clone()) })
            .unwrap_or_else(|| id.clone());
        users.push(AuthUserDto {
            id,
            name,
            login_name,
            role,
        });
    }

    Json(json!({ "users": users })).into_response()
}

async fn verify_user_credential(
    pool: &SqlitePool,
    user_input: &str,
    credential: &str,
    cred_type: &str,
) -> Result<(String, Option<String>), &'static str> {
    let trimmed_user = user_input.trim();
    let row: Option<UsrLoginRow> = if cred_type == "3" {
        sqlx::query_as(
            r#"
            SELECT FTUsrCode, FTUsrLogin, FTUsrLoginPwd, CAST(FDUsrPwdStart AS TEXT) AS FDUsrPwdStart, CAST(FDUsrPwdExpired AS TEXT) AS FDUsrPwdExpired
            FROM TCNMUsrLogin
            WHERE FTUsrLogType = '3'
              AND COALESCE(NULLIF(TRIM(FTUsrStaActive), ''), '1') <> '2'
              AND FTUsrLogin = ?
            ORDER BY FDUsrPwdStart DESC
            LIMIT 1
            "#,
        )
        .bind(credential)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
    } else {
        if trimmed_user.is_empty() {
            return Err("กรุณาระบุรหัสผู้ใช้งาน");
        }
        sqlx::query_as(
            r#"
            SELECT FTUsrCode, FTUsrLogin, FTUsrLoginPwd, CAST(FDUsrPwdStart AS TEXT) AS FDUsrPwdStart, CAST(FDUsrPwdExpired AS TEXT) AS FDUsrPwdExpired
            FROM TCNMUsrLogin
            WHERE FTUsrLogType = ?
              AND COALESCE(NULLIF(TRIM(FTUsrStaActive), ''), '1') <> '2'
              AND (FTUsrCode = ? OR FTUsrLogin = ?)
            ORDER BY CASE WHEN FTUsrCode = ? THEN 0 ELSE 1 END, FDUsrPwdStart DESC
            LIMIT 1
            "#,
        )
        .bind(cred_type)
        .bind(trimmed_user)
        .bind(trimmed_user)
        .bind(trimmed_user)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
    };

    let Some(login) = row else {
        return Err("รหัสผู้ใช้หรือข้อมูลยืนยันตัวตนไม่ถูกต้อง");
    };

    if is_not_started(login.FDUsrPwdStart.as_deref()) || is_expired(login.FDUsrPwdExpired.as_deref()) {
        return Err("บัญชีผู้ใช้งานยังไม่เริ่มใช้งานหรือหมดอายุแล้ว");
    }

    let matched = if cred_type == "3" {
        constant_time_eq(
            credential.as_bytes(),
            login.FTUsrLogin.as_deref().unwrap_or("").trim().as_bytes(),
        )
    } else {
        matches_database_credential(credential, login.FTUsrLoginPwd.as_deref().unwrap_or(""))
    };

    if !matched {
        return Err("รหัสผู้ใช้หรือข้อมูลยืนยันตัวตนไม่ถูกต้อง");
    }

    Ok((login.FTUsrCode, login.FTUsrLogin))
}

/// POST /api/auth/login — verify credentials against TCNMUsrLogin & issue Bearer session token.
pub async fn post_auth_login(
    State(st): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Response {
    let cred_type = normalize_credential_type(body.credential_type.as_deref());
    let user_id_input = body.user_id.as_deref().unwrap_or("").trim();
    let credential = body.credential.as_deref().unwrap_or("");

    if credential.trim().is_empty() {
        return err_json(StatusCode::UNAUTHORIZED, "กรุณากรอกข้อมูลยืนยันตัวตน");
    }

    let (usr_code, login_name) = match verify_user_credential(&st.pool, user_id_input, credential, cred_type).await {
        Ok(v) => v,
        Err(msg) => return err_json(StatusCode::UNAUTHORIZED, msg),
    };

    let usr_name = resolve_user_display_name(&st.pool, &usr_code, login_name.as_deref()).await;
    let (roles, role_label) = resolve_user_roles(&st.pool, &usr_code).await;
    let (branch_id, branch_name) = resolve_default_branch(&st.pool).await;

    let raw_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = hash_token(&raw_token);
    let now = Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let exp_str = (now + ChronoDuration::hours(SESSION_TTL_HOURS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let roles_json = serde_json::to_string(&roles).unwrap_or_else(|_| "[]".to_string());

    // Prune expired sessions and store the new session cleanly
    let _ = sqlx::query("DELETE FROM TCNTLabelAuthSession WHERE FTSesExpiresAt <= ?")
        .bind(&now_str)
        .execute(&st.pool)
        .await;

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNTLabelAuthSession (
            FTSesTokenHash, FTUsrCode, FTUsrName, FTRolesJson,
            FTBchCode, FTBchName, FTSesLocked, FTSesExpiresAt, FDCreateOn
        ) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(&token_hash)
    .bind(&usr_code)
    .bind(&usr_name)
    .bind(&roles_json)
    .bind(&branch_id)
    .bind(&branch_name)
    .bind(&exp_str)
    .bind(&now_str)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({
        "sessionToken": raw_token,
        "user": AuthSessionUser {
            id: usr_code,
            name: usr_name,
            role: role_label,
            roles,
            branch_id,
            branch_name,
            locked: false,
        }
    }))
    .into_response()
}

/// GET /api/auth/me — validate Bearer session token & return current user session.
pub async fn get_auth_me(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return err_json(StatusCode::UNAUTHORIZED, "กรุณาเข้าสู่ระบบ");
    };
    match load_valid_session(&st.pool, &token).await {
        Ok(s) => Json(json!({ "user": session_row_to_user(&s) })).into_response(),
        Err((status, msg)) => err_json(status, msg),
    }
}

/// POST /api/auth/lock — lock active session until unlocked by PIN/Password.
pub async fn post_auth_lock(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return err_json(StatusCode::UNAUTHORIZED, "กรุณาเข้าสู่ระบบ");
    };
    let session = match load_valid_session(&st.pool, &token).await {
        Ok(s) => s,
        Err((status, msg)) => return err_json(status, msg),
    };

    if let Err(e) = sqlx::query("UPDATE TCNTLabelAuthSession SET FTSesLocked = 1 WHERE FTSesTokenHash = ?")
        .bind(&session.FTSesTokenHash)
        .execute(&st.pool)
        .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({ "locked": true })).into_response()
}

/// POST /api/auth/unlock — verify PIN or Password for locked session user and unlock.
pub async fn post_auth_unlock(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UnlockRequest>,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return err_json(StatusCode::UNAUTHORIZED, "กรุณาเข้าสู่ระบบ");
    };
    let mut session = match load_valid_session(&st.pool, &token).await {
        Ok(s) => s,
        Err((status, msg)) => return err_json(status, msg),
    };

    let credential = body.credential.as_deref().unwrap_or("");
    if credential.trim().is_empty() {
        return err_json(StatusCode::UNAUTHORIZED, "กรุณากรอก PIN หรือรหัสผ่านเพื่อปลดล็อก");
    }

    // Try PIN ('2') first, then Password ('1') for the locked session user (mirrors StoreFront verifyCashierForShiftClose)
    let mut verified = false;
    for ct in ["2", "1"] {
        if verify_user_credential(&st.pool, &session.FTUsrCode, credential, ct)
            .await
            .is_ok()
        {
            verified = true;
            break;
        }
    }
    if !verified {
        return err_json(StatusCode::UNAUTHORIZED, "รหัส PIN หรือรหัสผ่านไม่ถูกต้อง");
    }

    if let Err(e) = sqlx::query("UPDATE TCNTLabelAuthSession SET FTSesLocked = 0 WHERE FTSesTokenHash = ?")
        .bind(&session.FTSesTokenHash)
        .execute(&st.pool)
        .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    session.FTSesLocked = 0;

    Json(json!({ "user": session_row_to_user(&session) })).into_response()
}

/// POST /api/auth/logout — revoke Bearer session token.
pub async fn post_auth_logout(State(st): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = bearer_token(&headers) {
        let token_hash = hash_token(&token);
        let _ = sqlx::query("DELETE FROM TCNTLabelAuthSession WHERE FTSesTokenHash = ?")
            .bind(&token_hash)
            .execute(&st.pool)
            .await;
    }
    Json(json!({ "loggedOut": true })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::Request,
        routing::{get, post},
        Router,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn setup_auth_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMUser (FTUsrCode TEXT PRIMARY KEY)",
            "CREATE TABLE IF NOT EXISTS TCNMUser_L (FTUsrCode TEXT, FNLngID INTEGER, FTUsrName TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMUsrLogin (FTUsrCode TEXT, FTUsrLogType TEXT, FTUsrLogin TEXT, FTUsrLoginPwd TEXT, FDUsrPwdStart TEXT, FDUsrPwdExpired TEXT, FTUsrStaActive TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMUsrActRole (FTRolCode TEXT, FTUsrCode TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMAppModule (FTAppCode TEXT, FTAppModule TEXT, FTAppSignType TEXT, FNAppSeqNo INTEGER)",
            "CREATE TABLE IF NOT EXISTS TCNSSysOption (FTSopCode TEXT PRIMARY KEY, FTSopValue TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMBranch_L (FTBchCode TEXT, FNLngID INTEGER, FTBchName TEXT)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        ensure_auth_schema(&pool).await.unwrap();

        sqlx::query("INSERT INTO TCNMUser VALUES ('00033')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO TCNMUser_L VALUES ('00033', 1, 'Store Admin')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO TCNMUsrActRole VALUES ('00070', '00033')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO TCNSSysOption VALUES ('DefaultBranch', '00001')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO TCNMBranch_L VALUES ('00001', 1, 'สาขาหลัก')")
            .execute(&pool)
            .await
            .unwrap();

        let aes_pin = encrypt_pos_aes_b64("1234").unwrap();
        let aes_pwd = encrypt_pos_aes_b64("AdaSoft@1").unwrap();
        sqlx::query("INSERT INTO TCNMUsrLogin VALUES ('00033', '2', '0810450011', ?, '2024-01-01', '2099-12-31', '1')")
            .bind(aes_pin)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO TCNMUsrLogin VALUES ('00033', '1', '0810450011', ?, '2024-01-01', '2099-12-31', '1')")
            .bind(aes_pwd)
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn business_routes_reject_missing_session_and_foreign_origin() {
        let pool = setup_auth_test_db().await;
        let app = Router::new()
            .route("/api/protected", get(|| async { StatusCode::OK }))
            .layer(axum::middleware::from_fn_with_state(pool, require_api_session));
        let anonymous = app.clone().oneshot(Request::builder().uri("/api/protected").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
        let cross_site = app.oneshot(Request::builder().uri("/api/protected")
            .header("host", "localhost:3000").header("origin", "https://untrusted.example")
            .body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(cross_site.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn settings_use_the_signed_in_session_and_respect_screen_lock() {
        let pool=setup_auth_test_db().await;
        sqlx::raw_sql("CREATE TABLE TCNSLabelStorage(FNLstID INTEGER PRIMARY KEY,FTBchCode TEXT); INSERT INTO TCNSLabelStorage VALUES(1,'00001')").execute(&pool).await.unwrap();
        let app=Router::new()
            .route("/api/auth/login",post(post_auth_login))
            .route("/api/auth/lock",post(post_auth_lock))
            .route("/api/config/options",post(||async{StatusCode::OK}))
            .with_state(AppState{pool:pool.clone()})
            .layer(axum::middleware::from_fn_with_state(pool.clone(),require_api_session));
        let login=app.clone().oneshot(Request::builder().method("POST").uri("/api/auth/login")
            .header("content-type","application/json")
            .body(Body::from(r#"{"userId":"00033","credential":"1234","credentialType":"2"}"#)).unwrap()).await.unwrap();
        assert_eq!(login.status(),StatusCode::OK);
        let bytes=axum::body::to_bytes(login.into_body(),65536).await.unwrap();
        let response:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
        let token=response["sessionToken"].as_str().unwrap();
        let request=|path:&str|Request::builder().method("POST").uri(path)
            .header("authorization",format!("Bearer {token}")).body(Body::empty()).unwrap();
        assert_eq!(app.clone().oneshot(request("/api/config/options")).await.unwrap().status(),StatusCode::OK);
        assert_eq!(app.clone().oneshot(request("/api/auth/lock")).await.unwrap().status(),StatusCode::OK);
        assert_eq!(app.oneshot(request("/api/config/options")).await.unwrap().status(),StatusCode::LOCKED);
        pool.close().await;
    }

    #[tokio::test]
    async fn storefront_aes_login_me_lock_unlock_logout_flow() {
        let pool = setup_auth_test_db().await;
        let app = Router::new()
            .route("/api/auth/login-options", get(get_login_options))
            .route("/api/auth/users", get(get_auth_users))
            .route("/api/auth/login", post(post_auth_login))
            .route("/api/auth/me", get(get_auth_me))
            .route("/api/auth/lock", post(post_auth_lock))
            .route("/api/auth/unlock", post(post_auth_unlock))
            .route("/api/auth/logout", post(post_auth_logout))
            .with_state(AppState { pool });

        // 1) Known POS6 AES vector (`1234` -> matches StoreFront AES key/iv)
        let enc_1234 = encrypt_pos_aes_b64("1234").unwrap();
        assert!(matches_database_credential("1234", &enc_1234));
        assert!(!matches_database_credential("9999", &enc_1234));

        // 2) List users (no passwords leaked)
        let res_users = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/users?credentialType=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_users.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(res_users.into_body(), 1024 * 64).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body_json["users"][0]["id"], "00033");

        // 3) Bad PIN -> 401
        let res_bad = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"userId":"00033","credential":"0000","credentialType":"2"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_bad.status(), StatusCode::UNAUTHORIZED);

        // 4) Valid PIN -> Bearer token + branch
        let res_ok = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"userId":"00033","credential":"1234","credentialType":"2"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_ok.status(), StatusCode::OK);
        let ok_bytes = axum::body::to_bytes(res_ok.into_body(), 1024 * 64).await.unwrap();
        let ok_json: serde_json::Value = serde_json::from_slice(&ok_bytes).unwrap();
        let token = ok_json["sessionToken"].as_str().unwrap().to_string();
        assert_eq!(ok_json["user"]["name"], "Store Admin");
        assert_eq!(ok_json["user"]["branchId"], "00001");

        // 5) Lock -> Me shows locked -> Unlock -> Logout
        let res_lock = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/lock")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_lock.status(), StatusCode::OK);

        let res_unlock = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/unlock")
                    .header("authorization", format!("Bearer {}", token))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"credential":"1234"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_unlock.status(), StatusCode::OK);

        let res_logout = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_logout.status(), StatusCode::OK);

        // After logout, /api/auth/me -> 401
        let res_me_after = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/me")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_me_after.status(), StatusCode::UNAUTHORIZED);
    }
}
