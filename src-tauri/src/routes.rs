#![allow(non_snake_case)]

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Local, NaiveDate, NaiveTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Connection, SqlitePool};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use crate::db::{
    apply_promo, format_date_th, is_seeded, promo_interval, resolve_accd_promotion, seed_defaults,
};
use crate::models::*;
use crate::print_drivers::{
    generate_commands, generate_test_pattern, tcp_send, tcp_test, DriverOptions, PrinterBrand,
};
use crate::templates::get_templates;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

#[cfg(target_os = "windows")]
fn new_powershell_cmd() -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("powershell");
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

fn err_json(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(json!({ "error": msg.into() }))).into_response()
}

fn gen_guid() -> String {
    Uuid::new_v4().to_string().replace('-', "").to_uppercase()
}

async fn resolve_default_user(pool: &SqlitePool) -> String {
    if let Ok(Some((v,))) = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'DefaultUser' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        if let Some(s) = v.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()) {
            return s;
        }
    }
    if let Ok(Some((u,))) = sqlx::query_as::<_, (String,)>(
        "SELECT FTUsrCode FROM TCNMUser ORDER BY FTUsrCode ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        if !u.trim().is_empty() {
            return u;
        }
    }
    "SYSTEM".to_string()
}

async fn lookup_paper_size(pool: &SqlitePool, raw_code: Option<&str>) -> (String, f64, f64, String) {
    let want = raw_code.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("A4");
    if let Ok(Some(ps)) = sqlx::query_as::<_, TCNMPaperSize>(
        "SELECT FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse FROM TCNMPaperSize WHERE FTPpsCode = ? AND FTPpsStaUse = '1' LIMIT 1",
    )
    .bind(want)
    .fetch_optional(pool)
    .await
    {
        return (
            ps.FTPpsCode,
            ps.FCPpsWidth,
            ps.FCPpsHeight,
            ps.FTPpsWinKind.unwrap_or_else(|| "A4".to_string()),
        );
    }
    if let Ok(Some(ps)) = sqlx::query_as::<_, TCNMPaperSize>(
        "SELECT FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse FROM TCNMPaperSize WHERE FTPpsStaUse = '1' ORDER BY FNPpsSeq ASC, FTPpsCode ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        return (
            ps.FTPpsCode,
            ps.FCPpsWidth,
            ps.FCPpsHeight,
            ps.FTPpsWinKind.unwrap_or_else(|| "A4".to_string()),
        );
    }
    ("A4".to_string(), 210.0, 297.0, "A4".to_string())
}

async fn fetch_preprint_map(pool: &SqlitePool) -> HashMap<String, TCNMPreprint> {
    let rows = sqlx::query_as::<_, TCNMPreprint>(
        "SELECT FTPprCode, FTPprName, FTPprFileName, FTPpsCode, FCPprWidth, FCPprHeight, FNPprCol, FNPprRow, FCPprLabelW, FCPprLabelH, FCPprMagTop, FCPprMagLeft, FCPprMagBottom, FCPprMagRight, FCPprGapH, FCPprGapV, FNPprPageNo, FTPprScope, FCPprOpacity, FTPprSheetImg, FTPprCellImg, FTPprCropJson, FTPprStaUse FROM TCNMPreprint WHERE FTPprStaUse = '1'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows.into_iter().map(|p| (p.FTPprCode.clone(), p)).collect()
}

fn hydrate_tmpl_layout(tmpl: &TCNMLabelTmpl, preprint_map: &HashMap<String, TCNMPreprint>) -> Option<String> {
    let raw = tmpl.FTLtmLayout.as_ref()?;
    let Some(ref ppr_code) = tmpl.FTPprCode else {
        return Some(raw.clone());
    };
    let Some(pp) = preprint_map.get(ppr_code) else {
        return Some(raw.clone());
    };
    let Ok(mut val) = serde_json::from_str::<Value>(raw) else {
        return Some(raw.clone());
    };
    if let Some(obj) = val.as_object_mut() {
        obj.insert(
            "designMode".to_string(),
            json!(tmpl.FTLtmMode.as_deref().unwrap_or("preprint")),
        );
        let mut pp_obj = obj
            .get("preprint")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        pp_obj.insert("preprintCode".to_string(), json!(pp.FTPprCode));
        pp_obj.insert("sourceName".to_string(), json!(pp.FTPprFileName.as_deref().unwrap_or(&pp.FTPprName)));
        pp_obj.insert("scope".to_string(), json!(pp.FTPprScope));
        pp_obj.insert("paperSize".to_string(), json!(pp.FTPpsCode));
        pp_obj.insert("paperWidthMm".to_string(), json!(pp.FCPprWidth));
        pp_obj.insert("paperHeightMm".to_string(), json!(pp.FCPprHeight));
        pp_obj.insert("pageIndex".to_string(), json!(pp.FNPprPageNo.unwrap_or(1)));
        if let Some(op) = pp.FCPprOpacity {
            if !pp_obj.contains_key("opacity") {
                pp_obj.insert("opacity".to_string(), json!(op));
            }
        }
        if let Some(ref s_img) = pp.FTPprSheetImg {
            pp_obj.insert("sheetBgDataUrl".to_string(), json!(s_img));
        }
        if let Some(ref c_img) = pp.FTPprCellImg {
            pp_obj.insert("cellBgDataUrl".to_string(), json!(c_img));
        }
        if let Some(ref crop_str) = pp.FTPprCropJson {
            if let Ok(cv) = serde_json::from_str::<Value>(crop_str) {
                pp_obj.insert("cropBox".to_string(), cv);
            }
        }
        obj.insert("preprint".to_string(), Value::Object(pp_obj));
    }
    serde_json::to_string(&val).ok().or_else(|| Some(raw.clone()))
}

async fn upsert_preprint_from_layout<'a>(
    executor: impl sqlx::Executor<'a, Database = sqlx::Sqlite>,
    template_code: Option<&str>,
    lbs_code: &str,
    lbs_name: &str,
    cols: i64,
    rows: i64,
    w_mm: f64,
    h_mm: f64,
    mag_top: f64,
    mag_left: f64,
    mag_bottom: f64,
    mag_right: f64,
    gap_h: f64,
    gap_v: f64,
    layout_val: &mut Value,
) -> Result<(String, Option<String>), sqlx::Error> {
    let mode = layout_val
        .get("designMode")
        .and_then(|v| v.as_str())
        .map(|s| if s == "preprint" { "preprint" } else { "full" })
        .unwrap_or("full")
        .to_string();

    let Some(pp_obj) = layout_val.get("preprint").and_then(|v| v.as_object()).cloned() else {
        return Ok((mode, None));
    };

    let existing_code = pp_obj
        .get("preprintCode")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    let sheet_img = pp_obj
        .get("sheetBgDataUrl")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    let cell_img = pp_obj
        .get("cellBgDataUrl")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    if sheet_img.is_none() && cell_img.is_none() {
        return Ok((mode, existing_code));
    }

    let ppr_code = template_code.map(|code| format!("PPR-DESIGN-{}", code))
        .or(existing_code).unwrap_or_else(|| format!("PPR-{}", lbs_code));
    let src_name = pp_obj
        .get("sourceName")
        .and_then(|v| v.as_str())
        .map(String::from);
    let ppr_name = format!("Pre-Print {}", lbs_name);
    let scope = pp_obj
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("sheet");
    let paper_code = pp_obj
        .get("paperSize")
        .and_then(|v| v.as_str())
        .unwrap_or("A4");
    let paper_w = pp_obj
        .get("paperWidthMm")
        .and_then(|v| v.as_f64())
        .unwrap_or(210.0);
    let paper_h = pp_obj
        .get("paperHeightMm")
        .and_then(|v| v.as_f64())
        .unwrap_or(297.0);
    let page_no = pp_obj.get("pageIndex").and_then(|v| v.as_i64()).unwrap_or(1);
    let opacity = pp_obj.get("opacity").and_then(|v| v.as_f64()).unwrap_or(0.72);
    let crop_json = pp_obj
        .get("cropBox")
        .and_then(|v| serde_json::to_string(v).ok());
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        r#"
        INSERT INTO TCNMPreprint
            (FTPprCode, FTPprName, FTPprFileName, FTPpsCode, FCPprWidth, FCPprHeight, FNPprCol, FNPprRow, FCPprLabelW, FCPprLabelH, FCPprMagTop, FCPprMagLeft, FCPprMagBottom, FCPprMagRight, FCPprGapH, FCPprGapV, FNPprPageNo, FTPprScope, FCPprOpacity, FTPprSheetImg, FTPprCellImg, FTPprCropJson, FTPprStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'DESIGNER', ?, 'DESIGNER')
        ON CONFLICT(FTPprCode) DO UPDATE SET
            FTPprFileName = COALESCE(excluded.FTPprFileName, TCNMPreprint.FTPprFileName),
            FTPpsCode = excluded.FTPpsCode,
            FCPprWidth = excluded.FCPprWidth,
            FCPprHeight = excluded.FCPprHeight,
            FNPprCol = excluded.FNPprCol,
            FNPprRow = excluded.FNPprRow,
            FCPprLabelW = excluded.FCPprLabelW,
            FCPprLabelH = excluded.FCPprLabelH,
            FCPprMagTop = excluded.FCPprMagTop,
            FCPprMagLeft = excluded.FCPprMagLeft,
            FCPprMagBottom = excluded.FCPprMagBottom,
            FCPprMagRight = excluded.FCPprMagRight,
            FCPprGapH = excluded.FCPprGapH,
            FCPprGapV = excluded.FCPprGapV,
            FNPprPageNo = excluded.FNPprPageNo,
            FTPprScope = excluded.FTPprScope,
            FCPprOpacity = excluded.FCPprOpacity,
            FTPprSheetImg = COALESCE(excluded.FTPprSheetImg, TCNMPreprint.FTPprSheetImg),
            FTPprCellImg = COALESCE(excluded.FTPprCellImg, TCNMPreprint.FTPprCellImg),
            FTPprCropJson = COALESCE(excluded.FTPprCropJson, TCNMPreprint.FTPprCropJson),
            FTPprStaUse = '1',
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'DESIGNER'
        "#,
    )
    .bind(&ppr_code)
    .bind(&ppr_name)
    .bind(&src_name)
    .bind(paper_code)
    .bind(paper_w)
    .bind(paper_h)
    .bind(cols.max(1))
    .bind(rows.max(1))
    .bind(w_mm)
    .bind(h_mm)
    .bind(mag_top)
    .bind(mag_left)
    .bind(mag_bottom)
    .bind(mag_right)
    .bind(gap_h)
    .bind(gap_v)
    .bind(page_no)
    .bind(scope)
    .bind(opacity)
    .bind(&sheet_img)
    .bind(&cell_img)
    .bind(&crop_json)
    .bind(&now)
    .bind(&now)
    .execute(executor)
    .await?;

    // Reference FTPprCode from layout metadata instead of duplicating large Base64 blobs in TCNMLabelTmpl
    if let Some(obj) = layout_val.as_object_mut() {
        if let Some(pp) = obj.get_mut("preprint").and_then(|v| v.as_object_mut()) {
            pp.insert("preprintCode".to_string(), json!(ppr_code));
            pp.remove("sheetBgDataUrl");
            pp.remove("cellBgDataUrl");
        }
    }

    Ok((mode, Some(ppr_code)))
}

pub(crate) async fn fetch_lan_printers(pool: &SqlitePool) -> Vec<Value> {
    let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT p.FTPrnCode, COALESCE(p.FTPrnType, 'GODEX'), pl.FTPrnName, sp.FTSppValue, sp.FTSppRef, sp.FTSppStaUse
        FROM TCNMPrinter p
        LEFT JOIN TCNMPrinter_L pl ON pl.FTPrnCode = p.FTPrnCode AND pl.FNLngID = 1
        LEFT JOIN TSysPortPrn sp ON sp.FTSppCode = p.FTSppCode
        WHERE COALESCE(p.FTPrnSrcType, '2') = '2'
        ORDER BY p.FTPrnCode ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|(code, brand, name, ip_port, ref_json, sta)| {
            let mut ip = String::new();
            let mut port = 9100_i64;
            if let Some(ref s) = ip_port {
                if let Some((h, p)) = s.rsplit_once(':') {
                    ip = h.to_string();
                    port = p.parse().unwrap_or(9100);
                } else if !s.trim().is_empty() {
                    ip = s.trim().to_string();
                }
            }
            let meta: Value = ref_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_else(|| json!({}));
            json!({
                "id": code,
                "name": name.unwrap_or_else(|| code.clone()),
                "brand": brand,
                "model": meta.get("model").and_then(|v| v.as_str()).unwrap_or(&brand),
                "ip": ip,
                "port": port,
                "dpi": meta.get("dpi").and_then(|v| v.as_i64()).unwrap_or(203),
                "labelWidth": meta.get("labelWidth").and_then(|v| v.as_f64()).unwrap_or(100.0),
                "labelHeight": meta.get("labelHeight").and_then(|v| v.as_f64()).unwrap_or(38.0),
                "speed": meta.get("speed").and_then(|v| v.as_i64()).unwrap_or(4),
                "darkness": meta.get("darkness").and_then(|v| v.as_i64()).unwrap_or(10),
                "enabled": sta.as_deref().unwrap_or("1") == "1",
            })
        })
        .collect()
}

async fn fetch_company_info(pool: &SqlitePool) -> Value {
    let row: Option<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT c.FTCmpCode, c.FTBchcode, cl.FTCmpName, cl.FTCmpShop
        FROM TCNMComp c
        LEFT JOIN TCNMComp_L cl ON cl.FTCmpCode = c.FTCmpCode AND cl.FNLngID = 1
        ORDER BY c.FTCmpCode ASC LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let bch_row: Option<(String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT b.FTBchCode, bl.FTBchName
        FROM TCNMBranch b
        LEFT JOIN TCNMBranch_L bl ON bl.FTBchCode = b.FTBchCode AND bl.FNLngID = 1
        WHERE COALESCE(b.FTBchStaActive, '1') = '1'
        ORDER BY b.FTBchCode ASC LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some((cmp_code, cmp_bch, cmp_name, cmp_shop)) = row {
        let shop = cmp_shop
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| cmp_name.as_deref().map(str::trim).filter(|s| !s.is_empty()))
            .or_else(|| bch_row.as_ref().and_then(|b| b.1.as_deref()))
            .unwrap_or("AdaSoft")
            .to_string();
        json!({
            "cmpCode": cmp_code,
            "cmpName": cmp_name.unwrap_or_else(|| shop.clone()),
            "shopName": shop,
            "defaultBchCode": cmp_bch.filter(|s| !s.trim().is_empty()).or_else(|| bch_row.as_ref().map(|b| b.0.clone())).unwrap_or_default(),
            "defaultBchName": bch_row.and_then(|b| b.1).unwrap_or_default(),
        })
    } else {
        json!({
            "cmpCode": "",
            "cmpName": bch_row.as_ref().and_then(|b| b.1.clone()).unwrap_or_default(),
            "shopName": bch_row.as_ref().and_then(|b| b.1.clone()).unwrap_or_default(),
            "defaultBchCode": bch_row.as_ref().map(|b| b.0.clone()).unwrap_or_default(),
            "defaultBchName": bch_row.and_then(|b| b.1).unwrap_or_default(),
        })
    }
}

// ─── /api/seed ───────────────────────────────────────────────────────────────

pub async fn get_seed(State(st): State<AppState>) -> Response {
    match is_seeded(&st.pool).await {
        Ok(seeded) => Json(json!({
            "seeded": seeded,
            "accdConnected": true
        }))
        .into_response(),
        Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

pub async fn post_seed(State(st): State<AppState>) -> Response {
    match seed_defaults(&st.pool).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

// ─── /api/branches ───────────────────────────────────────────────────────────

pub async fn get_branches(State(st): State<AppState>) -> Response {
    let rows = sqlx::query_as::<_, TCNMBranch>(
        r#"
        SELECT b.FTBchCode,
               COALESCE(bl.FTBchName, b.FTBchCode) AS FTBchName,
               COALESCE(b.FTBchStaActive, '1') AS FTBchStaUse,
               CAST(b.FDCreateOn AS TEXT) AS FDCreateOn,
               b.FTCreateBy,
               CAST(b.FDLastUpdOn AS TEXT) AS FDLastUpdOn,
               b.FTLastUpdBy
        FROM TCNMBranch b
        LEFT JOIN TCNMBranch_L bl ON bl.FTBchCode = b.FTBchCode AND bl.FNLngID = 1
        WHERE COALESCE(b.FTBchStaActive, '1') = '1'
        ORDER BY b.FTBchCode ASC
        "#,
    )
    .fetch_all(&st.pool)
    .await;

    match rows {
        Ok(list) => Json(list).into_response(),
        Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

// ─── /api/filters/master ─────────────────────────────────────────────────────

#[derive(serde::Serialize, sqlx::FromRow)]
struct FilterOption {
    code: String,
    name: String,
}

async fn query_filter_options(pool: &sqlx::SqlitePool, sql: &str) -> Vec<FilterOption> {
    sqlx::query_as::<_, FilterOption>(sql)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

pub async fn get_filter_master(State(st): State<AppState>) -> Response {
    let accd = &st.pool;

    let groups = query_filter_options(
        accd,
        "SELECT FTPgpChain AS code, COALESCE(FTPgpName, FTPgpChain) AS name FROM TCNMPdtGrp_L WHERE FNLngID = 1 ORDER BY FTPgpChain ASC",
    )
    .await;
    let types = query_filter_options(
        accd,
        "SELECT FTPtyCode AS code, COALESCE(FTPtyName, FTPtyCode) AS name FROM TCNMPdtType_L WHERE FNLngID = 1 ORDER BY FTPtyCode ASC",
    )
    .await;
    let brands = query_filter_options(
        accd,
        "SELECT FTPbnCode AS code, COALESCE(FTPbnName, FTPbnCode) AS name FROM TCNMPdtBrand_L WHERE FNLngID = 1 ORDER BY FTPbnCode ASC",
    )
    .await;
    let models = query_filter_options(
        accd,
        "SELECT FTPmoCode AS code, COALESCE(FTPmoName, FTPmoCode) AS name FROM TCNMPdtModel_L WHERE FNLngID = 1 ORDER BY FTPmoCode ASC",
    )
    .await;
    let price_groups = query_filter_options(
        accd,
        "SELECT FTPplCode AS code, COALESCE(FTPplName, FTPplCode) AS name FROM TCNMPdtPriList_L WHERE FNLngID = 1 ORDER BY FTPplCode ASC",
    )
    .await;
    let agencies = query_filter_options(
        accd,
        "SELECT FTAgnCode AS code, COALESCE(FTAgnName, FTAgnCode) AS name FROM TCNMAgency_L WHERE FNLngID = 1 ORDER BY FTAgnCode ASC",
    )
    .await;

    Json(json!({
        "groups": groups,
        "types": types,
        "brands": brands,
        "models": models,
        "priceGroups": price_groups,
        "agencies": agencies,
    }))
    .into_response()
}

// ─── /api/label-sizes ────────────────────────────────────────────────────────

pub async fn get_label_sizes(State(st): State<AppState>) -> Response {
    let sizes = sqlx::query_as::<_, TCNMLabelSize>(
        "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsStaUse = '1' ORDER BY FTLbsCode ASC",
    )
    .fetch_all(&st.pool)
    .await;

    let sizes = match sizes {
        Ok(s) => s,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let tmpls = sqlx::query_as::<_, TCNMLabelTmpl>(
        "SELECT FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode, FTLtmOriginal IS NOT NULL AS hasOriginal FROM TCNMLabelTmpl WHERE FTLtmStaUse = '1'",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let preprint_map = fetch_preprint_map(&st.pool).await;

    let printers = sqlx::query_as::<_, TCNMLabelPrinter>(
        "SELECT FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNMLabelPrinter WHERE FTLbpStaUse = '1'",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let result: Vec<Value> = sizes
        .into_iter()
        .map(|s| {
            let s_tmpls: Vec<Value> = tmpls
                .iter()
                .filter(|t| t.FTLbsCode == s.FTLbsCode)
                .map(|t| {
                    let hydrated_layout = hydrate_tmpl_layout(t, &preprint_map);
                    json!({
                        "FTLtmCode": t.FTLtmCode,
                        "FTLtmName": t.FTLtmName,
                        "FTLtmPriType": t.FTLtmPriType,
                        "FTLtmLayout": hydrated_layout,
                        "hasOriginal": t.hasOriginal,
                        "FTLtmMode": t.FTLtmMode,
                        "FTPprCode": t.FTPprCode
                    })
                })
                .collect();
            let s_printers: Vec<&TCNMLabelPrinter> = printers
                .iter()
                .filter(|p| p.FTLbsCode == s.FTLbsCode)
                .collect();
            let mut val = serde_json::to_value(&s).unwrap_or(json!({}));
            if let Some(obj) = val.as_object_mut() {
                obj.insert("templates".to_string(), json!(s_tmpls));
                obj.insert("printers".to_string(), json!(s_printers));
            }
            val
        })
        .collect();

    Json(result).into_response()
}

// ─── /api/templates ──────────────────────────────────────────────────────────

pub async fn get_templates_route(State(st): State<AppState>) -> Response {
    let mut db_tmpls = sqlx::query_as::<_, TCNMLabelTmpl>(
        "SELECT FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode FROM TCNMLabelTmpl WHERE FTLtmStaUse = '1' ORDER BY FTLtmCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let preprint_map = fetch_preprint_map(&st.pool).await;
    for t in &mut db_tmpls {
        if let Some(hl) = hydrate_tmpl_layout(t, &preprint_map) {
            t.FTLtmLayout = Some(hl);
        }
    }

    Json(json!({
        "static": get_templates(),
        "db": db_tmpls
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTemplateReq {
    pub template_code: Option<String>,
    pub lbs_code: String,
    pub pri_type: String,
    pub name: Option<String>,
    pub layout: Value,
    pub paper_size: Option<String>,
    pub paper_width_mm: Option<f64>,
    pub paper_height_mm: Option<f64>,
    pub cols: Option<i64>,
    pub rows: Option<i64>,
    pub mag_top: Option<f64>,
    pub mag_left: Option<f64>,
    pub mag_bottom: Option<f64>,
    pub mag_right: Option<f64>,
    pub gap_h: Option<f64>,
    pub gap_v: Option<f64>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
}

pub async fn post_save_template(
    State(st): State<AppState>,
    Json(body): Json<SaveTemplateReq>,
) -> Response {
    let lbs_code = body.lbs_code.trim();
    if lbs_code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lbsCode is required");
    }
    let pri_type = if body.pri_type.trim() == "2" { "2" } else { "1" };
    let design_code = format!(
        "{}-DESIGN-{}",
        lbs_code,
        if pri_type == "2" { "PMT" } else { "NOR" }
    );
    let target = body.template_code.as_deref();
    let tmpl_code = target.unwrap_or(&design_code);
    if let Some(code) = target {
        match sqlx::query_scalar::<_, i64>("SELECT 1 FROM TCNMLabelTmpl WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmPriType=? AND FTLtmStaUse='1'")
            .bind(code).bind(lbs_code).bind(pri_type).fetch_optional(&st.pool).await {
            Ok(Some(_)) => {},
            Ok(None) => return err_json(StatusCode::NOT_FOUND, "ไม่พบแบบป้ายที่เลือก"),
            Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }
    let tmpl_name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("Custom {} ({})", lbs_code, if pri_type == "2" { "Promo" } else { "Normal" }));

    let cur_size: Option<(i64, i64, f64, f64, f64, f64, Option<f64>, Option<f64>, f64, f64)> = sqlx::query_as(
        "SELECT COALESCE(FNLbsCol, 1), COALESCE(FNLbsRow, 1), FCLbsWidth, FCLbsHeight, COALESCE(FCLbsMagTop, 3.0), COALESCE(FCLbsMagLeft, 3.0), FCLbsMagBottom, FCLbsMagRight, COALESCE(FCLbsGapH, 2.0), COALESCE(FCLbsGapV, 2.0) FROM TCNMLabelSize WHERE FTLbsCode = ? LIMIT 1",
    )
    .bind(lbs_code)
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);

    let eff_top = body.mag_top.or_else(|| cur_size.as_ref().map(|r| r.4)).unwrap_or(3.0);
    let eff_left = body.mag_left.or_else(|| cur_size.as_ref().map(|r| r.5)).unwrap_or(3.0);
    let eff_bottom = body.mag_bottom.or_else(|| cur_size.as_ref().and_then(|r| r.6)).unwrap_or(eff_top);
    let eff_right = body.mag_right.or_else(|| cur_size.as_ref().and_then(|r| r.7)).unwrap_or(eff_left);

    let paper_size_opt = match body.paper_size.as_deref() {
        Some(raw) => Some(lookup_paper_size(&st.pool, Some(raw)).await.0),
        None => None,
    };
    let mut tx = match st.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let mut layout_val = body.layout.clone();
    let (ltm_mode, ppr_code_opt) = match upsert_preprint_from_layout(
        &mut *tx,
        target,
        lbs_code,
        &tmpl_name,
        body.cols.or_else(|| cur_size.as_ref().map(|r| r.0)).unwrap_or(1),
        body.rows.or_else(|| cur_size.as_ref().map(|r| r.1)).unwrap_or(1),
        body.width_mm.or_else(|| cur_size.as_ref().map(|r| r.2)).unwrap_or(100.0),
        body.height_mm.or_else(|| cur_size.as_ref().map(|r| r.3)).unwrap_or(38.0),
        eff_top,
        eff_left,
        eff_bottom,
        eff_right,
        body.gap_h.or_else(|| cur_size.as_ref().map(|r| r.8)).unwrap_or(2.0),
        body.gap_v.or_else(|| cur_size.as_ref().map(|r| r.9)).unwrap_or(2.0),
        &mut layout_val,
    )
    .await {
        Ok(result) => result,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let layout_str = serde_json::to_string(&layout_val).unwrap_or_else(|_| "{}".to_string());
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let res = if target.is_some() {
        sqlx::query("UPDATE TCNMLabelTmpl SET
            FTLtmOriginal = CASE WHEN FTLtmCode = ? THEN FTLtmOriginal ELSE COALESCE(FTLtmOriginal,
                json_object('layout', FTLtmLayout, 'mode', FTLtmMode, 'paper', FTPprCode)) END,
            FTLtmLayout=?, FTLtmMode=?, FTPprCode=?, FDLastUpdOn=?, FTLastUpdBy='DESIGNER'
            WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmPriType=? AND FTLtmStaUse='1'")
            .bind(&design_code).bind(&layout_str).bind(&ltm_mode).bind(&ppr_code_opt)
            .bind(&now).bind(tmpl_code).bind(lbs_code).bind(pri_type).execute(&mut *tx).await
    } else {
        sqlx::query(
            r#"
            INSERT INTO TCNMLabelTmpl
                (FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
            VALUES (?, ?, ?, ?, ?, '1', ?, ?, ?, 'DESIGNER', ?, 'DESIGNER')
            ON CONFLICT(FTLtmCode) DO UPDATE SET
                FTLbsCode = excluded.FTLbsCode,
                FTLtmPriType = excluded.FTLtmPriType,
                FTLtmName = excluded.FTLtmName,
                FTLtmLayout = excluded.FTLtmLayout,
                FTLtmStaUse = '1',
                FTLtmMode = excluded.FTLtmMode,
                FTPprCode = excluded.FTPprCode,
                FDLastUpdOn = excluded.FDLastUpdOn,
                FTLastUpdBy = 'DESIGNER'
            "#,
        )
        .bind(&tmpl_code)
        .bind(lbs_code)
        .bind(pri_type)
        .bind(&tmpl_name)
        .bind(&layout_str)
        .bind(&ltm_mode)
        .bind(&ppr_code_opt)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
    };
    match res {
        Ok(result) if result.rows_affected() == 0 => return err_json(StatusCode::NOT_FOUND, "ไม่พบแบบป้ายที่เลือก"),
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        _ => {}
    }

    let col_sql = if pri_type == "2" {
        r#"
        UPDATE TCNMLabelSize SET
            FTLbsTmplPmt = COALESCE(?, FTLbsTmplPmt),
            FTPprCode = COALESCE(?, FTPprCode),
            FTLbsPaperSize = COALESCE(?, FTLbsPaperSize),
            FCLbsPaperWidth = COALESCE(?, FCLbsPaperWidth),
            FCLbsPaperHeight = COALESCE(?, FCLbsPaperHeight),
            FNLbsCol = COALESCE(?, FNLbsCol),
            FNLbsRow = COALESCE(?, FNLbsRow),
            FCLbsMagTop = COALESCE(?, FCLbsMagTop),
            FCLbsMagLeft = COALESCE(?, FCLbsMagLeft),
            FCLbsMagBottom = COALESCE(?, FCLbsMagBottom),
            FCLbsMagRight = COALESCE(?, FCLbsMagRight),
            FCLbsGapH = COALESCE(?, FCLbsGapH),
            FCLbsGapV = COALESCE(?, FCLbsGapV),
            FCLbsWidth = COALESCE(?, FCLbsWidth),
            FCLbsHeight = COALESCE(?, FCLbsHeight),
            FDLastUpdOn = ?,
            FTLastUpdBy = 'DESIGNER'
        WHERE FTLbsCode = ?
        "#
    } else {
        r#"
        UPDATE TCNMLabelSize SET
            FTLbsTmplNor = COALESCE(?, FTLbsTmplNor),
            FTPprCode = COALESCE(?, FTPprCode),
            FTLbsPaperSize = COALESCE(?, FTLbsPaperSize),
            FCLbsPaperWidth = COALESCE(?, FCLbsPaperWidth),
            FCLbsPaperHeight = COALESCE(?, FCLbsPaperHeight),
            FNLbsCol = COALESCE(?, FNLbsCol),
            FNLbsRow = COALESCE(?, FNLbsRow),
            FCLbsMagTop = COALESCE(?, FCLbsMagTop),
            FCLbsMagLeft = COALESCE(?, FCLbsMagLeft),
            FCLbsMagBottom = COALESCE(?, FCLbsMagBottom),
            FCLbsMagRight = COALESCE(?, FCLbsMagRight),
            FCLbsGapH = COALESCE(?, FCLbsGapH),
            FCLbsGapV = COALESCE(?, FCLbsGapV),
            FCLbsWidth = COALESCE(?, FCLbsWidth),
            FCLbsHeight = COALESCE(?, FCLbsHeight),
            FDLastUpdOn = ?,
            FTLastUpdBy = 'DESIGNER'
        WHERE FTLbsCode = ?
        "#
    };
    let size_result = sqlx::query(col_sql)
        .bind(if target.is_none() { Some(tmpl_code) } else { None })
        .bind(&ppr_code_opt)
        .bind(paper_size_opt)
        .bind(body.paper_width_mm.filter(|v| *v >= 30.0))
        .bind(body.paper_height_mm.filter(|v| *v >= 30.0))
        .bind(body.cols.filter(|v| *v >= 1))
        .bind(body.rows.filter(|v| *v >= 1))
        .bind(body.mag_top.filter(|v| *v >= 0.0))
        .bind(body.mag_left.filter(|v| *v >= 0.0))
        .bind(body.mag_bottom.filter(|v| *v >= 0.0))
        .bind(body.mag_right.filter(|v| *v >= 0.0))
        .bind(body.gap_h.filter(|v| *v >= 0.0))
        .bind(body.gap_v.filter(|v| *v >= 0.0))
        .bind(body.width_mm.filter(|v| *v >= 5.0))
        .bind(body.height_mm.filter(|v| *v >= 5.0))
        .bind(&now)
        .bind(lbs_code)
        .execute(&mut *tx)
        .await;
    if let Err(e) = size_result {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    if let Err(e) = tx.commit().await {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({
        "ok": true,
        "tmplCode": tmpl_code,
        "lbsCode": lbs_code,
        "priType": pri_type
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAsTemplateReq {
    pub source_lbs_code: String,
    pub new_lbs_code: Option<String>,
    pub new_lbs_name: String,
    pub pri_type: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub cols: Option<i64>,
    pub rows: Option<i64>,
    pub mag_top: Option<f64>,
    pub mag_left: Option<f64>,
    pub mag_bottom: Option<f64>,
    pub mag_right: Option<f64>,
    pub gap_h: Option<f64>,
    pub gap_v: Option<f64>,
    pub paper_size: Option<String>,
    pub paper_width_mm: Option<f64>,
    pub paper_height_mm: Option<f64>,
    pub layout: Value,
}

pub async fn post_save_as_template(
    State(st): State<AppState>,
    Json(body): Json<SaveAsTemplateReq>,
) -> Response {
    let source_code = body.source_lbs_code.trim();
    let new_name = body.new_lbs_name.trim();
    if new_name.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุชื่อแบบใหม่");
    }

    let pri_type = if body.pri_type.trim() == "2" { "2" } else { "1" };

    // Inherit physical dimensions & sheet grid from source format (unless explicit custom values are supplied)
    let src_row: Option<(String, f64, f64, i64, i64, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>, Option<i64>, Option<String>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT FTLbsGroup, FCLbsWidth, FCLbsHeight, COALESCE(FNLbsCol, 1), COALESCE(FNLbsRow, 1), FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight FROM TCNMLabelSize WHERE FTLbsCode = ? LIMIT 1",
    )
    .bind(source_code)
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);

    let layout_w = body.layout.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
    let layout_h = body.layout.get("height").and_then(|v| v.as_f64()).unwrap_or(38.0);
    let req_w = body.width_mm.filter(|v| *v >= 10.0);
    let req_h = body.height_mm.filter(|v| *v >= 10.0);

    let src_paper_size = src_row.as_ref().and_then(|r| r.13.as_deref());
    let (paper_size, def_pw, def_ph, _) =
        lookup_paper_size(&st.pool, body.paper_size.as_deref().or(src_paper_size)).await;
    let paper_w = body
        .paper_width_mm
        .filter(|v| *v >= 30.0)
        .or_else(|| if body.paper_size.is_none() { src_row.as_ref().and_then(|r| r.14) } else { None })
        .unwrap_or(def_pw);
    let paper_h = body
        .paper_height_mm
        .filter(|v| *v >= 30.0)
        .or_else(|| if body.paper_size.is_none() { src_row.as_ref().and_then(|r| r.15) } else { None })
        .unwrap_or(def_ph);

    let mag_top = body.mag_top.or_else(|| src_row.as_ref().and_then(|r| r.5)).unwrap_or(3.0).max(0.0);
    let mag_left = body.mag_left.or_else(|| src_row.as_ref().and_then(|r| r.6)).unwrap_or(3.0).max(0.0);
    let mag_bottom = body.mag_bottom.or_else(|| src_row.as_ref().and_then(|r| r.7)).unwrap_or(mag_top).max(0.0);
    let mag_right = body.mag_right.or_else(|| src_row.as_ref().and_then(|r| r.8)).unwrap_or(mag_left).max(0.0);
    let gap_h = body.gap_h.or_else(|| src_row.as_ref().and_then(|r| r.9)).unwrap_or(2.0).max(0.0);
    let gap_v = body.gap_v.or_else(|| src_row.as_ref().and_then(|r| r.10)).unwrap_or(2.0).max(0.0);

    let (lbs_group, width_mm, height_mm, mut cols, mut rows, qty_nor, qty_pmt) = match (src_row.as_ref(), req_w, req_h) {
        (Some(sr), Some(cw), Some(ch))
            if (cw - sr.1).abs() > 0.05 || (ch - sr.2).abs() > 0.05 || body.paper_size.is_some() =>
        {
            let usable_w = (paper_w - mag_left - mag_right).max(10.0);
            let usable_h = (paper_h - mag_top - mag_bottom).max(10.0);
            let c = (((usable_w + gap_h) / (cw + gap_h)).floor() as i64).max(1);
            let r = (((usable_h + gap_v) / (ch + gap_v)).floor() as i64).max(1);
            (sr.0.clone(), cw, ch, c, r, c * r, c * r)
        }
        (Some(sr), _, _) => (
            sr.0.clone(),
            sr.1,
            sr.2,
            sr.3,
            sr.4,
            sr.11.unwrap_or(sr.3 * sr.4),
            sr.12.unwrap_or(sr.3 * sr.4),
        ),
        (None, cw_opt, ch_opt) => {
            let cw = cw_opt.unwrap_or(layout_w);
            let ch = ch_opt.unwrap_or(layout_h);
            let usable_w = (paper_w - mag_left - mag_right).max(10.0);
            let usable_h = (paper_h - mag_top - mag_bottom).max(10.0);
            let c = (((usable_w + gap_h) / (cw + gap_h)).floor() as i64).max(1);
            let r = (((usable_h + gap_v) / (ch + gap_v)).floor() as i64).max(1);
            ("1".to_string(), cw, ch, c, r, c * r, c * r)
        }
    };
    if let Some(c) = body.cols.filter(|v| *v >= 1) {
        cols = c;
    }
    if let Some(r) = body.rows.filter(|v| *v >= 1) {
        rows = r;
    }
    let final_qty_nor = if body.cols.is_some() || body.rows.is_some() { cols * rows } else { qty_nor };
    let final_qty_pmt = if body.cols.is_some() || body.rows.is_some() { cols * rows } else { qty_pmt };

    let raw_code = body.new_lbs_code.as_deref().unwrap_or("").trim();
    let sanitized: String = raw_code
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect();
    let new_lbs_code = if !sanitized.is_empty() {
        sanitized.to_uppercase()
    } else {
        let suffix = &gen_guid()[..4];
        format!("CUS-{}x{}-{}", width_mm.round() as i64, height_mm.round() as i64, suffix)
    };

    let tmpl_code = format!(
        "{}-DESIGN-{}",
        new_lbs_code,
        if pri_type == "2" { "PMT" } else { "NOR" }
    );

    let mut layout_val = body.layout.clone();
    if let Some(obj) = layout_val.as_object_mut() {
        obj.insert("lbsCode".to_string(), json!(new_lbs_code));
        obj.insert("name".to_string(), json!(new_name));
        obj.insert("width".to_string(), json!(width_mm));
        obj.insert("height".to_string(), json!(height_mm));
        obj.insert("customDesign".to_string(), json!(true));
    }
    let (ltm_mode, ppr_code_opt) = match upsert_preprint_from_layout(
        &st.pool,
        None,
        &new_lbs_code,
        new_name,
        cols,
        rows,
        width_mm,
        height_mm,
        mag_top,
        mag_left,
        mag_bottom,
        mag_right,
        gap_h,
        gap_v,
        &mut layout_val,
    )
    .await {
        Ok(result) => result,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let layout_str = serde_json::to_string(&layout_val).unwrap_or_else(|_| "{}".to_string());
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tmpl_nor = if pri_type == "1" { Some(tmpl_code.as_str()) } else { None };
    let tmpl_pmt = if pri_type == "2" { Some(tmpl_code.as_str()) } else { None };

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNMLabelSize
            (FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FTPprCode, FTLbsRmk, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'CUSTOM', '1', ?, ?, ?, ?, ?, ?, 'DESIGNER', ?, 'DESIGNER')
        ON CONFLICT(FTLbsCode) DO UPDATE SET
            FTLbsName = excluded.FTLbsName,
            FCLbsWidth = excluded.FCLbsWidth,
            FCLbsHeight = excluded.FCLbsHeight,
            FNLbsCol = excluded.FNLbsCol,
            FNLbsRow = excluded.FNLbsRow,
            FCLbsMagTop = excluded.FCLbsMagTop,
            FCLbsMagLeft = excluded.FCLbsMagLeft,
            FCLbsMagBottom = excluded.FCLbsMagBottom,
            FCLbsMagRight = excluded.FCLbsMagRight,
            FCLbsGapH = excluded.FCLbsGapH,
            FCLbsGapV = excluded.FCLbsGapV,
            FNLbsQtyPageNor = excluded.FNLbsQtyPageNor,
            FNLbsQtyPagePmt = excluded.FNLbsQtyPagePmt,
            FTLbsPaperSize = excluded.FTLbsPaperSize,
            FCLbsPaperWidth = excluded.FCLbsPaperWidth,
            FCLbsPaperHeight = excluded.FCLbsPaperHeight,
            FTPprCode = COALESCE(excluded.FTPprCode, TCNMLabelSize.FTPprCode),
            FTLbsTmplNor = COALESCE(excluded.FTLbsTmplNor, TCNMLabelSize.FTLbsTmplNor),
            FTLbsTmplPmt = COALESCE(excluded.FTLbsTmplPmt, TCNMLabelSize.FTLbsTmplPmt),
            FTLbsStaUse = '1',
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'DESIGNER'
        "#,
    )
    .bind(&new_lbs_code)
    .bind(&lbs_group)
    .bind(new_name)
    .bind(width_mm)
    .bind(height_mm)
    .bind(cols)
    .bind(rows)
    .bind(mag_top)
    .bind(mag_left)
    .bind(mag_bottom)
    .bind(mag_right)
    .bind(gap_h)
    .bind(gap_v)
    .bind(final_qty_nor)
    .bind(final_qty_pmt)
    .bind(tmpl_nor)
    .bind(tmpl_pmt)
    .bind(&paper_size)
    .bind(paper_w)
    .bind(paper_h)
    .bind(&ppr_code_opt)
    .bind(source_code)
    .bind(&now)
    .bind(&now)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNMLabelTmpl
            (FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
        VALUES (?, ?, ?, ?, ?, '1', ?, ?, ?, 'DESIGNER', ?, 'DESIGNER')
        ON CONFLICT(FTLtmCode) DO UPDATE SET
            FTLbsCode = excluded.FTLbsCode,
            FTLtmPriType = excluded.FTLtmPriType,
            FTLtmName = excluded.FTLtmName,
            FTLtmLayout = excluded.FTLtmLayout,
            FTLtmStaUse = '1',
            FTLtmMode = excluded.FTLtmMode,
            FTPprCode = excluded.FTPprCode,
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'DESIGNER'
        "#,
    )
    .bind(&tmpl_code)
    .bind(&new_lbs_code)
    .bind(pri_type)
    .bind(new_name)
    .bind(&layout_str)
    .bind(&ltm_mode)
    .bind(&ppr_code_opt)
    .bind(&now)
    .bind(&now)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({
        "ok": true,
        "lbsCode": new_lbs_code,
        "lbsName": new_name,
        "tmplCode": tmpl_code,
        "priType": pri_type,
        "layout": layout_val
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCustomFormatReq {
    pub lbs_code: String,
    pub allow_built_in: Option<bool>,
}

pub async fn post_delete_custom_format(
    State(st): State<AppState>,
    Json(body): Json<DeleteCustomFormatReq>,
) -> Response {
    let lbs_code = body.lbs_code.trim();
    if lbs_code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lbsCode is required");
    }

    if !body.allow_built_in.unwrap_or(false) {
        // Only allow deleting user-created custom formats when called from Design Studio
        let is_custom: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT FTLbsHhtCode FROM TCNMLabelSize WHERE FTLbsCode = ? AND (FTLbsHhtCode = 'CUSTOM' OR FTCreateBy IN ('DESIGNER', 'USER_CONFIG')) LIMIT 1",
        )
        .bind(lbs_code)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        if is_custom.is_none() {
            return err_json(
                StatusCode::BAD_REQUEST,
                "ไม่สามารถลบรูปแบบมาตรฐานของระบบได้ (ลบได้เฉพาะแบบที่สร้างขึ้นเอง)",
            );
        }
    }

    // Collect any linked FTPprCode before deleting so unused Pre-Print vault entries get cleaned up too
    let mut linked_preprints: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(rows) = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT FTPprCode FROM TCNMLabelSize WHERE FTLbsCode = ? AND COALESCE(FTPprCode, '') != ''",
    )
    .bind(lbs_code)
    .fetch_all(&st.pool)
    .await
    {
        for (p,) in rows {
            if let Some(code) = p.filter(|s| !s.trim().is_empty()) {
                linked_preprints.insert(code);
            }
        }
    }
    if let Ok(rows) = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT FTPprCode FROM TCNMLabelTmpl WHERE FTLbsCode = ? AND COALESCE(FTPprCode, '') != ''",
    )
    .bind(lbs_code)
    .fetch_all(&st.pool)
    .await
    {
        for (p,) in rows {
            if let Some(code) = p.filter(|s| !s.trim().is_empty()) {
                linked_preprints.insert(code);
            }
        }
    }

    let _ = sqlx::query("DELETE FROM TCNMLabelTmpl WHERE FTLbsCode = ?")
        .bind(lbs_code)
        .execute(&st.pool)
        .await;
    let _ = sqlx::query("DELETE FROM TCNMLabelPrinter WHERE FTLbsCode = ?")
        .bind(lbs_code)
        .execute(&st.pool)
        .await;
    let _ = sqlx::query("UPDATE TSysOption SET FTSopValue = '' WHERE FTSopCode = 'DefaultTemplate' AND FTSopValue = ?")
        .bind(lbs_code)
        .execute(&st.pool)
        .await;
    if let Err(e) = sqlx::query("DELETE FROM TCNMLabelSize WHERE FTLbsCode = ?")
        .bind(lbs_code)
        .execute(&st.pool)
        .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    // Delete orphan Pre-Print vault records no longer referenced by any label size or template
    for ppr_code in linked_preprints {
        let size_refs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM TCNMLabelSize WHERE FTPprCode = ?",
        )
        .bind(&ppr_code)
        .fetch_one(&st.pool)
        .await
        .unwrap_or(0);
        let tmpl_refs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM TCNMLabelTmpl WHERE FTPprCode = ?",
        )
        .bind(&ppr_code)
        .fetch_one(&st.pool)
        .await
        .unwrap_or(0);
        if size_refs == 0 && tmpl_refs == 0 {
            let _ = sqlx::query("DELETE FROM TCNMPreprint WHERE FTPprCode = ?")
                .bind(&ppr_code)
                .execute(&st.pool)
                .await;
        }
    }

    Json(json!({ "ok": true, "lbsCode": lbs_code })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetTemplateReq {
    pub template_code: Option<String>,
    pub lbs_code: String,
    pub pri_type: String,
}

pub async fn post_reset_template(
    State(st): State<AppState>,
    Json(body): Json<ResetTemplateReq>,
) -> Response {
    let lbs_code = body.lbs_code.trim();
    if lbs_code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lbsCode is required");
    }
    let pri_type = if body.pri_type.trim() == "2" { "2" } else { "1" };
    let tmpl_code = format!(
        "{}-DESIGN-{}",
        lbs_code,
        if pri_type == "2" { "PMT" } else { "NOR" }
    );
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(code) = body.template_code.as_deref().filter(|code| *code != tmpl_code) {
        return match sqlx::query("UPDATE TCNMLabelTmpl SET
            FTLtmLayout=json_extract(FTLtmOriginal, '$.layout'),
            FTLtmMode=json_extract(FTLtmOriginal, '$.mode'),
            FTPprCode=json_extract(FTLtmOriginal, '$.paper'),
            FTLtmOriginal=NULL, FDLastUpdOn=?, FTLastUpdBy='DESIGNER'
            WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmPriType=? AND FTLtmOriginal IS NOT NULL")
            .bind(&now).bind(code).bind(lbs_code).bind(pri_type).execute(&st.pool).await {
            Ok(result) if result.rows_affected() == 1 => Json(json!({"ok": true, "tmplCode": code})).into_response(),
            Ok(_) => err_json(StatusCode::NOT_FOUND, "ไม่พบต้นฉบับสำหรับแบบป้ายที่เลือก"),
            Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
    }

    if let Err(e) = sqlx::query("DELETE FROM TCNMLabelTmpl WHERE FTLtmCode = ?")
        .bind(&tmpl_code)
        .execute(&st.pool)
        .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let std_row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ? LIMIT 1",
    )
    .bind(lbs_code)
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);

    let restore_tmpl = std_row.and_then(|(nor, pmt)| if pri_type == "2" { pmt } else { nor });
    let col_sql = if pri_type == "2" {
        "UPDATE TCNMLabelSize SET FTLbsTmplPmt = ?, FDLastUpdOn = ?, FTLastUpdBy = 'DESIGNER' WHERE FTLbsCode = ? AND FTLbsTmplPmt = ?"
    } else {
        "UPDATE TCNMLabelSize SET FTLbsTmplNor = ?, FDLastUpdOn = ?, FTLastUpdBy = 'DESIGNER' WHERE FTLbsCode = ? AND FTLbsTmplNor = ?"
    };
    let _ = sqlx::query(col_sql)
        .bind(restore_tmpl)
        .bind(&now)
        .bind(lbs_code)
        .bind(&tmpl_code)
        .execute(&st.pool)
        .await;

    Json(json!({ "ok": true, "lbsCode": lbs_code, "priType": pri_type })).into_response()
}

// ─── /api/printers ───────────────────────────────────────────────────────────

pub async fn get_printers(State(st): State<AppState>) -> Response {
    let servers = sqlx::query_as::<_, TCNMPrinterServer>(
        "SELECT FTPsvCode, FTPsvName, FTPsvRmk, FTPsvStaUse, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNMPrinterServer WHERE FTPsvStaUse = '1' ORDER BY FTPsvCode ASC",
    )
    .fetch_all(&st.pool)
    .await;

    let servers = match servers {
        Ok(s) => s,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let printers = sqlx::query_as::<_, TCNMLabelPrinter>(
        "SELECT FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNMLabelPrinter WHERE FTLbpStaUse = '1'",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let label_sizes = sqlx::query_as::<_, TCNMLabelSize>(
        "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsStaUse = '1' ORDER BY FTLbsCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let size_map: HashMap<String, &TCNMLabelSize> = label_sizes
        .iter()
        .map(|s| (s.FTLbsCode.clone(), s))
        .collect();

    let server_vals: Vec<Value> = servers
        .into_iter()
        .map(|srv| {
            let srv_printers: Vec<Value> = printers
                .iter()
                .filter(|p| p.FTPsvCode == srv.FTPsvCode)
                .map(|p| {
                    let mut pv = serde_json::to_value(p).unwrap_or(json!({}));
                    if let Some(obj) = pv.as_object_mut() {
                        obj.insert(
                            "labelSize".to_string(),
                            size_map
                                .get(&p.FTLbsCode)
                                .and_then(|s| serde_json::to_value(s).ok())
                                .unwrap_or(Value::Null),
                        );
                    }
                    pv
                })
                .collect();
            let mut sv = serde_json::to_value(&srv).unwrap_or(json!({}));
            if let Some(obj) = sv.as_object_mut() {
                obj.insert("printers".to_string(), json!(srv_printers));
            }
            sv
        })
        .collect();

    let size_summary: Vec<Value> = label_sizes
        .iter()
        .map(|s| {
            json!({
                "FTLbsCode": s.FTLbsCode,
                "FTLbsName": s.FTLbsName,
                "FCLbsWidth": s.FCLbsWidth,
                "FCLbsHeight": s.FCLbsHeight
            })
        })
        .collect();

    Json(json!({
        "servers": server_vals,
        "labelSizes": size_summary
    }))
    .into_response()
}

pub async fn get_windows_printers() -> Response {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName System.Drawing
$def = (New-Object System.Drawing.Printing.PrinterSettings).PrinterName
$list = foreach ($name in [System.Drawing.Printing.PrinterSettings]::InstalledPrinters) {
  $ps = New-Object System.Drawing.Printing.PrinterSettings
  $ps.PrinterName = $name
  $disabledTrays = @{}
  $props = @(Get-PrinterProperty -PrinterName $name -ErrorAction SilentlyContinue)
  foreach ($p in $props) {
    if ($p.PropertyName -match 'OptTray(\d+)$') {
      $valStr = ("$($p.Value)").Trim().ToUpper()
      if (($p.Value -is [bool] -and -not $p.Value) -or $valStr -eq 'FALSE') {
        $disabledTrays["Tray $($matches[1])"] = $true
      }
    }
  }
  $trays = @(
    $ps.PaperSources |
      ForEach-Object { $_.SourceName } |
      Where-Object { $_ -and $_.Trim() -ne '' -and -not $disabledTrays.ContainsKey($_.Trim()) } |
      Select-Object -Unique
  )
  if ($trays -contains 'Automatically Select' -and $trays -contains 'Auto Select') {
    $trays = @($trays | Where-Object { $_ -ne 'Auto Select' })
  }
  [PSCustomObject]@{
    name = $name
    isDefault = ($name -eq $def)
    trays = $trays
  }
}
$list | ConvertTo-Json -Depth 4 -Compress
"#;
        let out = new_powershell_cmd()
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .await;

        if let Ok(o) = out {
            if o.status.success() {
                let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    let arr = if val.is_array() { val } else { json!([val]) };
                    return Json(json!({ "printers": arr })).into_response();
                }
            }
        }
    }
    Json(json!({ "printers": [] })).into_response()
}

// ─── /api/label-printers (ตั้งค่าไดร์เวอร์เครื่องพิมพ์ แยกปกติ/โปรโมชัน) ─────

#[derive(Deserialize, serde::Serialize)]
pub struct EventPrinterRoute {
    pub printer: String,
    pub tray: String,
    pub protocol: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLabelPrinterBody {
    pub template_routes: Option<std::collections::BTreeMap<String, EventPrinterRoute>>,
    pub event_routes: Option<std::collections::BTreeMap<String, EventPrinterRoute>>,
    pub lbs_code: String,
    pub server_code: Option<String>,
    pub normal_printer: Option<String>,
    pub normal_tray: Option<String>,
    pub normal_protocol: Option<String>,
    pub promo_printer: Option<String>,
    pub promo_tray: Option<String>,
    pub promo_protocol: Option<String>,
}

pub async fn get_label_printers(State(st): State<AppState>) -> Response {
    let server_row: Option<(String, String)> = sqlx::query_as(
        "SELECT FTPsvCode, FTPsvName FROM TCNMPrinterServer WHERE FTPsvStaUse = '1' ORDER BY FTPsvCode ASC LIMIT 1",
    )
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);
    let (server_code, server_name) = server_row.unwrap_or_else(|| ("LOCAL".to_string(), "เครื่องนี้ (Local)".to_string()));

    let sizes: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT FTLbsCode, FTLbsName, FTLbsGroup FROM TCNMLabelSize WHERE FTLbsStaUse = '1' ORDER BY FTLbsCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let rows: Vec<(String, String, Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT FTLbsCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol FROM TCNMLabelPrinter WHERE FTPsvCode = ? AND FTLbpStaUse = '1'",
    )
    .bind(&server_code)
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let template_rows:Vec<(String,String)>=sqlx::query_as("SELECT id,value FROM ABPrintSetting WHERE id LIKE 'templateRoutes:%'").fetch_all(&st.pool).await.unwrap_or_default();
    let event_rows:Vec<(String,String)>=sqlx::query_as("SELECT id,value FROM ABPrintSetting WHERE id LIKE 'eventRoutes:%'").fetch_all(&st.pool).await.unwrap_or_default();
    let mappings: Vec<Value> = sizes
        .into_iter()
        .map(|(code, name, grp)| {
            let nor = rows.iter().find(|r| r.0 == code && r.1 == "1");
            let pmt = rows.iter().find(|r| r.0 == code && r.1 == "2");
            json!({
                "templateRoutingSupported": true,
                "templateRoutes": template_rows.iter().find(|(id,_)|id==&format!("templateRoutes:{}:{}",server_code,code)).and_then(|(_,value)|serde_json::from_str::<Value>(value).ok()).unwrap_or(json!({})),
                "eventRoutingSupported": true,
                "eventRoutes": event_rows.iter().find(|(id,_)|id==&format!("eventRoutes:{}:{}",server_code,code)).and_then(|(_,value)|serde_json::from_str::<Value>(value).ok()).unwrap_or(json!({})),
                "lbsCode": code,
                "lbsName": name,
                "lbsGroup": grp,
                "normalPrinter": nor.and_then(|r| r.2.clone()).unwrap_or_default(),
                "normalTray": nor.and_then(|r| r.3.clone()).unwrap_or_default(),
                "normalProtocol": nor.map(|r| r.4.clone()).unwrap_or_else(|| "WINDOWS".to_string()),
                "promoPrinter": pmt.and_then(|r| r.2.clone()).unwrap_or_default(),
                "promoTray": pmt.and_then(|r| r.3.clone()).unwrap_or_default(),
                "promoProtocol": pmt.map(|r| r.4.clone()).unwrap_or_else(|| "WINDOWS".to_string()),
            })
        })
        .collect();

    Json(json!({
        "serverCode": server_code,
        "serverName": server_name,
        "mappings": mappings,
    }))
    .into_response()
}

pub async fn post_label_printers(
    State(st): State<AppState>,
    Json(body): Json<SaveLabelPrinterBody>,
) -> Response {
    if body.lbs_code.trim().is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lbsCode is required");
    }
    if let Some(events)=&body.event_routes {
        for (event,route) in events {
            if !["PRICE_CHG","PROMO_START","PROMO_END_REVERT","NEW_PDT"].contains(&event.as_str()) || route.printer.trim().is_empty() {
                return err_json(StatusCode::BAD_REQUEST,"เลือกเหตุการณ์และเครื่องพิมพ์ให้ครบ");
            }
        }
    }
    if let Some(routes)=&body.template_routes {
        for (code,route) in routes {
            let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM TCNMLabelTmpl WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmStaUse='1'").bind(code).bind(&body.lbs_code).fetch_one(&st.pool).await.unwrap_or(0);
            if exists != 1 || route.printer.trim().is_empty() {return err_json(StatusCode::BAD_REQUEST,"แบบป้ายหรือเครื่องพิมพ์ไม่ถูกต้อง");}
        }
    }
    let mut tx=match st.pool.begin().await {Ok(tx)=>tx,Err(e)=>return err_json(StatusCode::INTERNAL_SERVER_ERROR,e.to_string())};
    let server_code = body.server_code.unwrap_or_else(|| "LOCAL".to_string());
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let pairs = [
        (
            "1",
            body.normal_printer.unwrap_or_default(),
            body.normal_tray.unwrap_or_default(),
            body.normal_protocol.unwrap_or_else(|| "WINDOWS".to_string()),
        ),
        (
            "2",
            body.promo_printer.unwrap_or_default(),
            body.promo_tray.unwrap_or_default(),
            body.promo_protocol.unwrap_or_else(|| "WINDOWS".to_string()),
        ),
    ];

    for (pri_type, printer, tray, protocol) in pairs {
        if let Err(e) = sqlx::query(
            "INSERT OR REPLACE INTO TCNMLabelPrinter (FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, '1', ?, 'SYSTEM', ?, 'SYSTEM')",
        )
        .bind(&body.lbs_code)
        .bind(&server_code)
        .bind(pri_type)
        .bind(&printer)
        .bind(&tray)
        .bind(&protocol)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }

    if let Some(events)=body.event_routes {
        if let Err(e)=sqlx::query("INSERT INTO ABPrintSetting(id,value) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET value=excluded.value")
            .bind(format!("eventRoutes:{}:{}",server_code,body.lbs_code)).bind(serde_json::to_string(&events).unwrap()).execute(&mut *tx).await {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR,e.to_string());
        }
    }
    if let Some(routes)=body.template_routes {
        if let Err(e)=sqlx::query("INSERT INTO ABPrintSetting(id,value) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET value=excluded.value")
            .bind(format!("templateRoutes:{}:{}",server_code,body.lbs_code)).bind(serde_json::to_string(&routes).unwrap()).execute(&mut *tx).await {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR,e.to_string());
        }
    }
    if let Err(e)=tx.commit().await {return err_json(StatusCode::INTERNAL_SERVER_ERROR,e.to_string());}
    Json(json!({ "ok": true, "lbsCode": body.lbs_code })).into_response()
}

// ─── /api/config/* & /api/label-sizes/* (Full Config Management) ─────────────

pub async fn get_config_all(State(st): State<AppState>) -> Response {
    let label_sizes = sqlx::query_as::<_, TCNMLabelSize>(
        "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize ORDER BY FTLbsStaUse ASC, FTLbsCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let options = sqlx::query_as::<_, TCNSSysOption>(
        "SELECT FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq FROM TCNSSysOption WHERE FTSopCode <> 'AutoPrintAdminPin' AND FTSopCode NOT LIKE 'Master%' ORDER BY FNSopSeq ASC, FTSopCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let servers = sqlx::query_as::<_, TCNMPrinterServer>(
        "SELECT FTPsvCode, FTPsvName, FTPsvRmk, FTPsvStaUse, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNMPrinterServer ORDER BY FTPsvCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let paper_sizes = sqlx::query_as::<_, TCNMPaperSize>(
        "SELECT FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse FROM TCNMPaperSize WHERE FTPpsStaUse = '1' ORDER BY FNPpsSeq ASC, FTPpsCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let preprints = sqlx::query_as::<_, TCNMPreprint>(
        "SELECT FTPprCode, FTPprName, FTPprFileName, FTPpsCode, FCPprWidth, FCPprHeight, FNPprCol, FNPprRow, FCPprLabelW, FCPprLabelH, FCPprMagTop, FCPprMagLeft, FCPprMagBottom, FCPprMagRight, FCPprGapH, FCPprGapV, FNPprPageNo, FTPprScope, FCPprOpacity, FTPprSheetImg, FTPprCellImg, FTPprCropJson, FTPprStaUse FROM TCNMPreprint WHERE FTPprStaUse = '1' ORDER BY FTPprCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let label_fields = sqlx::query_as::<_, TCNMLabelField>(
        "SELECT FTLbfCode, FTLbfName, FTLbfType, FTLbfSample, FCLbfDefW, FCLbfDefH, FNLbfDefFont, FTLbfDefWeight, FTLbfDefColor, FNLbfSeq, FTLbfStaAdv, FTLbfStaUse FROM TCNMLabelField WHERE FTLbfStaUse = '1' ORDER BY FNLbfSeq ASC, FTLbfCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let lan_printers = fetch_lan_printers(&st.pool).await;
    let company = fetch_company_info(&st.pool).await;

    Json(json!({
        "labelSizes": label_sizes,
        "options": options,
        "servers": servers,
        "paperSizes": paper_sizes,
        "preprints": preprints,
        "labelFields": label_fields,
        "lanPrinters": lan_printers,
        "company": company,
        "platform": std::env::consts::OS,
        "supportsWindowsPrint": cfg!(target_os = "windows"),
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLabelSizeBody {
    pub lbs_code: String,
    pub lbs_name: String,
    pub lbs_group: Option<String>,
    pub width_mm: f64,
    pub height_mm: f64,
    pub cols: i64,
    pub rows: i64,
    pub mag_top: Option<f64>,
    pub mag_left: Option<f64>,
    pub mag_bottom: Option<f64>,
    pub mag_right: Option<f64>,
    pub gap_h: Option<f64>,
    pub gap_v: Option<f64>,
    pub qty_page_nor: Option<i64>,
    pub qty_page_pmt: Option<i64>,
    pub sta_use: Option<String>,
    pub is_new: Option<bool>,
    pub paper_size: Option<String>,
    pub paper_width_mm: Option<f64>,
    pub paper_height_mm: Option<f64>,
    pub offset_x: Option<f64>,
    pub offset_y: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelSizeOffset {
    lbs_code: String,
    offset_x: f64,
    offset_y: f64,
}

pub async fn post_label_size_offset(
    State(st): State<AppState>,
    Json(body): Json<LabelSizeOffset>,
) -> Response {
    if !(-30.0..=30.0).contains(&body.offset_x) || !(-30.0..=30.0).contains(&body.offset_y) {
        return err_json(StatusCode::BAD_REQUEST, "ค่าชดเชยต้องอยู่ระหว่าง -30 ถึง 30 มม.");
    }
    let result = sqlx::query(
        "UPDATE TCNMLabelSize SET FCLbsOffsetX=?, FCLbsOffsetY=?, FDLastUpdOn=?, FTLastUpdBy='USER_CONFIG' WHERE FTLbsCode=?",
    )
    .bind(body.offset_x)
    .bind(body.offset_y)
    .bind(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .bind(body.lbs_code.trim())
    .execute(&st.pool)
    .await;
    match result {
        Ok(row) if row.rows_affected() > 0 => Json(json!({ "ok": true })).into_response(),
        Ok(_) => err_json(StatusCode::NOT_FOUND, "ไม่พบขนาดป้าย"),
        Err(error) => err_json(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

pub async fn post_save_label_size(
    State(st): State<AppState>,
    Json(body): Json<SaveLabelSizeBody>,
) -> Response {
    let raw_code = body.lbs_code.trim();
    if raw_code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุรหัสขนาดป้าย (FTLbsCode)");
    }
    let name = body.lbs_name.trim();
    if name.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุชื่อขนาดป้าย (FTLbsName)");
    }
    if body.width_mm < 5.0 || body.height_mm < 5.0 {
        return err_json(StatusCode::BAD_REQUEST, "ความกว้างและความสูงต้องไม่น้อยกว่า 5 mm");
    }

    let is_new = body.is_new.unwrap_or(false);
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT FTLbsCode FROM TCNMLabelSize WHERE FTLbsCode = ? LIMIT 1")
            .bind(raw_code)
            .fetch_optional(&st.pool)
            .await
            .unwrap_or(None);

    if is_new && existing.is_some() {
        return err_json(
            StatusCode::BAD_REQUEST,
            format!("รหัสป้าย '{}' มีอยู่ในระบบแล้ว", raw_code),
        );
    }

    let grp = if body.lbs_group.as_deref() == Some("2") { "2" } else { "1" };
    let cols = body.cols.max(1);
    let rows = body.rows.max(1);
    let qty_nor = body.qty_page_nor.unwrap_or(cols * rows).max(1);
    let qty_pmt = body.qty_page_pmt.unwrap_or(cols * rows).max(1);
    let mag_top = body.mag_top.unwrap_or(3.0).max(0.0);
    let mag_left = body.mag_left.unwrap_or(3.0).max(0.0);
    let mag_bottom = body.mag_bottom.unwrap_or(mag_top).max(0.0);
    let mag_right = body.mag_right.unwrap_or(mag_left).max(0.0);
    let gap_h = body.gap_h.unwrap_or(2.0).max(0.0);
    let gap_v = body.gap_v.unwrap_or(2.0).max(0.0);
    let sta_use = if body.sta_use.as_deref() == Some("2") { "2" } else { "1" };
    let (paper_size, def_pw, def_ph, _) = lookup_paper_size(&st.pool, body.paper_size.as_deref()).await;
    let paper_w = body.paper_width_mm.filter(|v| *v >= 30.0).unwrap_or(def_pw);
    let paper_h = body.paper_height_mm.filter(|v| *v >= 30.0).unwrap_or(def_ph);
    let offset_x = body.offset_x.unwrap_or(0.0).clamp(-30.0, 30.0);
    let offset_y = body.offset_y.unwrap_or(0.0).clamp(-30.0, 30.0);
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if existing.is_some() {
        if let Err(e) = sqlx::query(
            r#"
            UPDATE TCNMLabelSize SET
                FTLbsGroup = ?,
                FTLbsName = ?,
                FCLbsWidth = ?,
                FCLbsHeight = ?,
                FNLbsCol = ?,
                FNLbsRow = ?,
                FCLbsMagTop = ?,
                FCLbsMagLeft = ?,
                FCLbsMagBottom = ?,
                FCLbsMagRight = ?,
                FCLbsGapH = ?,
                FCLbsGapV = ?,
                FNLbsQtyPageNor = ?,
                FNLbsQtyPagePmt = ?,
                FTLbsStaUse = ?,
                FTLbsPaperSize = ?,
                FCLbsPaperWidth = ?,
                FCLbsPaperHeight = ?,
                FCLbsOffsetX = ?,
                FCLbsOffsetY = ?,
                FDLastUpdOn = ?,
                FTLastUpdBy = 'USER_CONFIG'
            WHERE FTLbsCode = ?
            "#,
        )
        .bind(grp)
        .bind(name)
        .bind(body.width_mm)
        .bind(body.height_mm)
        .bind(cols)
        .bind(rows)
        .bind(mag_top)
        .bind(mag_left)
        .bind(mag_bottom)
        .bind(mag_right)
        .bind(gap_h)
        .bind(gap_v)
        .bind(qty_nor)
        .bind(qty_pmt)
        .bind(sta_use)
        .bind(paper_size)
        .bind(paper_w)
        .bind(paper_h)
        .bind(offset_x)
        .bind(offset_y)
        .bind(&now)
        .bind(raw_code)
        .execute(&st.pool)
        .await
        {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    } else {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO TCNMLabelSize
                (FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'CUSTOM', ?, ?, ?, ?, ?, ?, ?, 'USER_CONFIG', ?, 'USER_CONFIG')
            "#,
        )
        .bind(raw_code)
        .bind(grp)
        .bind(name)
        .bind(body.width_mm)
        .bind(body.height_mm)
        .bind(cols)
        .bind(rows)
        .bind(mag_top)
        .bind(mag_left)
        .bind(mag_bottom)
        .bind(mag_right)
        .bind(gap_h)
        .bind(gap_v)
        .bind(qty_nor)
        .bind(qty_pmt)
        .bind(sta_use)
        .bind(paper_size)
        .bind(paper_w)
        .bind(paper_h)
        .bind(offset_x)
        .bind(offset_y)
        .bind(&now)
        .bind(&now)
        .execute(&st.pool)
        .await
        {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }

    Json(json!({ "ok": true, "lbsCode": raw_code })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleLabelSizeBody {
    pub lbs_code: String,
    pub sta_use: String,
}

pub async fn post_toggle_label_size(
    State(st): State<AppState>,
    Json(body): Json<ToggleLabelSizeBody>,
) -> Response {
    let lbs_code = body.lbs_code.trim();
    if lbs_code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lbsCode is required");
    }
    let sta = if body.sta_use.trim() == "1" { "1" } else { "2" };
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Err(e) = sqlx::query(
        "UPDATE TCNMLabelSize SET FTLbsStaUse = ?, FDLastUpdOn = ?, FTLastUpdBy = 'USER_CONFIG' WHERE FTLbsCode = ?",
    )
    .bind(sta)
    .bind(&now)
    .bind(lbs_code)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({ "ok": true, "lbsCode": lbs_code, "staUse": sta })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysOptionItem {
    pub code: String,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigOptionsBody {
    pub options: Vec<SysOptionItem>,
    pub server_code: Option<String>,
    pub server_name: Option<String>,
}

pub async fn post_save_config_options(
    State(st): State<AppState>,
    Json(body): Json<SaveConfigOptionsBody>,
) -> Response {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    for item in body.options {
        let code = item.code.trim();
        if code == "AutoPrintAdminPin" {
            return err_json(StatusCode::FORBIDDEN, "ไม่สามารถเปลี่ยน PIN ผ่านค่าตั้งค่าทั่วไปได้");
        }
        if code.is_empty() {
            continue;
        }
        if code.starts_with("Master") || code == "DefaultBranch" {
            if code == "DefaultBranch" && item.value.trim() == crate::storage::configured_branch(&st.pool).await {
                continue;
            }
            return err_json(StatusCode::BAD_REQUEST, "เปลี่ยนแหล่งข้อมูลและสาขาได้จากหน้าตั้งค่าแหล่งข้อมูล StoreFront เท่านั้น");
        }
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO TCNSSysOption (FTSopCode, FTSopApp, FTSopName, FTSopValue, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
            VALUES (?, 'PrintBar', ?, ?, ?, 'USER_CONFIG', ?, 'USER_CONFIG')
            ON CONFLICT(FTSopCode) DO UPDATE SET
                FTSopValue = excluded.FTSopValue,
                FDLastUpdOn = excluded.FDLastUpdOn,
                FTLastUpdBy = 'USER_CONFIG'
            "#,
        )
        .bind(code)
        .bind(code)
        .bind(item.value.trim())
        .bind(&now)
        .bind(&now)
        .execute(&st.pool)
        .await
        {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }

    if let (Some(sc), Some(sn)) = (body.server_code.as_deref(), body.server_name.as_deref()) {
        let sc_t = sc.trim();
        let sn_t = sn.trim();
        if !sc_t.is_empty() && !sn_t.is_empty() {
            let _ = sqlx::query(
                "UPDATE TCNMPrinterServer SET FTPsvName = ?, FDLastUpdOn = ?, FTLastUpdBy = 'USER_CONFIG' WHERE FTPsvCode = ?",
            )
            .bind(sn_t)
            .bind(&now)
            .bind(sc_t)
            .execute(&st.pool)
            .await;
        }
    }

    Json(json!({ "ok": true })).into_response()
}

// ─── /api/print/windows-direct ───────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsDirectPrintBody {
    pub printer_name: String,
    pub tray_name: Option<String>,
    pub lbs_code: Option<String>,
    pub lbs_name: Option<String>,
    pub pri_type_label: Option<String>,
    pub test_mode: Option<bool>,
    pub page_images: Option<Vec<String>>,
    pub paper_size: Option<String>,
    pub paper_width_mm: Option<f64>,
    pub paper_height_mm: Option<f64>,
}

static WINDOWS_PRINT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn post_windows_direct_print(
    State(st): State<AppState>,
    Json(body): Json<WindowsDirectPrintBody>,
) -> Response {
    let printer_name = body.printer_name.trim().to_string();
    if printer_name.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุชื่อเครื่องพิมพ์ Windows");
    }

    #[cfg(target_os = "windows")]
    {
        let _print_guard = WINDOWS_PRINT_LOCK.lock().await;
        let tray_name = body.tray_name.unwrap_or_default();
        let lbs_code = body.lbs_code.unwrap_or_else(|| "TEST".to_string());
        let lbs_name = body.lbs_name.unwrap_or_else(|| lbs_code.clone());
        let pri_type_label = body.pri_type_label.unwrap_or_else(|| "ปกติ".to_string());
        let test_mode = body.test_mode.unwrap_or(false);
        let page_images = body.page_images.unwrap_or_default();
        let (configured_code, def_pw, def_ph, _) =
            lookup_paper_size(&st.pool, body.paper_size.as_deref()).await;
        let paper_size = body.paper_size.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or(&configured_code);
        let paper_width_mm = body.paper_width_mm.filter(|v| *v >= 10.0).unwrap_or(def_pw);
        let paper_height_mm = body.paper_height_mm.filter(|v| *v >= 10.0).unwrap_or(def_ph);

        let job_uid = uuid::Uuid::new_v4().to_string();
        let tmp_dir = std::env::temp_dir().join(format!("adabarprint_{}", job_uid));
        let _ = std::fs::create_dir_all(&tmp_dir);

        let payload_json = json!({
            "printerName": printer_name,
            "trayName": tray_name,
            "lbsCode": lbs_code,
            "lbsName": lbs_name,
            "priTypeLabel": pri_type_label,
            "testMode": test_mode,
            "pageImages": page_images,
            "paperSize": paper_size,
            "paperCode": paper_size,
            "paperWidthMm": paper_width_mm,
            "paperHeightMm": paper_height_mm,
        });
        let payload_path = tmp_dir.join("job.json");
        if let Err(e) = std::fs::write(&payload_path, serde_json::to_string(&payload_json).unwrap_or_default()) {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }

        let export_dir = crate::db::resolve_exports_dir();
        let _ = std::fs::create_dir_all(&export_dir);
        let ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let safe_code: String = lbs_code.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        let clean_export = std::fs::canonicalize(&export_dir)
            .unwrap_or(export_dir)
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_string();
        let pdf_out = std::path::PathBuf::from(clean_export)
            .join(format!("{}_{}_{}.pdf", ts, safe_code, &job_uid[..6]));

        let ps_script = r#"
$ErrorActionPreference = 'Stop'
$payloadPath = $args[0]
$pdfTarget = $args[1]
$cfg = Get-Content -Raw -Encoding UTF8 $payloadPath | ConvertFrom-Json
$lowerName = $cfg.printerName.ToLower()
$isVirtual = ($lowerName -like '*print to pdf*' -or $lowerName -like '*xps*' -or $lowerName -like '*adobe pdf*')
$pages = @($cfg.pageImages)
$resolvedTray = $(if ($cfg.trayName -and $cfg.trayName.Trim() -ne '') { $cfg.trayName.Trim() } else { 'Default' })

Add-Type -AssemblyName System.Drawing
$doc = New-Object System.Drawing.Printing.PrintDocument
$doc.PrinterSettings.PrinterName = $cfg.printerName
if (-not $doc.PrinterSettings.IsValid) {
  throw ("ไม่พบเครื่องพิมพ์: " + $cfg.printerName)
}
$doc.PrintController = New-Object System.Drawing.Printing.StandardPrintController
$doc.OriginAtMargins = $false

# ตั้งค่าขนาดกระดาษต่อ Job (A4 / B5 JIS / B5 ISO / Letter / Custom / Roll)
# APPLY_PAGE_SETTINGS
$ticketBin = ''
if ($matchedTray) {
  $resolvedTray = $matchedTray.SourceName
  $ticketBin = "DEVMODE:RawKind=" + $matchedTray.RawKind
}

$outFile = ''
if ($isVirtual) {
  $doc.PrinterSettings.PrintToFile = $true
  $outFile = $pdfTarget
  $doc.PrinterSettings.PrintFileName = $outFile
}

$doc.DocumentName = "AdaBarprint_" + $cfg.lbsCode
$script:idx = 0

$doc.add_QueryPageSettings({
  param($sender, $e)
  if ($targetPaper) { $e.PageSettings.PaperSize = $targetPaper }
  if ($matchedTray) { $e.PageSettings.PaperSource = $matchedTray }
})

$doc.add_PrintPage({
  param($sender, $e)
  $g = $e.Graphics
  $vcb = $g.VisibleClipBounds
  $pa = $e.PageSettings.PrintableArea
  $hmX = [double]$e.PageSettings.HardMarginX
  $hmY = [double]$e.PageSettings.HardMarginY

  if ($cfg.testMode -or $pages.Count -eq 0) {
    $fTitle = New-Object System.Drawing.Font('Tahoma', 16, [System.Drawing.FontStyle]::Bold)
    $fBody  = New-Object System.Drawing.Font('Tahoma', 11)
    $fSub   = New-Object System.Drawing.Font('Tahoma', 9.5)
    $b = [System.Drawing.Brushes]::Black
    $boxW = [Math]::Min(660.0, [Math]::Max(300.0, [double]$vcb.Width - 48.0))
    $boxX = [double]$vcb.X + (([double]$vcb.Width - $boxW) / 2.0)
    $boxY = [double]$vcb.Y + 24.0
    $g.DrawRectangle([System.Drawing.Pens]::Black, [float]$boxX, [float]$boxY, [float]$boxW, 250.0)
    $tx = [float]($boxX + 20.0)
    $g.DrawString("AdaSoft PrintBar - Test Print", $fTitle, $b, $tx, [float]($boxY + 18.0))
    $g.DrawString(("รหัสฉลาก: " + $cfg.lbsCode + " (" + $cfg.lbsName + ")"), $fBody, $b, $tx, [float]($boxY + 62.0))
    $g.DrawString(("ประเภทราคา: " + $cfg.priTypeLabel), $fBody, $b, $tx, [float]($boxY + 90.0))
    $g.DrawString(("เครื่องพิมพ์: " + $cfg.printerName), $fBody, $b, $tx, [float]($boxY + 118.0))
    $g.DrawString(("ถาดกระดาษ: " + $resolvedTray + $(if ($ticketBin) { " [$ticketBin]" } else { "" })), $fBody, $b, $tx, [float]($boxY + 146.0))
    $marginTxt = ("Printable Margin: Origin=(" + [Math]::Round($hmX * 0.254, 1) + "mm," + [Math]::Round($hmY * 0.254, 1) + "mm) Area=(" + [Math]::Round([double]$pa.Width * 0.254, 1) + "x" + [Math]::Round([double]$pa.Height * 0.254, 1) + "mm)")
    $g.DrawString($marginTxt, $fSub, $b, $tx, [float]($boxY + 176.0))
    $g.DrawString(("เวลาพิมพ์: " + (Get-Date -Format 'dd/MM/yyyy HH:mm:ss')), $fBody, $b, $tx, [float]($boxY + 206.0))
    $e.HasMorePages = $false
    return
  }

  $b64 = $pages[$script:idx]
  if ($b64 -match '^data:image/[^;]+;base64,(.+)$') { $b64 = $matches[1] }
  $bytes = [Convert]::FromBase64String($b64)
  $ms = New-Object System.IO.MemoryStream(,$bytes)
  $img = [System.Drawing.Image]::FromStream($ms)

  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  if ($cfg.paperCode -eq 'ROLL' -and $cfg.printerName -match '\bEZ[ -]?520\b') {
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
  }

  if ($isVirtual) {
    $bounds = $e.PageBounds
    $g.DrawImage($img, [float]0, [float]0, [float]$bounds.Width, [float]$bounds.Height)
  } else {
    # พิมพ์สเกลจริง 1:1 mm จากขอบกระดาษกายภาพ (ชดเชย HardMarginX/Y แทนการย่อเข้า PrintableArea)
    $g.PageUnit = [System.Drawing.GraphicsUnit]::Display
    $targetW = [float]($pwMm / 0.254)
    $targetH = [float]($phMm / 0.254)
    $dx = [float](-$hmX)
    $dy = [float](-$hmY)
    $g.DrawImage($img, $dx, $dy, $targetW, $targetH)
  }

  $img.Dispose()
  $ms.Dispose()
  $script:idx++
  $e.HasMorePages = ($script:idx -lt $pages.Count)
})

$doc.Print()
[PSCustomObject]@{ ok = $true; printer = $cfg.printerName; tray = $resolvedTray; ticketBin = $ticketBin; mode = 'GDI-DEVMODE'; v4Error = ''; outFile = $outFile } | ConvertTo-Json -Compress
"#;

        let script_path = tmp_dir.join("print.ps1");
        let mut ps_bytes = vec![0xEFu8, 0xBB, 0xBF];
        ps_bytes.extend_from_slice(ps_script.replace("# APPLY_PAGE_SETTINGS", include_str!("windows_page_settings.ps1")).as_bytes());
        if let Err(e) = std::fs::write(&script_path, &ps_bytes) {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }

        let out = new_powershell_cmd()
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-STA",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_path.to_str().unwrap_or_default(),
                payload_path.to_str().unwrap_or_default(),
                pdf_out.to_str().unwrap_or_default(),
            ])
            .output()
            .await;

        let _ = std::fs::remove_dir_all(&tmp_dir);

        return match out {
            Ok(o) if o.status.success() => {
                let raw = String::from_utf8_lossy(&o.stdout);
                let json_line = raw
                    .lines()
                    .rev()
                    .find(|l| l.trim().starts_with('{'))
                    .unwrap_or("{}");
                let parsed = serde_json::from_str::<Value>(json_line).unwrap_or_else(|_| json!({ "ok": true }));
                let v4_err = parsed.get("v4Error").and_then(|v| v.as_str()).unwrap_or("");
                if !v4_err.is_empty() {
                    eprintln!("[windows-direct] V4 PrintTicket fallback to GDI: {}", v4_err);
                }
                Json(json!({
                    "ok": true,
                    "printer": printer_name,
                    "tray": parsed.get("tray").and_then(|v| v.as_str()).unwrap_or("Default"),
                    "ticketBin": parsed.get("ticketBin").and_then(|v| v.as_str()).unwrap_or(""),
                    "mode": parsed.get("mode").and_then(|v| v.as_str()).unwrap_or(""),
                    "v4Error": v4_err,
                    "outputFile": parsed.get("outFile").and_then(|v| v.as_str()).unwrap_or(""),
                    "message": format!("ส่งพิมพ์ไปที่ {}{} เรียบร้อย", printer_name, if tray_name.is_empty() { "".to_string() } else { format!(" (ถาด {})", tray_name) }),
                })).into_response()
            }
            Ok(o) => {
                let err_txt = String::from_utf8_lossy(&o.stderr).trim().to_string();
                err_json(StatusCode::INTERNAL_SERVER_ERROR, format!("พิมพ์ไม่สำเร็จ: {}", err_txt))
            }
            Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
    }

    #[cfg(not(target_os = "windows"))]
    {
        err_json(StatusCode::NOT_IMPLEMENTED, "Windows Direct Print รองรับเฉพาะระบบปฏิบัติการ Windows")
    }
}

pub async fn post_printers(
    State(st): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let now = Utc::now().to_rfc3339();
    let user = resolve_default_user(&st.pool).await;
    let kind = body.get("type").and_then(|v| v.as_str()).unwrap_or("");

    if kind == "server" {
        let Some(code) = body.get("FTPsvCode").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) else {
            return err_json(StatusCode::BAD_REQUEST, "FTPsvCode is required");
        };
        let name = body.get("FTPsvName").and_then(|v| v.as_str());
        let rmk = body.get("FTPsvRmk").and_then(|v| v.as_str());
        let sta = body.get("FTPsvStaUse").and_then(|v| v.as_str()).unwrap_or("1");

        let res = sqlx::query(
            r#"
            INSERT INTO TCNMPrinterServer (FTPsvCode, FTPsvName, FTPsvRmk, FTPsvStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(FTPsvCode) DO UPDATE SET
                FTPsvName = excluded.FTPsvName,
                FTPsvRmk = excluded.FTPsvRmk,
                FTPsvStaUse = excluded.FTPsvStaUse,
                FDLastUpdOn = excluded.FDLastUpdOn,
                FTLastUpdBy = excluded.FTLastUpdBy
            "#,
        )
        .bind(code).bind(name).bind(rmk).bind(sta).bind(&now).bind(&user).bind(&now).bind(&user)
        .execute(&st.pool)
        .await;

        return match res {
            Ok(_) => Json(json!({ "ok": true, "message": format!("Printer server {} saved", code) })).into_response(),
            Err(e) => err_json(StatusCode::BAD_REQUEST, format!("DB Error: {}", e)),
        };
    }

    if kind == "labelPrinter" {
        let lbs = body.get("FTLbsCode").and_then(|v| v.as_str()).unwrap_or("");
        let psv = body.get("FTPsvCode").and_then(|v| v.as_str()).unwrap_or("");
        let pri = body.get("FTLbpPriType").and_then(|v| v.as_str()).unwrap_or("");
        if lbs.is_empty() || psv.is_empty() || pri.is_empty() {
            return err_json(StatusCode::BAD_REQUEST, "FTLbsCode, FTPsvCode, FTLbpPriType are required");
        }
        let printer = body.get("FTLbpPrinter").and_then(|v| v.as_str());
        let tray = body.get("FTLbpTray").and_then(|v| v.as_str());
        let protocol = body.get("FTLbpProtocol").and_then(|v| v.as_str()).unwrap_or("EZPL");
        let sta = body.get("FTLbpStaUse").and_then(|v| v.as_str()).unwrap_or("1");

        let res = sqlx::query(
            r#"
            INSERT INTO TCNMLabelPrinter (FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(FTLbsCode, FTPsvCode, FTLbpPriType) DO UPDATE SET
                FTLbpPrinter = excluded.FTLbpPrinter,
                FTLbpTray = excluded.FTLbpTray,
                FTLbpProtocol = excluded.FTLbpProtocol,
                FTLbpStaUse = excluded.FTLbpStaUse,
                FDLastUpdOn = excluded.FDLastUpdOn,
                FTLastUpdBy = excluded.FTLastUpdBy
            "#,
        )
        .bind(lbs).bind(psv).bind(pri).bind(printer).bind(tray).bind(protocol).bind(sta)
        .bind(&now).bind(&user).bind(&now).bind(&user)
        .execute(&st.pool)
        .await;

        return match res {
            Ok(_) => Json(json!({ "ok": true, "message": "Printer mapping saved" })).into_response(),
            Err(e) => err_json(StatusCode::BAD_REQUEST, format!("DB Error: {}", e)),
        };
    }

    err_json(StatusCode::BAD_REQUEST, "Invalid type")
}

// ─── /api/promotions ─────────────────────────────────────────────────────────

pub async fn get_promotions(State(st): State<AppState>) -> Response {
    let accd = &st.pool;
    #[derive(sqlx::FromRow)]
    struct AccdHd {
        FTPmhDocNo: String,
        FTBchCode: String,
        FTPmhName: Option<String>,
        FTPmhNameSlip: Option<String>,
        FTPmhRmk: Option<String>,
        FDPmhDStart: Option<String>,
        FDPmhDStop: Option<String>,
        FTPmhStaApv: Option<String>,
        FDCreateOn: Option<String>,
    }
    #[derive(sqlx::FromRow)]
    struct AccdDt {
        FTPmhDocNo: String,
        FTBchCode: String,
        FNPmdSeq: i64,
        FTPmdRefCode: Option<String>,
        FTPmdBarCode: Option<String>,
        FTPgtStaGetType: Option<String>,
        FCPgtGetvalue: Option<f64>,
    }

    let hds = sqlx::query_as::<_, AccdHd>(
        r#"
        SELECT hd.FTPmhDocNo, hd.FTBchCode,
               COALESCE(hdl.FTPmhName, hd.FTPmhDocNo) AS FTPmhName,
               hdl.FTPmhNameSlip, hdl.FTPmhRmk,
               CAST(hd.FDPmhDStart AS TEXT) AS FDPmhDStart,
               CAST(hd.FDPmhDStop AS TEXT) AS FDPmhDStop,
               hd.FTPmhStaApv,
               CAST(hd.FDCreateOn AS TEXT) AS FDCreateOn
        FROM TCNTPdtPmtHD hd
        LEFT JOIN TCNTPdtPmtHD_L hdl ON hd.FTPmhDocNo = hdl.FTPmhDocNo AND hd.FTBchCode = hdl.FTBchCode AND hdl.FNLngID = 1
        WHERE COALESCE(hd.FTPmhStaClosed, '2') != '1'
        ORDER BY hd.FDCreateOn DESC
        "#,
    )
    .fetch_all(accd)
    .await;

    let hds = match hds {
        Ok(h) => h,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let dts = sqlx::query_as::<_, AccdDt>(
        r#"
        SELECT dt.FTPmhDocNo, dt.FTBchCode, dt.FNPmdSeq, dt.FTPmdRefCode, dt.FTPmdBarCode,
               cg.FTPgtStaGetType, cg.FCPgtGetvalue
        FROM TCNTPdtPmtDT dt
        LEFT JOIN TCNTPdtPmtCG cg ON dt.FTPmhDocNo = cg.FTPmhDocNo AND dt.FTBchCode = cg.FTBchCode AND dt.FTPmdGrpName = cg.FTPmdGrpName
        ORDER BY dt.FNPmdSeq ASC
        "#,
    )
    .fetch_all(accd)
    .await
    .unwrap_or_default();

    let result: Vec<Value> = hds
        .into_iter()
        .map(|h| {
            let dets: Vec<Value> = dts
                .iter()
                .filter(|d| d.FTPmhDocNo == h.FTPmhDocNo && d.FTBchCode == h.FTBchCode)
                .map(|d| {
                    let cond: i64 = match d.FTPgtStaGetType.as_deref().unwrap_or("4") {
                        "1" => 1,
                        "2" => 2,
                        "3" => 3,
                        _ => 4,
                    };
                    json!({
                        "FTPmhCode": d.FTPmhDocNo,
                        "FNPmdSeq": d.FNPmdSeq,
                        "FTPdtCode": d.FTPmdRefCode,
                        "FTBarCode": d.FTPmdBarCode,
                        "FCPmdPricePmt": if cond == 3 { d.FCPgtGetvalue } else { None },
                        "FNPmdGetCond": cond,
                        "FCPmdGetValue": d.FCPgtGetvalue.unwrap_or(0.0),
                        "FTPmdStaUse": "1"
                    })
                })
                .collect();
            json!({
                "FTPmhCode": h.FTPmhDocNo,
                "FTPmhDocNo": h.FTPmhDocNo,
                "FTPmhName": h.FTPmhName,
                "FTPmhNameSlip": h.FTPmhNameSlip,
                "FTPmhRmk": h.FTPmhRmk,
                "FDPmhDStart": h.FDPmhDStart,
                "FDPmhDStop": h.FDPmhDStop,
                "FTPmhStaApv": h.FTPmhStaApv.unwrap_or_else(|| "1".to_string()),
                "FTPmhStaAll": "2",
                "FTPmhStaUse": "1",
                "FDCreateOn": h.FDCreateOn,
                "details": dets
            })
        })
        .collect();

    Json(result).into_response()
}

// ─── /api/products ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductsQuery {
    pub keyword: Option<String>,
    pub exact_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub pdt_from: Option<String>,
    pub pdt_to: Option<String>,
    pub grp_from: Option<String>,
    pub grp_to: Option<String>,
    pub type_from: Option<String>,
    pub type_to: Option<String>,
    pub brand_from: Option<String>,
    pub brand_to: Option<String>,
    pub model_from: Option<String>,
    pub model_to: Option<String>,
    pub bch_from: Option<String>,
    pub bch_to: Option<String>,
    pub doc_no_from: Option<String>,
    pub doc_no_to: Option<String>,
    pub doc_date_from: Option<String>,
    pub doc_date_to: Option<String>,
    pub price_group: Option<String>,
    pub agency: Option<String>,
    pub effective_date: Option<String>,
    pub price_type: Option<String>,
}

fn push_range_filter<'a>(
    qb: &mut sqlx::QueryBuilder<'a, sqlx::Sqlite>,
    col: &str,
    from_val: &'a str,
    to_val: &'a str,
) {
    if !from_val.is_empty() && !to_val.is_empty() {
        qb.push(" AND COALESCE(").push(col).push(", '') >= ").push_bind(from_val);
        qb.push(" AND COALESCE(").push(col).push(", '') <= ").push_bind(to_val);
    } else if !from_val.is_empty() {
        qb.push(" AND COALESCE(").push(col).push(", '') = ").push_bind(from_val);
    } else if !to_val.is_empty() {
        qb.push(" AND COALESCE(").push(col).push(", '') <= ").push_bind(to_val);
    }
}

pub async fn get_products(
    State(st): State<AppState>,
    Query(q): Query<ProductsQuery>,
) -> Response {
    let keyword = q.keyword.unwrap_or_default().trim().to_string();
    let limit = q.limit.unwrap_or(200).clamp(1, 5000);
    let pdt_from = q.pdt_from.unwrap_or_default().trim().to_string();
    let pdt_to = q.pdt_to.unwrap_or_default().trim().to_string();
    let grp_from = q.grp_from.unwrap_or_default().trim().to_string();
    let grp_to = q.grp_to.unwrap_or_default().trim().to_string();
    let type_from = q.type_from.unwrap_or_default().trim().to_string();
    let type_to = q.type_to.unwrap_or_default().trim().to_string();
    let brand_from = q.brand_from.unwrap_or_default().trim().to_string();
    let brand_to = q.brand_to.unwrap_or_default().trim().to_string();
    let model_from = q.model_from.unwrap_or_default().trim().to_string();
    let model_to = q.model_to.unwrap_or_default().trim().to_string();
    let bch_from = q.bch_from.unwrap_or_default().trim().to_string();
    let bch_to = q.bch_to.unwrap_or_default().trim().to_string();
    let doc_no_from = q.doc_no_from.unwrap_or_default().trim().to_string();
    let doc_no_to = q.doc_no_to.unwrap_or_default().trim().to_string();
    let doc_date_from = q.doc_date_from.unwrap_or_default().trim().to_string();
    let doc_date_to = q.doc_date_to.unwrap_or_default().trim().to_string();
    let price_group = q.price_group.unwrap_or_default().trim().to_string();
    let agency = q.agency.unwrap_or_default().trim().to_string();
    let eff_date = q.effective_date.unwrap_or_default().trim().to_string();
    let price_type = q.price_type.unwrap_or_default().trim().to_string();
    let eff_cmp = if eff_date.is_empty() {
        Utc::now().format("%Y-%m-%d").to_string()
    } else {
        eff_date.clone()
    };

    let accd = &st.pool;
    {
        #[derive(sqlx::FromRow)]
        struct AccdProductRow {
            FTPdtCode: String,
            FTBarCode: String,
            FTPdtName: String,
            FTPdtNameOth: Option<String>,
            FTPdtUnit: Option<String>,
            FTPdtBrand: Option<String>,
            FTPdtCgyName: Option<String>,
            FTPdtDim: Option<String>,
            FTBarStaUse: Option<String>,
            FCPdtPrice: Option<f64>,
        }

        let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
            r#"
            SELECT b.FTPdtCode,
                   b.FTBarCode,
                   COALESCE(pl.FTPdtName, b.FTPdtCode) AS FTPdtName,
                   pl.FTPdtNameOth,
                   ul.FTPunName AS FTPdtUnit,
                   bl.FTPbnName AS FTPdtBrand,
                   gl.FTPgpName AS FTPdtCgyName,
                   ps.FTPdtUnitDim AS FTPdtDim,
                   COALESCE(b.FTBarStaUse, '1') AS FTBarStaUse,
                   COALESCE((SELECT CAST(pr.FCPgdPriceRet AS REAL) FROM TCNTPdtPrice4PDT pr WHERE pr.FTPdtCode = b.FTPdtCode AND pr.FTPghDocType = '1' AND (pr.FTPunCode = b.FTPunCode OR COALESCE(pr.FTPunCode, '') = '')
            "#,
        );
        if !price_group.is_empty() {
            qb.push(" AND pr.FTPplCode = ").push_bind(&price_group);
        }
        if !eff_date.is_empty() {
            qb.push(" AND (COALESCE(pr.FDPghDStart, '') = '' OR substr(pr.FDPghDStart, 1, 10) <= ")
                .push_bind(&eff_cmp)
                .push(") AND (COALESCE(pr.FDPghDStop, '') = '' OR substr(pr.FDPghDStop, 1, 10) >= ")
                .push_bind(&eff_cmp)
                .push(")");
        }
        qb.push(
            r#"
                    ORDER BY CASE WHEN COALESCE(pr.FTPunCode, '') = '' THEN 1 ELSE 0 END, pr.FDPghDStart DESC, pr.FDCreateOn DESC LIMIT 1), 0.0) AS FCPdtPrice
            FROM TCNMPdtBar b
            LEFT JOIN TCNMPdt p ON p.FTPdtCode = b.FTPdtCode
            LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
            LEFT JOIN TCNMPdtUnit_L ul ON ul.FTPunCode = b.FTPunCode AND ul.FNLngID = 1
            LEFT JOIN TCNMPdtBrand_L bl ON bl.FTPbnCode = p.FTPbnCode AND bl.FNLngID = 1
            LEFT JOIN TCNMPdtGrp_L gl ON gl.FTPgpChain = p.FTPgpChain AND gl.FNLngID = 1
            LEFT JOIN TCNMPdtPackSize ps ON ps.FTPdtCode = b.FTPdtCode AND ps.FTPunCode = b.FTPunCode
            WHERE COALESCE(b.FTBarStaUse, '1') = '1'
            "#,
        );

        let exact = q.exact_code.as_deref().unwrap_or("").trim();
        if !exact.is_empty() {
            qb.push(" AND (b.FTBarCode = ").push_bind(exact)
                .push(" COLLATE NOCASE OR b.FTPdtCode = ").push_bind(exact).push(" COLLATE NOCASE)");
        } else if !keyword.is_empty() {
            let like = format!("%{}%", keyword);
            qb.push(" AND (b.FTPdtCode LIKE ")
                .push_bind(like.clone())
                .push(" OR b.FTBarCode LIKE ")
                .push_bind(like.clone())
                .push(" OR COALESCE(pl.FTPdtName, '') LIKE ")
                .push_bind(like.clone())
                .push(" OR COALESCE(pl.FTPdtNameOth, '') LIKE ")
                .push_bind(like)
                .push(")");
        }

        if !pdt_from.is_empty() && !pdt_to.is_empty() {
            qb.push(" AND b.FTPdtCode >= ").push_bind(&pdt_from);
            qb.push(" AND b.FTPdtCode <= ").push_bind(&pdt_to);
        } else if !pdt_from.is_empty() {
            let like_pdt = format!("%{}%", pdt_from);
            qb.push(" AND (b.FTPdtCode >= ")
                .push_bind(&pdt_from)
                .push(" OR COALESCE(pl.FTPdtName, '') LIKE ")
                .push_bind(like_pdt)
                .push(")");
        } else if !pdt_to.is_empty() {
            qb.push(" AND b.FTPdtCode <= ").push_bind(&pdt_to);
        }

        push_range_filter(&mut qb, "p.FTPgpChain", &grp_from, &grp_to);
        push_range_filter(&mut qb, "p.FTPtyCode", &type_from, &type_to);
        push_range_filter(&mut qb, "p.FTPbnCode", &brand_from, &brand_to);
        push_range_filter(&mut qb, "p.FTPmoCode", &model_from, &model_to);

        let has_doc_filter = !doc_no_from.is_empty() || !doc_no_to.is_empty() || !doc_date_from.is_empty() || !doc_date_to.is_empty();
        if has_doc_filter {
            qb.push(" AND (EXISTS (SELECT 1 FROM TCNTPdtPrice4PDT pr2 WHERE pr2.FTPdtCode = b.FTPdtCode");
            push_range_filter(&mut qb, "pr2.FTPghDocNo", &doc_no_from, &doc_no_to);
            if !doc_date_from.is_empty() {
                qb.push(" AND substr(COALESCE(pr2.FDPghDStart, ''), 1, 10) >= ").push_bind(&doc_date_from);
            }
            if !doc_date_to.is_empty() {
                qb.push(" AND substr(COALESCE(pr2.FDPghDStart, ''), 1, 10) <= ").push_bind(&doc_date_to);
            }
            if !price_group.is_empty() {
                qb.push(" AND pr2.FTPplCode = ").push_bind(&price_group);
            }
            if !agency.is_empty() {
                qb.push(" AND pr2.FTAgnCode = ").push_bind(&agency);
            }
            qb.push(") OR EXISTS (SELECT 1 FROM TCNTPdtPmtDT dt2 JOIN TCNTPdtPmtHD hd2 ON dt2.FTPmhDocNo = hd2.FTPmhDocNo AND dt2.FTBchCode = hd2.FTBchCode WHERE (dt2.FTPmdBarCode = b.FTBarCode OR dt2.FTPmdRefCode = b.FTPdtCode OR dt2.FTPmdBarCode = '*') AND COALESCE(hd2.FTPmhStaApv, '1') = '1' AND COALESCE(hd2.FTPmhStaClosed, '2') != '1'");
            push_range_filter(&mut qb, "hd2.FTPmhDocNo", &doc_no_from, &doc_no_to);
            push_range_filter(&mut qb, "hd2.FTBchCode", &bch_from, &bch_to);
            if !doc_date_from.is_empty() {
                qb.push(" AND substr(COALESCE(hd2.FDPmhDStart, ''), 1, 10) >= ").push_bind(&doc_date_from);
            }
            if !doc_date_to.is_empty() {
                qb.push(" AND substr(COALESCE(hd2.FDPmhDStart, ''), 1, 10) <= ").push_bind(&doc_date_to);
            }
            qb.push("))");
        } else {
            if !price_group.is_empty() {
                qb.push(" AND EXISTS (SELECT 1 FROM TCNTPdtPrice4PDT pr3 WHERE pr3.FTPdtCode = b.FTPdtCode AND pr3.FTPplCode = ")
                    .push_bind(&price_group)
                    .push(")");
            }
            if !agency.is_empty() {
                qb.push(" AND EXISTS (SELECT 1 FROM TCNTPdtPrice4PDT pr4 WHERE pr4.FTPdtCode = b.FTPdtCode AND pr4.FTAgnCode = ")
                    .push_bind(&agency)
                    .push(")");
            }
        }

        if price_type.eq_ignore_ascii_case("Promotion") {
            qb.push(" AND (EXISTS (SELECT 1 FROM TCNTPdtPmtDT dtp JOIN TCNTPdtPmtHD hdp ON dtp.FTPmhDocNo = hdp.FTPmhDocNo AND dtp.FTBchCode = hdp.FTBchCode WHERE (dtp.FTPmdBarCode = b.FTBarCode OR dtp.FTPmdRefCode = b.FTPdtCode OR dtp.FTPmdBarCode = '*') AND COALESCE(hdp.FTPmhStaApv, '1') = '1' AND COALESCE(hdp.FTPmhStaClosed, '2') != '1'");
            push_range_filter(&mut qb, "hdp.FTBchCode", &bch_from, &bch_to);
            qb.push(" AND (COALESCE(hdp.FDPmhDStart, '') = '' OR substr(hdp.FDPmhDStart, 1, 10) <= ")
                .push_bind(&eff_cmp)
                .push(") AND (COALESCE(hdp.FDPmhDStop, '') = '' OR substr(hdp.FDPmhDStop, 1, 10) >= ")
                .push_bind(&eff_cmp)
                .push(")) OR EXISTS (SELECT 1 FROM TCNTPmtGrpDT gd6 JOIN TCNTPmtHD hd6 ON gd6.FTAgnCode = hd6.FTAgnCode AND gd6.FTPmhDocNo = hd6.FTPmhDocNo WHERE (gd6.FTPmdRefCode = b.FTBarCode OR gd6.FTPmdRefCode = b.FTPdtCode OR gd6.FTPmdRefCode = '*') AND COALESCE(hd6.FTPmhStaActive, '1') = '1' AND (COALESCE(hd6.FDPmhDStart, '') = '' OR substr(hd6.FDPmhDStart, 1, 10) <= ")
                .push_bind(&eff_cmp)
                .push(") AND (COALESCE(hd6.FDPmhDStop, '') = '' OR substr(hd6.FDPmhDStop, 1, 10) >= ")
                .push_bind(&eff_cmp)
                .push(")))");
        }

        qb.push(" GROUP BY b.FTPdtCode, b.FTBarCode ORDER BY ");
        if !exact.is_empty() {
            qb.push("(b.FTBarCode = ").push_bind(exact).push(" COLLATE NOCASE) DESC, ");
        }
        qb.push("b.FTPdtCode ASC, b.FTBarCode ASC LIMIT ")
            .push_bind(limit).push(" OFFSET ").push_bind(q.offset.unwrap_or(0).max(0));

        let rows_res = qb.build_query_as::<AccdProductRow>().fetch_all(accd).await;

        let rows = match rows_res {
            Ok(r) => r,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string(), "products": [], "total": 0 }))).into_response(),
        };

        let total: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT FTBarCode) FROM TCNMPdtBar WHERE COALESCE(FTBarStaUse, '1') = '1'")
            .fetch_one(accd)
            .await
            .unwrap_or((0,));

        let labels = sqlx::query_as::<_, TCNMPdtLabel>(
            "SELECT FTPdtCode, FTBarCode, FTLbsCode, FNPdlQty, FTPdlSource, FTPdlStaDef FROM TCNMPdtLabel WHERE FTPdlStaDef = '1'",
        )
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default();

        let sizes = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize",
        )
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default();
        let size_map: HashMap<String, TCNMLabelSize> = sizes.into_iter().map(|s| (s.FTLbsCode.clone(), s)).collect();

        let img_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT FTImgRefID, FTImgObj FROM TCNMImgPdt WHERE FTImgTable = 'TCNMPdt' AND COALESCE(FTImgObj, '') != '' AND FTImgObj NOT LIKE '#%' ORDER BY FNImgSeq ASC",
        )
        .fetch_all(accd)
        .await
        .unwrap_or_default();
        let mut img_map: HashMap<String, String> = HashMap::new();
        for (ref_id, img_obj) in img_rows {
            img_map.entry(ref_id).or_insert(img_obj);
        }

        let out: Vec<Value> = rows
            .into_iter()
            .map(|r| {
                let symbology = if r.FTBarCode.len() == 13 && r.FTBarCode.chars().all(|c| c.is_ascii_digit()) {
                    "EAN13"
                } else if r.FTBarCode.len() == 8 && r.FTBarCode.chars().all(|c| c.is_ascii_digit()) {
                    "EAN8"
                } else {
                    "CODE128"
                };
                let p_labels: Vec<Value> = labels
                    .iter()
                    .filter(|l| l.FTPdtCode == r.FTPdtCode)
                    .take(1)
                    .map(|l| {
                        let mut lv = serde_json::to_value(l).unwrap_or(json!({}));
                        if let Some(obj) = lv.as_object_mut() {
                            obj.insert(
                                "labelSize".to_string(),
                                size_map.get(&l.FTLbsCode).and_then(|s| serde_json::to_value(s).ok()).unwrap_or(Value::Null),
                            );
                        }
                        lv
                    })
                    .collect();

                let p = r.FCPdtPrice.unwrap_or(0.0);
                let retail_price = p;
                let pdt_image = img_map.get(&r.FTPdtCode).cloned().unwrap_or_default();

                json!({
                    "FTPdtCode": r.FTPdtCode,
                    "FTPdtName": r.FTPdtName,
                    "FTPdtNameOth": r.FTPdtNameOth,
                    "FTPdtSku": r.FTPdtCode,
                    "FTPdtBrand": r.FTPdtBrand,
                    "FTPdtCgyName": r.FTPdtCgyName,
                    "FTPdtUnit": r.FTPdtUnit,
                    "FTPdtDim": r.FTPdtDim,
                    "FCPdtPrice": retail_price,
                    "FTPdtImg": pdt_image,
                    "FTPdtStaUse": "1",
                    "bars": [{
                        "FTPdbBarCode": r.FTBarCode,
                        "FTPdtCode": r.FTPdtCode,
                        "FTPdbSymbology": symbology,
                        "FTPdbStaPri": "1",
                        "FTPdbStaUse": r.FTBarStaUse.unwrap_or_else(|| "1".to_string())
                    }],
                    "labels": p_labels
                })
            })
            .collect();

        return Json(json!({ "products": out, "total": total.0 })).into_response();
    }
}

// ─── /api/history ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_history(
    State(st): State<AppState>,
    Query(q): Query<HistoryQuery>,
) -> Response {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let jobs = sqlx::query_as::<_, TCNTLabelJob>(
        "SELECT FTLjbJobID, FTLjbBchCode, FTLjbSource, FTLjbPriType, CAST(FDLjbFrom AS TEXT) AS FDLjbFrom, CAST(FDLjbTo AS TEXT) AS FDLjbTo, FTLjbStaVfy, FTLjbLbsOvr, FTLjbStatus, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNTLabelJob ORDER BY FDCreateOn DESC LIMIT ? OFFSET ?",
    )
    .bind(limit).bind(offset)
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelJob")
        .fetch_one(&st.pool)
        .await
        .unwrap_or((0,));

    let mut job_vals = Vec::new();
    for j in jobs {
        let dt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelJobDt WHERE FTLjbJobID = ?")
            .bind(&j.FTLjbJobID)
            .fetch_one(&st.pool)
            .await
            .unwrap_or((0,));
        let logs = sqlx::query_as::<_, TCNTLabelLog>(
            "SELECT FTLlgLogID, FTLjbJobID, FTLlgLbsCode, FTLlgPriTyp, FTLlgPsvCode, FTLlgPrinter, FNLlgQtyLabel, FNLlgQtyPage, CAST(FDLlgPriOn AS TEXT) AS FDLlgPriOn, FTLlgPriBy, FTLlgStatus, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNTLabelLog WHERE FTLjbJobID = ?",
        )
        .bind(&j.FTLjbJobID)
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default();

        let mut jv = serde_json::to_value(&j).unwrap_or(json!({}));
        if let Some(obj) = jv.as_object_mut() {
            obj.insert("_count".to_string(), json!({ "details": dt_count.0, "logs": logs.len() }));
            obj.insert("logs".to_string(), json!(logs));
        }
        job_vals.push(jv);
    }

    let total_pages = (total.0 + limit - 1) / limit;
    Json(json!({
        "jobs": job_vals,
        "total": total.0,
        "page": page,
        "limit": limit,
        "totalPages": total_pages
    }))
    .into_response()
}

pub async fn get_history_job(
    State(st): State<AppState>,
    Path(job_id): Path<String>,
) -> Response {
    if job_id.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "jobId is required");
    }

    let job = sqlx::query_as::<_, TCNTLabelJob>(
        "SELECT FTLjbJobID, FTLjbBchCode, FTLjbSource, FTLjbPriType, CAST(FDLjbFrom AS TEXT) AS FDLjbFrom, CAST(FDLjbTo AS TEXT) AS FDLjbTo, FTLjbStaVfy, FTLjbLbsOvr, FTLjbStatus, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNTLabelJob WHERE FTLjbJobID = ?",
    )
    .bind(&job_id)
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);

    let Some(job) = job else {
        return err_json(StatusCode::NOT_FOUND, "Job not found");
    };

    let details = sqlx::query_as::<_, TCNTLabelJobDt>(
        "SELECT FTLjbJobID, FNLjdSeq, FTLjdBarCode, FTLjdPdtCode, FTLjdName, FTLjdNameOth, FTLjdVolume, FTLjdSubDept, FTLjdReple, CAST(FDLjdPriDate AS TEXT) AS FDLjdPriDate, FTLjdPmtIntv, FTLjdPriStat, FTLjdSellUnit, FCLjdSetPrice, FCLjdSalePrice, FTLjdPhasing, FTLjdPerUnit, FTLjdCapFree, FTLjdCapName, FTLjdLabSize, FNLjdLabQty, FNLjdQtyOrg, FCLjdPmtCond, FCLjdPmtVal, CAST(FDLjdPmtFrom AS TEXT) AS FDLjdPmtFrom, CAST(FDLjdPmtTo AS TEXT) AS FDLjdPmtTo, FNLjdBarSeq, FNLjdPage, FTLjdPriTyp, FTLjdTmpl, FTLjdStaSel, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNTLabelJobDt WHERE FTLjbJobID = ? ORDER BY FNLjdSeq ASC",
    )
    .bind(&job_id)
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let logs = sqlx::query_as::<_, TCNTLabelLog>(
        "SELECT FTLlgLogID, FTLjbJobID, FTLlgLbsCode, FTLlgPriTyp, FTLlgPsvCode, FTLlgPrinter, FNLlgQtyLabel, FNLlgQtyPage, CAST(FDLlgPriOn AS TEXT) AS FDLlgPriOn, FTLlgPriBy, FTLlgStatus, CAST(FDCreateOn AS TEXT) AS FDCreateOn, FTCreateBy, CAST(FDLastUpdOn AS TEXT) AS FDLastUpdOn, FTLastUpdBy FROM TCNTLabelLog WHERE FTLjbJobID = ?",
    )
    .bind(&job_id)
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let mut job_val = serde_json::to_value(&job).unwrap_or(json!({}));
    if let Some(obj) = job_val.as_object_mut() {
        obj.insert("details".to_string(), json!(&details));
        obj.insert("logs".to_string(), json!(&logs));
    }

    Json(json!({
        "details": details,
        "logs": logs,
        "job": job_val
    }))
    .into_response()
}

// ─── /api/print/build-line ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildLineBody {
    pub barcode: String,
    pub pdt_code: Option<String>,
    pub qty: Option<i64>,
    pub bch_code: String,
    pub pri_type: String,
    pub verify: Option<bool>,
    pub override_lbs: Option<String>,
    pub effective_date: Option<String>,
    pub price_group: Option<String>,
}

pub(crate) async fn product_url_template(pool: &sqlx::SqlitePool) -> String {
    sqlx::query_as::<_, (Option<String>,)>(
            "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'UrlTemplate' LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|r| r.0)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://firster.com/product/{code}/?branch={bch}".to_string())
}

pub async fn post_build_line(
    State(st): State<AppState>,
    Json(body): Json<BuildLineBody>,
) -> Response {
    if body.barcode.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุบาร์โค้ดสินค้า");
    }
    let source_branch = crate::storage::configured_branch(&st.pool).await;
    if source_branch.is_empty() || body.bch_code.trim() != source_branch {
        return err_json(StatusCode::BAD_REQUEST, "สาขาของรายการพิมพ์ต้องตรงกับแหล่งข้อมูล StoreFront ที่ตั้งไว้");
    }
    let qty = body.qty.unwrap_or(1);
    if !(1..=10000).contains(&qty) {
        return err_json(StatusCode::BAD_REQUEST, "จำนวนป้ายต้องอยู่ระหว่าง 1 ถึง 10000");
    }
    let verify = body.verify.unwrap_or(true);
    let on_date = body
        .effective_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .and_then(|d| d.and_time(NaiveTime::from_hms_opt(12, 0, 0)?).and_utc().into())
        .unwrap_or_else(Utc::now);
    let on_date_str = on_date.format("%Y-%m-%d").to_string();

    let accd = &st.pool;
    {
        #[derive(sqlx::FromRow)]
        struct AccdBarPdt {
            FTPdtCode: String,
            FTBarCode: String,
            FTPunCode: String,
            FTPdtName: String,
            FTPdtNameOth: Option<String>,
            FTPdtUnit: Option<String>,
            FTPdtBrand: Option<String>,
            FTPdtCgyName: Option<String>,
            FTPdtDim: Option<String>,
            FCPdtPriceRET: Option<f64>,
            FCPdtPriceWHS: Option<f64>,
        }

        let pdt_filter: Option<&str> = body
            .pdt_code
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let bar_pdt = sqlx::query_as::<_, AccdBarPdt>(
            r#"
            SELECT b.FTPdtCode, b.FTBarCode, b.FTPunCode,
                   COALESCE(pl.FTPdtName, b.FTPdtCode) AS FTPdtName,
                   pl.FTPdtNameOth,
                   ul.FTPunName AS FTPdtUnit,
                   bl.FTPbnName AS FTPdtBrand,
                   gl.FTPgpName AS FTPdtCgyName,
                   ps.FTPdtUnitDim AS FTPdtDim,
                   ps.FCPdtPriceRET,
                   ps.FCPdtPriceWHS
            FROM TCNMPdtBar b
            LEFT JOIN TCNMPdt p ON p.FTPdtCode = b.FTPdtCode
            LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
            LEFT JOIN TCNMPdtUnit_L ul ON ul.FTPunCode = b.FTPunCode AND ul.FNLngID = 1
            LEFT JOIN TCNMPdtBrand_L bl ON bl.FTPbnCode = p.FTPbnCode AND bl.FNLngID = 1
            LEFT JOIN TCNMPdtGrp_L gl ON gl.FTPgpChain = p.FTPgpChain AND gl.FNLngID = 1
            LEFT JOIN TCNMPdtPackSize ps ON ps.FTPdtCode = b.FTPdtCode AND ps.FTPunCode = b.FTPunCode
            WHERE b.FTBarCode = ?
              AND (? IS NULL OR b.FTPdtCode = ?)
              AND COALESCE(b.FTBarStaUse, '1') = '1'
            LIMIT 1
            "#,
        )
        .bind(&body.barcode)
        .bind(pdt_filter)
        .bind(pdt_filter)
        .fetch_optional(accd)
        .await
        .unwrap_or(None);

        let Some(item) = bar_pdt else {
            return err_json(StatusCode::NOT_FOUND, format!("ไม่พบสินค้าสำหรับบาร์โค้ด {}", body.barcode));
        };

        // A missing price must remain distinct from an explicit zero price.
        #[derive(sqlx::FromRow)]
        struct AccdPriceRow {
            FCPgdPriceRet: Option<f64>,
            FCPgdPriceWhs: Option<f64>,
        }
        let ppl_code: Option<(Option<String>,)> = sqlx::query_as("SELECT FTPplCode FROM TCNMBranch WHERE FTBchCode = ?")
            .bind(&body.bch_code)
            .fetch_optional(accd)
            .await
            .unwrap_or(None);
        let bch_ppl = body
            .price_group
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| ppl_code.and_then(|r| r.0).unwrap_or_default());

        let p4 = sqlx::query_as::<_, AccdPriceRow>(
            r#"
            SELECT CAST(FCPgdPriceRet AS REAL) AS FCPgdPriceRet,
                   CAST(FCPgdPriceWhs AS REAL) AS FCPgdPriceWhs
            FROM TCNTPdtPrice4PDT
            WHERE FTPdtCode = ? AND (FTPunCode = ? OR FTPunCode = '')
              AND FTPghDocType = '1'
              AND (FTPplCode = ? OR FTPplCode = '')
              AND (COALESCE(FDPghDStart, '') = '' OR substr(FDPghDStart, 1, 10) <= ?)
              AND (COALESCE(FDPghDStop, '') = '' OR substr(FDPghDStop, 1, 10) >= ?)
            ORDER BY CASE WHEN FTPplCode = ? THEN 0 ELSE 1 END,
                     CASE WHEN COALESCE(FTPunCode, '') = '' THEN 1 ELSE 0 END,
                     FDPghDStart DESC, FTPghTStart DESC
            LIMIT 1
            "#,
        )
        .bind(&item.FTPdtCode)
        .bind(&item.FTPunCode)
        .bind(&bch_ppl)
        .bind(&on_date_str)
        .bind(&on_date_str)
        .bind(&bch_ppl)
        .fetch_optional(accd)
        .await
        .unwrap_or(None);

        // pri_type selects the label's normal/promotion layout, not wholesale pricing.
        let sale_price = p4.as_ref().and_then(|r| r.FCPgdPriceRet).or(item.FCPdtPriceRET);
        let Some(sale_price) = sale_price.filter(|p|p.is_finite() && *p>=0.0) else {
            return err_json(StatusCode::UNPROCESSABLE_ENTITY, "ไม่พบราคาขายที่ใช้ได้ กรุณาดาวน์โหลดข้อมูลหรือตรวจสอบราคาสินค้าก่อนพิมพ์");
        };
        let unit_price = sale_price;

        let promo = match resolve_accd_promotion(accd, &body.barcode, &item.FTPdtCode, on_date, verify).await {
            Ok(promo)=>promo,
            Err(error)=>return err_json(StatusCode::UNPROCESSABLE_ENTITY,error.to_string()),
        };
        let is_promo = promo.as_ref().map(|p| p.is_discount || p.is_gift).unwrap_or(false);
        let final_price = match &promo {
            Some(p) if p.is_discount => apply_promo(sale_price, p),
            _ => sale_price,
        };

        let def_label = sqlx::query_as::<_, TCNMPdtLabel>(
            "SELECT FTPdtCode, FTBarCode, FTLbsCode, FNPdlQty, FTPdlSource, FTPdlStaDef FROM TCNMPdtLabel WHERE FTPdtCode = ? AND FTBarCode = ? AND FTPdlStaDef = '1' LIMIT 1",
        )
        .bind(&item.FTPdtCode).bind(&body.barcode)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        let lbs_code = body
            .override_lbs
            .filter(|s| !s.is_empty())
            .or_else(|| def_label.map(|l| l.FTLbsCode))
            .unwrap_or_else(|| "SH-100x38".to_string());

        let label_size = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&lbs_code)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        let tmpl_code = if is_promo {
            label_size.as_ref().and_then(|s| s.FTLbsTmplPmt.clone()).unwrap_or_else(|| format!("{}-PMT", lbs_code))
        } else {
            label_size.as_ref().and_then(|s| s.FTLbsTmplNor.clone()).unwrap_or_else(|| format!("{}-NOR", lbs_code))
        };

        let dim_str = item.FTPdtDim.clone().unwrap_or_default();
        let unit_price_str = Regex::new(r"\d+(?:\.\d+)?")
            .ok()
            .and_then(|re| re.find(&dim_str).and_then(|m| m.as_str().parse::<f64>().ok()))
            .filter(|&v| v > 0.0 && final_price > 0.0)
            .map(|v| format!("{:.3} บาท/หน่วย", final_price / v))
            .unwrap_or_default();

        let bch_name: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT FTBchName FROM TCNMBranch_L WHERE FTBchCode = ? AND FNLngID = 1 LIMIT 1",
        )
        .bind(&body.bch_code)
        .fetch_optional(accd)
        .await
        .unwrap_or(None);

        let print_date = Utc::now().format("%d/%m/%Y").to_string();
        let pri_status = if is_promo { "2" } else { "1" };
        let pri_type_name = if is_promo { "โปรโมชั่น" } else { "ปกติ" };
        let promo_interval_str = promo
            .as_ref()
            .map(|p| promo_interval(p.dstart.as_deref(), p.dstop.as_deref()))
            .unwrap_or_default();

        let local_img: Option<(String,)> = sqlx::query_as(
            "SELECT FTImgObj FROM TCNMImgPdt WHERE FTImgRefID = ? AND FTImgTable = 'TCNMPdt' AND COALESCE(FTImgObj, '') != '' AND FTImgObj NOT LIKE '#%' ORDER BY FNImgSeq ASC LIMIT 1",
        )
        .bind(&item.FTPdtCode)
        .fetch_optional(accd)
        .await
        .unwrap_or(None);
        let product_image = local_img.map(|r| r.0).unwrap_or_default();

        let url_tmpl = product_url_template(&st.pool).await;
        let product_url = url_tmpl
            .replace("{code}", &item.FTPdtCode)
            .replace("{bch}", &body.bch_code);

        let ctx = json!({
            "product": {
                "code": item.FTPdtCode,
                "name": item.FTPdtName,
                "name_oth": item.FTPdtNameOth.as_deref().unwrap_or(""),
                "sku": item.FTPdtCode,
                "unit": item.FTPdtUnit.as_deref().unwrap_or(""),
                "brand": item.FTPdtBrand.as_deref().unwrap_or(""),
                "category": item.FTPdtCgyName.as_deref().unwrap_or(""),
                "barcode": item.FTBarCode,
                "dim": dim_str,
                "image": product_image,
                "sub_dept": "",
                "phasing": "",
                "reple_type": "",
                "reg_no": "",
                "url": product_url,
            },
            "price": {
                "value": final_price,
                "old_value": if is_promo { json!(sale_price) } else { Value::Null },
                "unit_price": unit_price,
                "type_name": pri_type_name,
            },
            "promotion": {
                "name": promo.as_ref().map(|p| p.name.as_str()).unwrap_or(""),
                "text": promo.as_ref().map(|p| p.cap_free.as_str()).unwrap_or(""),
                "free_text": promo.as_ref().map(|p| p.cap_name.as_str()).unwrap_or(""),
                "interval": promo_interval_str,
                "get_cond": promo.as_ref().map(|p| p.get_cond).unwrap_or(0),
                "is_discount": promo.as_ref().map(|p| p.is_discount).unwrap_or(false),
                "is_gift": promo.as_ref().map(|p| p.is_gift).unwrap_or(false),
            },
            "store": {
                "code": body.bch_code,
                "name": bch_name.and_then(|r| r.0).unwrap_or_default(),
            },
            "tag": {
                "print_date": print_date,
                "pri_status": pri_status,
                "selling_unit": item.FTPdtUnit.as_deref().unwrap_or(""),
                "pri_per_unit": unit_price_str,
                "lab_size": lbs_code,
            }
        });

        let promo_json = promo.as_ref().map(|p| {
            json!({
                "name": p.name,
                "text": p.cap_free,
                "freeText": p.cap_name,
                "interval": if p.dstart.is_some() && p.dstop.is_some() {
                    format!("เริ่ม {} ถึง {}", format_date_th(p.dstart.as_deref()), format_date_th(p.dstop.as_deref()))
                } else { String::new() },
                "getCond": p.get_cond,
                "isDiscount": p.is_discount,
                "isGift": p.is_gift,
            })
        });

        return Json(json!({
            "barcode": item.FTBarCode,
            "pdtCode": item.FTPdtCode,
            "pdtName": item.FTPdtName,
            "pdtNameOth": item.FTPdtNameOth.as_deref().unwrap_or(""),
            "sku": item.FTPdtCode,
            "brand": item.FTPdtBrand.as_deref().unwrap_or(""),
            "category": item.FTPdtCgyName.as_deref().unwrap_or(""),
            "dim": dim_str,
            "unit": item.FTPdtUnit.as_deref().unwrap_or(""),
            "subDept": "",
            "phasing": "",
            "repleType": "",
            "qty": qty,
            "priType": if is_promo { "2" } else { "1" },
            "lbsCode": lbs_code,
            "tmplCode": tmpl_code,
            "salePrice": sale_price,
            "setPrice": final_price,
            "isPromo": is_promo,
            "promo": promo_json,
            "ctx": ctx,
        }))
        .into_response()
    }
}

// ─── /api/print/create-job ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLine {
    pub barcode: String,
    pub qty: Option<i64>,
    pub ctx: Value,
    pub pri_type: String,
    pub lbs_code: String,
    pub sale_price: f64,
    pub set_price: Option<f64>,
    pub promo: Option<Value>,
    pub tmpl_code: String,
    pub pdt_code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobBody {
    pub lines: Vec<JobLine>,
    pub bch_code: String,
    pub source: Option<String>,
    pub r#override: Option<String>,
    pub start_col: Option<i64>,
    pub user: Option<String>,
    pub server_code: Option<String>,
    pub normal_printer: Option<String>,
    pub promo_printer: Option<String>,
    pub default_printer: Option<String>,
}

pub async fn post_create_job(
    State(st): State<AppState>,
    Json(body): Json<CreateJobBody>,
) -> Response {
    if body.lines.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "lines is required");
    }

    let job_id = gen_guid();
    let now = Utc::now().to_rfc3339();
    let created_by = match body.user.filter(|s| !s.trim().is_empty()) {
        Some(u) => u,
        None => resolve_default_user(&st.pool).await,
    };
    let first_pri_type = body.lines[0].pri_type.clone();

    let mut tx = match st.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO TCNTLabelJob (FTLjbJobID, FTLjbBchCode, FTLjbSource, FTLjbPriType, FDLjbFrom, FTLjbStaVfy, FTLjbLbsOvr, FTLjbStatus, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, '1', ?, '1', ?, ?, ?, ?)",
    )
    .bind(&job_id).bind(&body.bch_code).bind(body.source.as_deref().unwrap_or("1")).bind(&first_pri_type)
    .bind(&now).bind(&body.r#override).bind(&now).bind(&created_by).bind(&now).bind(&created_by)
    .execute(&mut *tx)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let mut group_map: BTreeMap<String, (String, String, i64, bool)> = BTreeMap::new();
    let mut seq: i64 = 0;

    for line in &body.lines {
        let qty = line.qty.unwrap_or(1).max(1);
        let is_promo = line.pri_type == "2" || line.promo.as_ref().map(|p| {
            p.get("is_discount").and_then(|v| v.as_bool()).unwrap_or(false)
                || p.get("is_gift").and_then(|v| v.as_bool()).unwrap_or(false)
        }).unwrap_or(false);

        let gkey = format!("{}|{}", line.lbs_code, line.pri_type);
        group_map
            .entry(gkey)
            .and_modify(|e| e.2 += qty)
            .or_insert((line.lbs_code.clone(), line.pri_type.clone(), qty, is_promo));

        let label_size = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&line.lbs_code)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);

        let cols = label_size.as_ref().map(|s| s.FNLbsCol).unwrap_or(1).max(1);
        let rows = label_size.as_ref().map(|s| s.FNLbsRow).unwrap_or(1).max(1);
        let per_page = if is_promo {
            label_size.as_ref().and_then(|s| s.FNLbsQtyPagePmt).unwrap_or(cols * rows)
        } else {
            label_size.as_ref().and_then(|s| s.FNLbsQtyPageNor).unwrap_or(cols * rows)
        }.max(1);

        let prod = line.ctx.get("product");
        let prom = line.ctx.get("promotion");
        let tag = line.ctx.get("tag");

        for _ in 0..qty {
            seq += 1;
            let bar_seq = ((body.start_col.unwrap_or(0) + (seq - 1)) % cols) + 1;
            let page_num = ((seq - 1) / per_page) + 1;

            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO TCNTLabelJobDt (
                    FTLjbJobID, FNLjdSeq, FTLjdBarCode, FTLjdPdtCode, FTLjdName, FTLjdNameOth,
                    FTLjdVolume, FTLjdSubDept, FTLjdReple, FDLjdPriDate, FTLjdPmtIntv, FTLjdPriStat,
                    FTLjdSellUnit, FCLjdSetPrice, FCLjdSalePrice, FTLjdPhasing, FTLjdPerUnit,
                    FTLjdCapFree, FTLjdCapName, FTLjdLabSize, FNLjdLabQty, FNLjdQtyOrg,
                    FNLjdBarSeq, FNLjdPage, FTLjdPriTyp, FTLjdTmpl, FTLjdStaSel,
                    FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?, '1', ?, ?, ?, ?)
                "#,
            )
            .bind(&job_id)
            .bind(seq)
            .bind(&line.barcode)
            .bind(&line.pdt_code)
            .bind(prod.and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(prod.and_then(|v| v.get("name_oth")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(prod.and_then(|v| v.get("dim")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(prod.and_then(|v| v.get("sub_dept")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(prod.and_then(|v| v.get("reple_type")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(&now)
            .bind(prom.and_then(|v| v.get("interval")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(if is_promo { "2" } else { "1" })
            .bind(tag.and_then(|v| v.get("selling_unit")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(line.set_price)
            .bind(line.sale_price)
            .bind(prod.and_then(|v| v.get("phasing")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(tag.and_then(|v| v.get("pri_per_unit")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(line.promo.as_ref().and_then(|v| v.get("free_text")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(line.promo.as_ref().and_then(|v| v.get("name")).and_then(|v| v.as_str()).unwrap_or(""))
            .bind(&line.lbs_code)
            .bind(qty)
            .bind(bar_seq)
            .bind(page_num)
            .bind(&line.pri_type)
            .bind(&line.tmpl_code)
            .bind(&now).bind(&created_by).bind(&now).bind(&created_by)
            .execute(&mut *tx)
            .await {
                return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            }
        }
    }

    let mut log_entries = Vec::new();
    for (_k, (lbs_code, pri_type, total_qty, is_promo)) in group_map {
        let label_size = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&lbs_code)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);

        let cols = label_size.as_ref().map(|s| s.FNLbsCol).unwrap_or(1).max(1);
        let rows = label_size.as_ref().map(|s| s.FNLbsRow).unwrap_or(1).max(1);
        let per_page = if is_promo {
            label_size.as_ref().and_then(|s| s.FNLbsQtyPagePmt).unwrap_or(cols * rows)
        } else {
            label_size.as_ref().and_then(|s| s.FNLbsQtyPageNor).unwrap_or(cols * rows)
        }.max(1);

        let qty_page = (total_qty + per_page - 1) / per_page;
        let log_id = gen_guid();
        let psv_code = body.server_code.as_deref().unwrap_or("LOCAL");
        let fallback_prt = body.default_printer.as_deref().unwrap_or("");
        let target_prt = if pri_type == "2" {
            body.promo_printer.as_deref().filter(|s| !s.is_empty()).unwrap_or(fallback_prt)
        } else {
            body.normal_printer.as_deref().filter(|s| !s.is_empty()).unwrap_or(fallback_prt)
        };

        if let Err(e) = sqlx::query(
            "INSERT INTO TCNTLabelLog (FTLlgLogID, FTLjbJobID, FTLlgLbsCode, FTLlgPriTyp, FTLlgPsvCode, FTLlgPrinter, FNLlgQtyLabel, FNLlgQtyPage, FDLlgPriOn, FTLlgPriBy, FTLlgStatus, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, ?, ?, ?)",
        )
        .bind(&log_id).bind(&job_id).bind(&lbs_code).bind(&pri_type)
        .bind(psv_code).bind(target_prt)
        .bind(total_qty).bind(qty_page).bind(&now).bind(&created_by).bind(&now).bind(&created_by).bind(&now).bind(&created_by)
        .execute(&mut *tx)
        .await {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }

        log_entries.push(json!({
            "lbsCode": lbs_code,
            "priType": pri_type,
            "totalQty": total_qty,
            "qtyPage": qty_page,
        }));
    }

    if let Err(e) = tx.commit().await {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({
        "jobId": job_id,
        "totalLabels": seq,
        "logs": log_entries
    }))
    .into_response()
}

// ─── /api/print/check & /api/print/send ──────────────────────────────────────

pub async fn post_print_check(Json(body): Json<Value>) -> Response {
    let ip = body.get("printerIp").and_then(|v| v.as_str()).unwrap_or("");
    if ip.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "printerIp จำเป็นต้องระบุ");
    }
    let port = body.get("printerPort").and_then(|v| v.as_u64()).unwrap_or(9100) as u16;
    let brand = body.get("brand").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");

    let (connected, message, latency_ms) = tcp_test(ip, port).await;
    Json(json!({
        "connected": connected,
        "ip": ip,
        "port": port,
        "brand": brand,
        "message": message,
        "latencyMs": latency_ms
    }))
    .into_response()
}

pub async fn post_print_send(
    State(st): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let ip = body.get("printerIp").and_then(|v| v.as_str()).unwrap_or("");
    let brand_str = body.get("brand").and_then(|v| v.as_str()).unwrap_or("");
    if ip.is_empty() || brand_str.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "printerIp และ brand จำเป็นต้องระบุ");
    }
    let Some(brand) = PrinterBrand::from_str(brand_str) else {
        return err_json(StatusCode::BAD_REQUEST, format!("brand ไม่ถูกต้อง: {}", brand_str));
    };

    let port = body.get("printerPort").and_then(|v| v.as_u64()).unwrap_or(9100) as u16;
    let test_mode = body.get("testMode").and_then(|v| v.as_bool()).unwrap_or(false);
    let preview_only = body.get("previewOnly").and_then(|v| v.as_bool()).unwrap_or(false);
    let options = body.get("options");
    let dpi = options.and_then(|o| o.get("dpi")).and_then(|v| v.as_u64()).unwrap_or(203) as u32;
    let speed = options.and_then(|o| o.get("speed")).and_then(|v| v.as_u64()).unwrap_or(3) as u32;
    let darkness = options.and_then(|o| o.get("darkness")).and_then(|v| v.as_u64()).unwrap_or(10) as u32;

    let command_data = if test_mode {
        let w = options.and_then(|o| o.get("labelWidth")).and_then(|v| v.as_f64()).unwrap_or(100.0);
        let h = options.and_then(|o| o.get("labelHeight")).and_then(|v| v.as_f64()).unwrap_or(38.0);
        generate_test_pattern(brand, dpi, w, h)
    } else if let Some(lines) = body.get("lines").and_then(|v| v.as_array()) {
        let mut groups: BTreeMap<(String,String,String), Vec<(Value, u32)>> = BTreeMap::new();
        for l in lines {
            if !l.get("selected").and_then(|v| v.as_bool()).unwrap_or(false) {
                continue;
            }
            let lbs = l.get("lbsCode").and_then(|v| v.as_str()).unwrap_or("SH-100x38");
            let pri = l.get("priType").and_then(|v| v.as_str()).unwrap_or("1");
            let qty = l.get("qty").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let ctx = l.get("ctx").cloned().unwrap_or(json!({}));
            groups.entry((lbs.to_string(),pri.to_string(),l.get("tmplCode").and_then(Value::as_str).unwrap_or("").to_string())).or_default().push((ctx, qty));
        }

        let templates = get_templates();
        let mut parts = Vec::new();
        for ((lbs_code, pri_type, explicit_template), lbls) in groups {
            let group_lbs = sqlx::query_as::<_, TCNMLabelSize>(
                "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
            )
            .bind(&lbs_code)
            .fetch_optional(&st.pool)
            .await
            .unwrap_or(None);

            let Some(group_lbs) = group_lbs else { return err_json(StatusCode::BAD_REQUEST,format!("ไม่พบขนาดป้าย {lbs_code}")); };
            let tmpl_code = if !explicit_template.is_empty() { explicit_template } else if pri_type == "2" {
                group_lbs.FTLbsTmplPmt.unwrap_or_else(|| "SH-100x38-PMT".to_string())
            } else {
                group_lbs.FTLbsTmplNor.unwrap_or_else(|| "SH-100x38-NOR".to_string())
            };
            let layout: Option<String> = sqlx::query_scalar("SELECT FTLtmLayout FROM TCNMLabelTmpl WHERE FTLtmCode=? AND FTLbsCode=? AND FTLtmStaUse='1'")
                .bind(&tmpl_code).bind(&lbs_code).fetch_optional(&st.pool).await.ok().flatten().flatten();
            let elements = layout.and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
                .and_then(|v| serde_json::from_value::<Vec<crate::templates::TemplateElement>>(v["elements"].clone()).ok())
                .or_else(||templates.iter().find(|t|t.code==tmpl_code).map(|t|t.elements.clone()));
            let Some(elements)=elements else{return err_json(StatusCode::BAD_REQUEST,format!("ไม่พบแบบพิมพ์ {}",tmpl_code));};
            let opts = DriverOptions {
                dpi,
                label_width_mm: group_lbs.FCLbsWidth,
                label_height_mm: group_lbs.FCLbsHeight,
                gap_mm: group_lbs.FCLbsGapV.unwrap_or(2.0).max(0.0),
                speed,
                darkness,
                copies: 1,
            };
            parts.push(match generate_commands(brand, &lbls, &elements, &opts) { Ok(cmd) => cmd, Err(e) => return err_json(StatusCode::BAD_REQUEST,e) });
        }
        parts.join("\r\n\r\n")
    } else {
        return err_json(StatusCode::BAD_REQUEST, "ไม่มีข้อมูลสำหรับพิมพ์");
    };

    if command_data.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "ไม่สามารถสร้างคำสั่งพิมพ์ได้");
    }

    let (success, message, bytes_sent) =
        if preview_only || brand == PrinterBrand::Samsung || brand == PrinterBrand::Windows {
        (
            true,
            format!("สร้างตัวอย่างคำสั่ง {} สำเร็จ (ไม่ได้ส่งพิมพ์จริง)", brand.language_label()),
            0,
        )
    } else {
        tcp_send(ip, port, &command_data).await
    };
    Json(json!({
        "success": success,
        "printerIp": ip,
        "printerPort": port,
        "brand": brand.as_str(),
        "brandLabel": brand.label(),
        "language": brand.language_label(),
        "fileExt": brand.file_ext(),
        "commandLength": command_data.len(),
        "message": message,
        "bytesSent": bytes_sent,
        "command": command_data
    }))
    .into_response()
}

// ─── /api/print/ezpl & /api/godex/* ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct EzplQuery {
    pub test: Option<String>,
    pub dpi: Option<u32>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

pub async fn get_print_ezpl(Query(q): Query<EzplQuery>) -> Response {
    if q.test.as_deref() == Some("1") {
        let ezpl = generate_test_pattern(
            PrinterBrand::Godex,
            q.dpi.unwrap_or(203),
            q.width.unwrap_or(100.0),
            q.height.unwrap_or(38.0),
        );
        return (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"godex_test.ezpl\""),
            ],
            ezpl,
        )
            .into_response();
    }
    Json(json!({
        "message": "EZPL API ready",
        "endpoints": ["POST /api/print/ezpl", "GET /api/print/ezpl?test=1"]
    }))
    .into_response()
}

pub async fn post_print_ezpl(
    State(st): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let label_size_code = body.get("labelSizeCode").and_then(|v| v.as_str()).unwrap_or("");
    let lbs = if !label_size_code.is_empty() {
        sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(label_size_code)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None)
    } else {
        None
    };

    let dpi = body.get("options").and_then(|o| o.get("dpi")).and_then(|v| v.as_u64()).unwrap_or(203) as u32;
    let w = lbs.as_ref().map(|s| s.FCLbsWidth).unwrap_or(100.0);
    let h = lbs.as_ref().map(|s| s.FCLbsHeight).unwrap_or(38.0);
    let templates = get_templates();

    if body.get("singleMode").and_then(|v| v.as_bool()).unwrap_or(false) {
        if let Some(sl) = body.get("singleLine") {
            let pri = sl.get("priType").and_then(|v| v.as_str()).unwrap_or("1");
            let tmpl_code = if pri == "2" {
                lbs.as_ref().and_then(|s| s.FTLbsTmplPmt.clone()).unwrap_or_else(|| "SH-100x38-PMT".to_string())
            } else {
                lbs.as_ref().and_then(|s| s.FTLbsTmplNor.clone()).unwrap_or_else(|| "SH-100x38-NOR".to_string())
            };
            let tmpl = templates.iter().find(|t| t.code == tmpl_code).unwrap_or(&templates[0]);
            let ctx = sl.get("ctx").cloned().unwrap_or(json!({}));
            let opts = DriverOptions { dpi, label_width_mm: w, label_height_mm: h, ..Default::default() };
            let ezpl = match generate_commands(PrinterBrand::Godex, &[(ctx, 1)], &tmpl.elements, &opts) { Ok(cmd) => cmd, Err(e) => return err_json(StatusCode::BAD_REQUEST,e) };
            return Json(json!({
                "ezpl": ezpl,
                "labelWidth": w,
                "labelHeight": h,
                "dpi": dpi,
                "templateCode": tmpl_code
            }))
            .into_response();
        }
    }

    let Some(lines) = body.get("lines").and_then(|v| v.as_array()).filter(|a| !a.is_empty()) else {
        return err_json(StatusCode::BAD_REQUEST, "No lines provided");
    };

    let mut groups: BTreeMap<String, Vec<(Value, u32)>> = BTreeMap::new();
    for l in lines {
        if !l.get("selected").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let lbs_c = l.get("lbsCode").and_then(|v| v.as_str()).unwrap_or("SH-100x38");
        let pri = l.get("priType").and_then(|v| v.as_str()).unwrap_or("1");
        let qty = l.get("qty").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let ctx = l.get("ctx").cloned().unwrap_or(json!({}));
        groups.entry(format!("{}_{}", lbs_c, pri)).or_default().push((ctx, qty));
    }

    let mut results = Vec::new();
    let mut parts = Vec::new();
    let mut total_labels = 0u32;

    for (key, lbls) in groups {
        let (lbs_code, pri_type) = key.split_once('_').unwrap_or(("SH-100x38", "1"));
        let group_lbs = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(lbs_code)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        let Some(group_lbs) = group_lbs else { return err_json(StatusCode::BAD_REQUEST,format!("ไม่พบขนาดป้าย {lbs_code}")); };
        let tmpl_code = if pri_type == "2" {
            group_lbs.FTLbsTmplPmt.unwrap_or_else(|| "SH-100x38-PMT".to_string())
        } else {
            group_lbs.FTLbsTmplNor.unwrap_or_else(|| "SH-100x38-NOR".to_string())
        };
        let tmpl = templates.iter().find(|t| t.code == tmpl_code).unwrap_or(&templates[0]);
        let opts = DriverOptions {
            dpi,
            label_width_mm: group_lbs.FCLbsWidth,
            label_height_mm: group_lbs.FCLbsHeight,
            ..Default::default()
        };
        let cnt: u32 = lbls.iter().map(|(_, q)| *q).sum();
        total_labels += cnt;
        parts.push(match generate_commands(PrinterBrand::Godex, &lbls, &tmpl.elements, &opts) { Ok(cmd) => cmd, Err(e) => return err_json(StatusCode::BAD_REQUEST,e) });
        results.push(json!({
            "group": format!("{} ({})", lbs_code, if pri_type == "2" { "Promo" } else { "Normal" }),
            "labelCount": cnt
        }));
    }

    Json(json!({
        "ezpl": parts.join("\r\n\r\n"),
        "groups": results,
        "totalLabels": total_labels
    }))
    .into_response()
}

pub async fn post_godex_status(Json(body): Json<Value>) -> Response {
    let ip = body.get("printerIp").and_then(|v| v.as_str()).unwrap_or("");
    if ip.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "printerIp is required");
    }
    let port = body.get("printerPort").and_then(|v| v.as_u64()).unwrap_or(9100) as u16;
    let (connected, message, _) = tcp_test(ip, port).await;
    Json(json!({
        "connected": connected,
        "ip": ip,
        "port": port,
        "message": message
    }))
    .into_response()
}

pub async fn post_godex_print(
    State(st): State<AppState>,
    Json(mut body): Json<Value>,
) -> Response {
    if let Some(obj) = body.as_object_mut() {
        obj.insert("brand".to_string(), json!("GODEX"));
    }
    crate::print_center::enqueue_lan(State(st), Json(body)).await
}

// ─── /api/import ─────────────────────────────────────────────────────────────

pub async fn post_import(State(_st): State<AppState>, _multipart: Multipart) -> Response {
    err_json(StatusCode::CONFLICT, "กรุณาใช้หน้าต่างนำเข้าข้อมูล (Excel / JSON / CSV) แบบใหม่เพื่อตรวจและนำเข้าข้อมูล")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterImportItem {
    pub pdt_code: String,
    pub pdt_name: String,
    pub bar_code: String,
    #[serde(default)]
    pub unit_code: String,
    #[serde(default)]
    pub unit_name: String,
    pub base_price: f64,
    #[serde(default)]
    pub effective_start: String,
    #[serde(default)]
    pub effective_stop: String,
    #[serde(default)]
    pub promo_price: Option<f64>,
    #[serde(default)]
    pub promo_name: String,
    #[serde(default)]
    pub promo_start: String,
    #[serde(default)]
    pub promo_stop: String,
    #[serde(default)]
    pub brand_name: String,
    #[serde(default)]
    pub group_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterImportBody {
    #[serde(default)]
    pub branch: String,
    pub items: Vec<MasterImportItem>,
}

/// Import Master product & price items (from Excel/JSON/CSV) into the Barprint-owned
/// standalone `AdaAccD.db`. Never modifies a StoreFront-shared POS database.
pub async fn post_import_master(
    State(st): State<AppState>,
    Json(body): Json<MasterImportBody>,
) -> Response {
    if body.items.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "ไม่มีรายการข้อมูลสินค้าที่จะนำเข้า");
    }
    if body.items.len() > 10000 {
        return err_json(StatusCode::BAD_REQUEST, "นำเข้าได้ไม่เกิน 10,000 รายการต่อครั้ง");
    }

    let mode = crate::standalone::mode(&st.pool).await;
    let configured: Option<(String, String)> = sqlx::query_as("SELECT path, branch FROM main.ABStorage WHERE id=1")
        .fetch_optional(&st.pool)
        .await
        .ok()
        .flatten();

    if configured.is_some() && mode == "POS_SHARED" {
        return err_json(
            StatusCode::CONFLICT,
            "ระบบกำลังเชื่อมต่อฐาน StoreFront ของ POS อยู่ (อ่านอย่างเดียว) จึงไม่อนุญาตให้นำเข้าทับฐาน POS — กรุณานำเข้าลงตารางพิมพ์ หรือเปลี่ยนเป็นโหมดเครื่องแยก",
        );
    }

    let target_path = configured
        .as_ref()
        .map(|(p, _)| std::path::PathBuf::from(p))
        .unwrap_or_else(crate::standalone::own_path);

    if let Err(e) = crate::standalone::prepare_destination(&target_path).await {
        return err_json(StatusCode::BAD_REQUEST, e);
    }

    let branch = if !body.branch.trim().is_empty() {
        body.branch.trim().to_string()
    } else if let Some((_, b)) = &configured {
        if !b.trim().is_empty() { b.trim().to_string() } else { "00001".to_string() }
    } else {
        "00001".to_string()
    };

    if configured.as_ref().is_some_and(|(_, b)| !b.is_empty() && b != &branch) {
        return err_json(StatusCode::BAD_REQUEST,"สาขาของไฟล์ต้องตรงกับสาขาที่ตั้งไว้");
    }
    let today = Local::now().format("%Y-%m-%d").to_string();
    let now_dt = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut conn = match sqlx::SqliteConnection::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&target_path)
            .create_if_missing(false)
            .busy_timeout(std::time::Duration::from_secs(10)),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, format!("เปิดฐานข้อมูล Master ไม่ได้: {e}")),
    };

    let mut imported = 0usize;
    let mut promo_count = 0usize;
    let write_res: Result<(), String> = async {
        let mut tx = conn.begin().await.map_err(|e| e.to_string())?;
        sqlx::query("INSERT OR IGNORE INTO TCNMBranch(FTBchCode,FTPplCode,FTWahCode) VALUES(?,'','')")
            .bind(&branch)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("INSERT OR IGNORE INTO TCNMBranch_L(FTBchCode,FNLngID,FTBchName) VALUES(?,1,?)")
            .bind(&branch)
            .bind(&branch)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        for (idx, item) in body.items.iter().enumerate() {
            let pdt_code = item.pdt_code.trim();
            let bar_code = item.bar_code.trim();
            let pdt_name = if item.pdt_name.trim().is_empty() { pdt_code } else { item.pdt_name.trim() };
            if pdt_code.is_empty() || bar_code.is_empty() || !item.base_price.is_finite() || item.base_price < 0.0 {
                return Err(format!("รายการที่ {} ข้อมูลไม่ครบถ้วน (ต้องมีรหัสสินค้า, บาร์โค้ด และราคา >= 0)", idx + 1));
            }
            let unit_code = if item.unit_code.trim().is_empty() { "PCS" } else { item.unit_code.trim() };
            let unit_name = if item.unit_name.trim().is_empty() { unit_code } else { item.unit_name.trim() };
            let d_start = if item.effective_start.trim().is_empty() { today.as_str() } else { item.effective_start.trim() };
            let d_stop = item.effective_stop.trim();

            sqlx::query("INSERT OR IGNORE INTO TCNMPdtUnit(FTPunCode,FDCreateOn) VALUES(?,?)")
                .bind(unit_code).bind(&now_dt).execute(&mut *tx).await.map_err(|e| e.to_string())?;
            sqlx::query("INSERT OR IGNORE INTO TCNMPdtUnit_L(FTPunCode,FNLngID,FTPunName) VALUES(?,1,?)")
                .bind(unit_code).bind(unit_name).execute(&mut *tx).await.map_err(|e| e.to_string())?;

            for (field, value) in [("EffectiveStart", d_start), ("EffectiveStop", d_stop), ("PromoStart", item.promo_start.trim()), ("PromoStop", item.promo_stop.trim())] {
                if !value.is_empty() && chrono::NaiveDate::parse_from_str(value,"%Y-%m-%d").map(|d| d.to_string()!=value).unwrap_or(true) {
                    return Err(format!("แถว {}: {field} ต้องเป็นวันที่ YYYY-MM-DD", idx+1));
                }
            }
            let promo_from = if item.promo_start.trim().is_empty(){d_start}else{item.promo_start.trim()};
            let promo_to = if item.promo_stop.trim().is_empty(){d_stop}else{item.promo_stop.trim()};
            if (!d_stop.is_empty() && d_stop < d_start) || item.promo_price.is_some_and(|price| price < 0.0 || (!promo_to.is_empty() && promo_to < promo_from)) {
                return Err(format!("แถว {}: ตรวจราคาและช่วงวันที่",idx+1));
            }
            sqlx::query("INSERT INTO TCNMPdt(FTPdtCode,FTPdtStaActive,FDCreateOn,FDLastUpdOn) VALUES(?,'1',?,?) ON CONFLICT(FTPdtCode) DO UPDATE SET FDLastUpdOn=excluded.FDLastUpdOn")
                .bind(pdt_code).bind(&now_dt).bind(&now_dt).execute(&mut *tx).await.map_err(|e| e.to_string())?;
            sqlx::query("INSERT INTO TCNMPdt_L(FTPdtCode,FNLngID,FTPdtName) VALUES(?,1,?) ON CONFLICT DO UPDATE SET FTPdtName=excluded.FTPdtName")
                .bind(pdt_code).bind(pdt_name).execute(&mut *tx).await.map_err(|e| e.to_string())?;
            sqlx::query("INSERT INTO TCNMPdtPackSize(FTPdtCode,FTPunCode,FCPdtUnitFact,FCPdtPriceRET) VALUES(?,?,1,?) ON CONFLICT DO UPDATE SET FCPdtPriceRET=excluded.FCPdtPriceRET")
                .bind(pdt_code).bind(unit_code).bind(item.base_price).execute(&mut *tx).await.map_err(|e| e.to_string())?;
            sqlx::query("INSERT INTO TCNMPdtBar(FTPdtCode,FTBarCode,FTPunCode,FNBarRefSeq,FTBarStaUse,FDCreateOn,FDLastUpdOn) VALUES(?,?,?,1,'1',?,?) ON CONFLICT DO UPDATE SET FTPunCode=excluded.FTPunCode,FDLastUpdOn=excluded.FDLastUpdOn")
                .bind(pdt_code).bind(bar_code).bind(unit_code).bind(&now_dt).bind(&now_dt).execute(&mut *tx).await.map_err(|e| e.to_string())?;

            let price_doc = format!("IMP:{}:{}", d_start, serde_json::json!([pdt_code,unit_code]));
            sqlx::query("INSERT INTO TCNTPdtPrice4PDT(FTBchCode,FTPdtCode,FTPunCode,FNPghSrcSeq,FDPghEffectiveStart,FDPghEffectiveStop,FDPghDStart,FDPghDStop,FTPghDocNo,FTPghDocType,FCPgdPrice,FCPgdPriceRet,FDCreateOn,FDLastUpdOn) VALUES(?,?,?,1,?,?,?,?,?,'1',?,?,?,?) ON CONFLICT DO UPDATE SET FCPgdPrice=excluded.FCPgdPrice,FCPgdPriceRet=excluded.FCPgdPriceRet,FDPghEffectiveStop=excluded.FDPghEffectiveStop,FDPghDStop=excluded.FDPghDStop,FDLastUpdOn=excluded.FDLastUpdOn")
                .bind(&branch).bind(pdt_code).bind(unit_code).bind(d_start).bind(d_stop).bind(d_start).bind(d_stop).bind(&price_doc).bind(item.base_price).bind(item.base_price).bind(&now_dt).bind(&now_dt)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
            if let Some(promo_pri) = item.promo_price.filter(|p| p.is_finite() && *p >= 0.0) {
                let p_start = if item.promo_start.trim().is_empty() { d_start } else { item.promo_start.trim() };
                let p_stop = if item.promo_stop.trim().is_empty() { d_stop } else { item.promo_stop.trim() };
                let p_doc = format!("IMP-PMT:{}", serde_json::json!([bar_code,p_start]));
                let p_name = if item.promo_name.trim().is_empty() { "โปรโมชันนำเข้า".to_string() } else { item.promo_name.trim().to_string() };
                let canonical: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM TCNTPmtHD WHERE NULLIF(TRIM(FTPmhRuleJson),'') IS NOT NULL) OR EXISTS(SELECT 1 FROM pragma_table_info('TCNTPmtGrpDT') WHERE name='FTPgdRefCode')")
                    .fetch_one(&mut *tx).await.map_err(|e|e.to_string())?;
                sqlx::query("INSERT OR REPLACE INTO TCNTPmtHD(FTAgnCode,FTPmhDocNo,FNPmhVersion,FTPmhStaActive,FTPmhName,FTPmhStaDoc,FTPmhStaPrcDoc,FDPmhDStart,FDPmhDStop,FNPmhPriority,FDCreateOn,FDLastUpdOn) VALUES('',?,1,'1',?,'1','1',?,?,100,?,?)")
                    .bind(&p_doc).bind(&p_name).bind(p_start).bind(p_stop).bind(&now_dt).bind(&now_dt)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                sqlx::query("INSERT OR REPLACE INTO TCNTPmtHD_L(FTAgnCode,FTPmhDocNo,FNPmhVersion,FNLngID,FTPmhName) VALUES('',?,1,1,?)")
                    .bind(&p_doc).bind(&p_name).execute(&mut *tx).await.map_err(|e| e.to_string())?;
                sqlx::query("INSERT OR REPLACE INTO TCNTPmtGrpDT(FTAgnCode,FTPmhDocNo,FTPmdGrpName,FNPmdSeqNo,FTPmdStaType,FTPmdRefType,FTPmdRefCode) VALUES('',?,'G1',1,'1','1',?)")
                    .bind(&p_doc).bind(bar_code).execute(&mut *tx).await.map_err(|e| e.to_string())?;
                sqlx::query("INSERT OR REPLACE INTO TCNTPmtCond(FTAgnCode,FTPmhDocNo,FNPmcSeqNo,FTPmcCondType,FCPmcValue) VALUES('',?,1,'FIX_PRI',?)")
                    .bind(&p_doc).bind(promo_pri).execute(&mut *tx).await.map_err(|e| e.to_string())?;
                if canonical {
                    let rule=json!({"groups":{"G1":{"include":[{"type":"BAR","code":bar_code}]}},"get":[{"type":"NP","target":"G1","value":promo_pri}]});
                    sqlx::query("UPDATE TCNTPmtHD SET FTPmhRuleJson=? WHERE FTAgnCode='' AND FTPmhDocNo=? AND FNPmhVersion=1")
                        .bind(rule.to_string()).bind(&p_doc).execute(&mut *tx).await.map_err(|e|e.to_string())?;
                }
                promo_count += 1;
            }
            imported += 1;
        }

        for t in ["TCNMBranch", "TCNMPdt", "TCNTPdtPrice4PDT", "TCNTPmtHD"] {
            sqlx::query("INSERT OR REPLACE INTO TSysSyncTaskDownload(FTSynTable,FDSynLastSuccess) VALUES(?,?)")
                .bind(t).bind(&now_dt).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
        sqlx::query("INSERT INTO ABMasterIdentity VALUES(1,'',?,'') ON CONFLICT(id) DO UPDATE SET branch=excluded.branch")
            .bind(&branch).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    let _ = conn.close().await;

    if let Err(e) = write_res {
        return err_json(StatusCode::BAD_REQUEST, e);
    }

    let path_str = target_path.to_string_lossy().to_string();
    let status_write: Result<(), sqlx::Error> = async {
        let mut tx=st.pool.begin().await?;
        sqlx::query("INSERT INTO ABStorage(id,path,branch,max_age_minutes) VALUES(1,?,?,1440) ON CONFLICT(id) DO UPDATE SET path=excluded.path,branch=excluded.branch")
        .bind(&path_str).bind(&branch).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO ABSource(id,mode,config,status,last_success) VALUES(1,'STANDALONE','{}','นำเข้าไฟล์ Master สำเร็จ',datetime('now')) ON CONFLICT(id) DO UPDATE SET mode='STANDALONE',status='นำเข้าไฟล์ Master สำเร็จ',last_success=datetime('now')")
        .execute(&mut *tx).await?;
        sqlx::query("UPDATE TCNSSysOption SET FTSopValue=? WHERE FTSopCode='DefaultBranch'")
        .bind(&branch).execute(&mut *tx).await?;

        tx.commit().await
    }.await;
    if let Err(error)=status_write {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR,format!("บันทึกสินค้าแล้ว แต่อัปเดตสถานะแหล่งข้อมูลไม่สำเร็จ: {error}"));
    }
    Json(json!({
        "ok": true,
        "imported": imported,
        "promoCount": promo_count,
        "path": path_str,
        "branch": branch,
        "message": format!("นำเข้า Master สำเร็จ {imported} รายการ")
    }))
    .into_response()
}

// ─── /api/paper-sizes, /api/preprints, /api/label-fields, /api/lan-printers ─

pub async fn get_paper_sizes(State(st): State<AppState>) -> Response {
    let rows = sqlx::query_as::<_, TCNMPaperSize>(
        "SELECT FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse FROM TCNMPaperSize WHERE FTPpsStaUse = '1' ORDER BY FNPpsSeq ASC, FTPpsCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();
    Json(rows).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePaperSizeBody {
    pub pps_code: String,
    pub pps_name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub win_kind: Option<String>,
    pub sta_custom: Option<String>,
    pub seq: Option<i64>,
}

pub async fn post_save_paper_size(
    State(st): State<AppState>,
    Json(body): Json<SavePaperSizeBody>,
) -> Response {
    let code = body.pps_code.trim().to_uppercase();
    let name = body.pps_name.trim();
    if code.is_empty() || name.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุรหัสและชื่อขนาดกระดาษ");
    }
    if body.width_mm < 10.0 || body.height_mm < 10.0 {
        return err_json(StatusCode::BAD_REQUEST, "ขนาดกระดาษต้องไม่น้อยกว่า 10 mm");
    }
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let wk = body.win_kind.unwrap_or_else(|| code.clone());
    let sta_c = if body.sta_custom.as_deref() == Some("1") { "1" } else { "2" };
    let seq = body.seq.unwrap_or(50);

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNMPaperSize
            (FTPpsCode, FTPpsName, FCPpsWidth, FCPpsHeight, FTPpsWinKind, FTPpsStaCustom, FNPpsSeq, FTPpsStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
        VALUES (?, ?, ?, ?, ?, ?, ?, '1', ?, 'USER_CONFIG', ?, 'USER_CONFIG')
        ON CONFLICT(FTPpsCode) DO UPDATE SET
            FTPpsName = excluded.FTPpsName,
            FCPpsWidth = excluded.FCPpsWidth,
            FCPpsHeight = excluded.FCPpsHeight,
            FTPpsWinKind = excluded.FTPpsWinKind,
            FTPpsStaCustom = excluded.FTPpsStaCustom,
            FNPpsSeq = excluded.FNPpsSeq,
            FTPpsStaUse = '1',
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'USER_CONFIG'
        "#,
    )
    .bind(&code).bind(name).bind(body.width_mm).bind(body.height_mm).bind(&wk).bind(sta_c).bind(seq)
    .bind(&now).bind(&now)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    Json(json!({ "ok": true, "ppsCode": code })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCodeBody {
    pub code: String,
}

pub async fn post_delete_paper_size(
    State(st): State<AppState>,
    Json(body): Json<DeleteCodeBody>,
) -> Response {
    let code = body.code.trim();
    if code.is_empty() || code == "A4" || code == "CUSTOM" {
        return err_json(StatusCode::BAD_REQUEST, "ไม่สามารถลบขนาดกระดาษหลักของระบบได้");
    }
    if let Err(e) = sqlx::query("DELETE FROM TCNMPaperSize WHERE FTPpsCode = ?")
        .bind(code)
        .execute(&st.pool)
        .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    Json(json!({ "ok": true })).into_response()
}

pub async fn get_preprints(State(st): State<AppState>) -> Response {
    let rows = sqlx::query_as::<_, TCNMPreprint>(
        "SELECT FTPprCode, FTPprName, FTPprFileName, FTPpsCode, FCPprWidth, FCPprHeight, FNPprCol, FNPprRow, FCPprLabelW, FCPprLabelH, FCPprMagTop, FCPprMagLeft, FCPprMagBottom, FCPprMagRight, FCPprGapH, FCPprGapV, FNPprPageNo, FTPprScope, FCPprOpacity, FTPprSheetImg, FTPprCellImg, FTPprCropJson, FTPprStaUse FROM TCNMPreprint WHERE FTPprStaUse = '1' ORDER BY FTPprCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();
    Json(rows).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePreprintBody {
    pub ppr_code: Option<String>,
    pub ppr_name: String,
    pub file_name: Option<String>,
    pub paper_size: Option<String>,
    pub paper_width_mm: f64,
    pub paper_height_mm: f64,
    pub cols: i64,
    pub rows: i64,
    pub label_width_mm: Option<f64>,
    pub label_height_mm: Option<f64>,
    pub mag_top: Option<f64>,
    pub mag_left: Option<f64>,
    pub mag_bottom: Option<f64>,
    pub mag_right: Option<f64>,
    pub gap_h: Option<f64>,
    pub gap_v: Option<f64>,
    pub page_no: Option<i64>,
    pub scope: Option<String>,
    pub opacity: Option<f64>,
    pub sheet_img: Option<String>,
    pub cell_img: Option<String>,
    pub crop_json: Option<Value>,
}

pub async fn post_save_preprint(
    State(st): State<AppState>,
    Json(body): Json<SavePreprintBody>,
) -> Response {
    let name = body.ppr_name.trim();
    if name.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุชื่อฟอร์ม Pre-Print");
    }
    let code = body
        .ppr_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| format!("PPR-{}", &gen_guid()[..8]));
    let (pps_code, _, _, _) = lookup_paper_size(&st.pool, body.paper_size.as_deref()).await;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let crop_str = body.crop_json.as_ref().and_then(|v| serde_json::to_string(v).ok());

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNMPreprint
            (FTPprCode, FTPprName, FTPprFileName, FTPpsCode, FCPprWidth, FCPprHeight, FNPprCol, FNPprRow, FCPprLabelW, FCPprLabelH, FCPprMagTop, FCPprMagLeft, FCPprMagBottom, FCPprMagRight, FCPprGapH, FCPprGapV, FNPprPageNo, FTPprScope, FCPprOpacity, FTPprSheetImg, FTPprCellImg, FTPprCropJson, FTPprStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'DESIGNER', ?, 'DESIGNER')
        ON CONFLICT(FTPprCode) DO UPDATE SET
            FTPprName = excluded.FTPprName,
            FTPprFileName = COALESCE(excluded.FTPprFileName, TCNMPreprint.FTPprFileName),
            FTPpsCode = excluded.FTPpsCode,
            FCPprWidth = excluded.FCPprWidth,
            FCPprHeight = excluded.FCPprHeight,
            FNPprCol = excluded.FNPprCol,
            FNPprRow = excluded.FNPprRow,
            FCPprLabelW = excluded.FCPprLabelW,
            FCPprLabelH = excluded.FCPprLabelH,
            FCPprMagTop = excluded.FCPprMagTop,
            FCPprMagLeft = excluded.FCPprMagLeft,
            FCPprMagBottom = excluded.FCPprMagBottom,
            FCPprMagRight = excluded.FCPprMagRight,
            FCPprGapH = excluded.FCPprGapH,
            FCPprGapV = excluded.FCPprGapV,
            FNPprPageNo = excluded.FNPprPageNo,
            FTPprScope = excluded.FTPprScope,
            FCPprOpacity = excluded.FCPprOpacity,
            FTPprSheetImg = COALESCE(excluded.FTPprSheetImg, TCNMPreprint.FTPprSheetImg),
            FTPprCellImg = COALESCE(excluded.FTPprCellImg, TCNMPreprint.FTPprCellImg),
            FTPprCropJson = COALESCE(excluded.FTPprCropJson, TCNMPreprint.FTPprCropJson),
            FTPprStaUse = '1',
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'DESIGNER'
        "#,
    )
    .bind(&code)
    .bind(name)
    .bind(&body.file_name)
    .bind(&pps_code)
    .bind(body.paper_width_mm.max(30.0))
    .bind(body.paper_height_mm.max(30.0))
    .bind(body.cols.max(1))
    .bind(body.rows.max(1))
    .bind(body.label_width_mm)
    .bind(body.label_height_mm)
    .bind(body.mag_top.unwrap_or(3.0))
    .bind(body.mag_left.unwrap_or(3.0))
    .bind(body.mag_bottom.or(body.mag_top).unwrap_or(3.0))
    .bind(body.mag_right.or(body.mag_left).unwrap_or(3.0))
    .bind(body.gap_h.unwrap_or(2.0))
    .bind(body.gap_v.unwrap_or(2.0))
    .bind(body.page_no.unwrap_or(1))
    .bind(body.scope.as_deref().unwrap_or("sheet"))
    .bind(body.opacity.unwrap_or(0.72))
    .bind(&body.sheet_img)
    .bind(&body.cell_img)
    .bind(&crop_str)
    .bind(&now)
    .bind(&now)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({ "ok": true, "pprCode": code })).into_response()
}

pub async fn post_delete_preprint(
    State(st): State<AppState>,
    Json(body): Json<DeleteCodeBody>,
) -> Response {
    let code = body.code.trim();
    if code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "code is required");
    }
    let _ = sqlx::query("UPDATE TCNMLabelTmpl SET FTPprCode = NULL, FTLtmMode = 'full' WHERE FTPprCode = ?")
        .bind(code).execute(&st.pool).await;
    let _ = sqlx::query("UPDATE TCNMLabelSize SET FTPprCode = NULL WHERE FTPprCode = ?")
        .bind(code).execute(&st.pool).await;
    let _ = sqlx::query("DELETE FROM TCNMPreprint WHERE FTPprCode = ?")
        .bind(code).execute(&st.pool).await;
    Json(json!({ "ok": true })).into_response()
}

pub async fn get_label_fields(State(st): State<AppState>) -> Response {
    let rows = sqlx::query_as::<_, TCNMLabelField>(
        "SELECT FTLbfCode, FTLbfName, FTLbfType, FTLbfSample, FCLbfDefW, FCLbfDefH, FNLbfDefFont, FTLbfDefWeight, FTLbfDefColor, FNLbfSeq, FTLbfStaAdv, FTLbfStaUse FROM TCNMLabelField WHERE FTLbfStaUse = '1' ORDER BY FNLbfSeq ASC, FTLbfCode ASC",
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();
    Json(rows).into_response()
}

pub async fn get_lan_printers(State(st): State<AppState>) -> Response {
    let rows = fetch_lan_printers(&st.pool).await;
    Json(rows).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLanPrinterBody {
    pub id: Option<String>,
    pub name: String,
    pub brand: String,
    pub model: Option<String>,
    pub ip: String,
    pub port: Option<i64>,
    pub dpi: Option<i64>,
    pub label_width: Option<f64>,
    pub label_height: Option<f64>,
    pub speed: Option<i64>,
    pub darkness: Option<i64>,
    pub enabled: Option<bool>,
}

pub async fn post_save_lan_printer(
    State(st): State<AppState>,
    Json(body): Json<SaveLanPrinterBody>,
) -> Response {
    let name = body.name.trim();
    let ip = body.ip.trim();
    if name.is_empty() || ip.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "กรุณาระบุชื่อและ IP ของเครื่องพิมพ์");
    }
    let prn_code = body
        .id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("P{}", &gen_guid()[..6]));
    let spp_code = format!("SPP_{}", prn_code);
    let port = body.port.unwrap_or(9100);
    let spp_val = format!("{}:{}", ip, port);
    let brand = body.brand.trim().to_uppercase();
    let meta = json!({
        "model": body.model.unwrap_or_else(|| brand.clone()),
        "dpi": body.dpi.unwrap_or(203),
        "labelWidth": body.label_width.unwrap_or(100.0),
        "labelHeight": body.label_height.unwrap_or(38.0),
        "speed": body.speed.unwrap_or(4),
        "darkness": body.darkness.unwrap_or(10),
    })
    .to_string();
    let sta = if body.enabled.unwrap_or(true) { "1" } else { "2" };
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO TSysPortPrn (FTSppCode, FTSppValue, FTSppRef, FTSppType, FTSppStaUse, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy)
        VALUES (?, ?, ?, 'TCP', ?, ?, 'USER_CONFIG', ?, 'USER_CONFIG')
        ON CONFLICT(FTSppCode) DO UPDATE SET
            FTSppValue = excluded.FTSppValue,
            FTSppRef = excluded.FTSppRef,
            FTSppStaUse = excluded.FTSppStaUse,
            FDLastUpdOn = excluded.FDLastUpdOn
        "#,
    )
    .bind(&spp_code).bind(&spp_val).bind(&meta).bind(sta).bind(&now).bind(&now)
    .execute(&st.pool).await;

    let _ = sqlx::query(
        r#"
        INSERT INTO TCNMPrinter (FTPrnCode, FTPrnSrcType, FTSppCode, FTPrnDriver, FTPrnType, FDLastUpdOn, FTLastUpdBy, FDCreateOn, FTCreateBy)
        VALUES (?, '2', ?, ?, ?, ?, 'USER_CONFIG', ?, 'USER_CONFIG')
        ON CONFLICT(FTPrnCode) DO UPDATE SET
            FTPrnSrcType = '2',
            FTSppCode = excluded.FTSppCode,
            FTPrnDriver = excluded.FTPrnDriver,
            FTPrnType = excluded.FTPrnType,
            FDLastUpdOn = excluded.FDLastUpdOn
        "#,
    )
    .bind(&prn_code).bind(&spp_code).bind(&brand).bind(&brand).bind(&now).bind(&now)
    .execute(&st.pool).await;

    let _ = sqlx::query(
        r#"
        INSERT INTO TCNMPrinter_L (FTPrnCode, FNLngID, FTPrnName, FTPrnRmk)
        VALUES (?, 1, ?, 'LAN Thermal Printer')
        ON CONFLICT(FTPrnCode, FNLngID) DO UPDATE SET
            FTPrnName = excluded.FTPrnName
        "#,
    )
    .bind(&prn_code).bind(name)
    .execute(&st.pool).await;

    Json(json!({ "ok": true, "prnCode": prn_code })).into_response()
}

pub async fn post_delete_lan_printer(
    State(st): State<AppState>,
    Json(body): Json<DeleteCodeBody>,
) -> Response {
    let code = body.code.trim();
    if code.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "code is required");
    }
    let spp_code = format!("SPP_{}", code);
    let _ = sqlx::query("DELETE FROM TCNMPrinter_L WHERE FTPrnCode = ?").bind(code).execute(&st.pool).await;
    let _ = sqlx::query("DELETE FROM TCNMPrinter WHERE FTPrnCode = ?").bind(code).execute(&st.pool).await;
    let _ = sqlx::query("DELETE FROM TSysPortPrn WHERE FTSppCode = ?").bind(&spp_code).execute(&st.pool).await;
    Json(json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
pub struct ImageProxyQuery {
    pub url: String,
}

fn sniff_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes
        .iter()
        .take(256)
        .copied()
        .collect::<Vec<u8>>()
        .windows(4)
        .any(|w| w.eq_ignore_ascii_case(b"<svg"))
    {
        Some("image/svg+xml")
    } else {
        None
    }
}

pub async fn get_image_proxy(Query(q): Query<ImageProxyQuery>) -> Response {
    let target = q.url.trim();
    if !(target.starts_with("https://") || target.starts_with("http://")) {
        return err_json(StatusCode::BAD_REQUEST, "only http/https URLs are supported");
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => return err_json(StatusCode::INTERNAL_SERVER_ERROR, format!("proxy client failed: {}", e)),
    };

    match client.get(target).send().await {
        Ok(resp) if resp.status().is_success() => match resp.bytes().await {
            Ok(bytes) if !bytes.is_empty() => {
                if let Some(mime) = sniff_image_mime(&bytes) {
                    axum::http::Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, mime)
                        .header(header::CACHE_CONTROL, "public, max-age=300")
                        .body(axum::body::Body::from(bytes))
                        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
                } else {
                    err_json(StatusCode::BAD_GATEWAY, "remote resource is not a supported image")
                }
            }
            _ => err_json(StatusCode::BAD_GATEWAY, "failed to fetch remote image"),
        },
        Ok(_) => err_json(StatusCode::BAD_GATEWAY, "failed to fetch remote image"),
        Err(e) => err_json(StatusCode::BAD_GATEWAY, format!("proxy fetch failed: {}", e)),
    }
}

// ─── Auto-Print @ 04:00 AM Schedule, Sync & Morning Ops APIs ────────────────

pub async fn get_schedules(State(st): State<AppState>) -> Response {
    let schedules = sqlx::query_as::<_, TCNTLabelSchedule>(
        r#"
        SELECT FTLscCode, FTLscName, FTLscStaUse, FTLscRunTime, FNLscPreSyncMin, FNLscJitterSec,
               FTLscDaysMask, FTLscTriggerRules, FTLscBchCode, FTLscPplCode, FTLscLbsCode,
               FNLscQtyMode, FTLscSyncMode, FNLscSyncRetry, FNLscRetryWaitSec, FTLscOnSyncFail,
               FTLscOnPrinterBack, FNLscMisfireMin, FTLscStaCoverSheet, FDLscLastRunAt,
               FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
        FROM TCNTLabelSchedule
        ORDER BY FTLscCode ASC
        "#,
    )
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let latest_run = sqlx::query_as::<_, TCNTLabelSchedRun>(
        "SELECT FTLsrRunID, FTLscCode, FDLsrBizDate, FTLsrTriggerBy, FTLsrStatus, FDLsrStartedAt, FDLsrSyncedAt, FTLsrSyncProofJson, FNLsrSyncAttempts, FNLsrCandTotal, FNLsrSkippedDup, FNLsrNorQty, FNLsrPmtQty, FNLsrFailQty, FTLjbJobID, FTLsrErrMsg, FDLsrFinishedAt FROM TCNTLabelSchedRun ORDER BY FDLsrStartedAt DESC LIMIT 1",
    )
    .fetch_optional(&st.pool)
    .await
    .unwrap_or(None);

    let computer_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "LOCAL-PC".to_string());
    let scheduler_active = std::env::var("AUTOPRINT_DISABLE_EMBEDDED_SCHEDULER").as_deref() != Ok("1");

    Json(json!({
        "schedules": schedules,
        "latestRun": latest_run,
        "computerName": computer_name,
        "schedulerActive": scheduler_active,
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveScheduleBody {
    pub code: Option<String>,
    pub name: Option<String>,
    pub sta_use: String,
    pub run_time: String,
    pub pre_sync_min: Option<i64>,
    pub jitter_sec: Option<i64>,
    pub days_mask: String,
    pub trigger_rules: String,
    pub bch_code: Option<String>,
    pub ppl_code: Option<String>,
    pub lbs_code: String,
    pub qty_mode: Option<i64>,
    pub sync_mode: Option<String>,
    pub sync_retry: Option<i64>,
    pub retry_wait_sec: Option<i64>,
    pub on_sync_fail: Option<String>,
    pub on_printer_back: Option<String>,
    pub misfire_min: Option<i64>,
    pub sta_cover_sheet: Option<String>,
}

pub async fn post_save_schedule(
    State(st): State<AppState>,
    Json(body): Json<SaveScheduleBody>,
) -> Response {
    let code = body.code.as_deref().map(str::trim).filter(|s| !s.is_empty()).unwrap_or("SCH-0400");
    let run_time = body.run_time.trim();
    if NaiveTime::parse_from_str(run_time, "%H:%M").is_err() {
        return err_json(StatusCode::BAD_REQUEST, "รูปแบบเวลาไม่ถูกต้อง (ต้องเป็น HH:MM เช่น 04:00)");
    }
    let sta_use = if body.sta_use.trim() == "1" { "1" } else { "2" };
    let on_prt_back = if body.on_printer_back.as_deref() == Some("AUTO_RESUME") {
        "AUTO_RESUME"
    } else {
        "WAIT_CONFIRM"
    };
    let on_sync_fail = if body.on_sync_fail.as_deref() == Some("USE_LOCAL_WARN") {
        "USE_LOCAL_WARN"
    } else {
        "ABORT"
    };
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO TCNTLabelSchedule (
            FTLscCode, FTLscName, FTLscStaUse, FTLscRunTime, FNLscPreSyncMin, FNLscJitterSec,
            FTLscDaysMask, FTLscTriggerRules, FTLscBchCode, FTLscPplCode, FTLscLbsCode,
            FNLscQtyMode, FTLscSyncMode, FNLscSyncRetry, FNLscRetryWaitSec, FTLscOnSyncFail,
            FTLscOnPrinterBack, FNLscMisfireMin, FTLscStaCoverSheet, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'USER_CONFIG', ?, 'USER_CONFIG')
        ON CONFLICT(FTLscCode) DO UPDATE SET
            FTLscName = excluded.FTLscName,
            FTLscStaUse = excluded.FTLscStaUse,
            FTLscRunTime = excluded.FTLscRunTime,
            FNLscPreSyncMin = excluded.FNLscPreSyncMin,
            FNLscJitterSec = excluded.FNLscJitterSec,
            FTLscDaysMask = excluded.FTLscDaysMask,
            FTLscTriggerRules = excluded.FTLscTriggerRules,
            FTLscBchCode = excluded.FTLscBchCode,
            FTLscPplCode = excluded.FTLscPplCode,
            FTLscLbsCode = excluded.FTLscLbsCode,
            FNLscQtyMode = excluded.FNLscQtyMode,
            FTLscSyncMode = excluded.FTLscSyncMode,
            FNLscSyncRetry = excluded.FNLscSyncRetry,
            FNLscRetryWaitSec = excluded.FNLscRetryWaitSec,
            FTLscOnSyncFail = excluded.FTLscOnSyncFail,
            FTLscOnPrinterBack = excluded.FTLscOnPrinterBack,
            FNLscMisfireMin = excluded.FNLscMisfireMin,
            FTLscStaCoverSheet = excluded.FTLscStaCoverSheet,
            FDLastUpdOn = excluded.FDLastUpdOn,
            FTLastUpdBy = 'USER_CONFIG'
        "#,
    )
    .bind(code)
    .bind(body.name.as_deref().unwrap_or("พิมพ์ป้ายอัตโนมัติประจำวัน (04:00 น.)"))
    .bind(sta_use)
    .bind(run_time)
    .bind(body.pre_sync_min.unwrap_or(30).clamp(5, 120))
    .bind(body.jitter_sec.unwrap_or(900).clamp(0, 3600))
    .bind(&body.days_mask)
    .bind(&body.trigger_rules)
    .bind(&body.bch_code)
    .bind(&body.ppl_code)
    .bind(body.lbs_code.trim())
    .bind(body.qty_mode.unwrap_or(1))
    .bind(body.sync_mode.as_deref().unwrap_or("STOREFRONT"))
    .bind(body.sync_retry.unwrap_or(3).clamp(1, 5))
    .bind(body.retry_wait_sec.unwrap_or(300).clamp(5, 1800))
    .bind(on_sync_fail)
    .bind(on_prt_back)
    .bind(body.misfire_min.unwrap_or(300).clamp(15, 720))
    .bind(if body.sta_cover_sheet.as_deref() == Some("2") { "2" } else { "1" })
    .bind(&now)
    .bind(&now)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    Json(json!({ "ok": true, "code": code })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerScheduleBody {
    pub mode: Option<String>, // DRY_RUN | PRINT_NOW
    pub biz_date: Option<String>,
    pub lbs_code: Option<String>,
    pub trigger_rules: Option<String>,
    pub bch_code: Option<String>,
    pub qty_mode: Option<i64>,
}

pub async fn post_trigger_schedule(
    State(st): State<AppState>,
    Path(code): Path<String>,
    Json(body): Json<TriggerScheduleBody>,
) -> Response {
    let mode = body.mode.unwrap_or_else(|| "DRY_RUN".to_string());
    let dry_run = !mode.eq_ignore_ascii_case("PRINT_NOW");
    let overrides = if dry_run
        && (body.lbs_code.is_some()
            || body.trigger_rules.is_some()
            || body.bch_code.is_some()
            || body.qty_mode.is_some())
    {
        Some(crate::scheduler_worker::DryRunOverrides {
            lbs_code: body.lbs_code,
            trigger_rules: body.trigger_rules,
            bch_code: body.bch_code,
            qty_mode: body.qty_mode,
        })
    } else {
        None
    };

    match crate::scheduler_worker::execute_schedule_run_with_overrides(
        &st.pool,
        code.trim(),
        "MANUAL",
        dry_run,
        body.biz_date.as_deref(),
        overrides,
    )
    .await
    {
        Ok(res) => Json(json!(res)).into_response(),
        Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRunsQuery {
    pub run_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_schedule_runs(
    State(st): State<AppState>,
    Query(q): Query<ScheduleRunsQuery>,
) -> Response {
    let limit = q.limit.unwrap_or(15).clamp(1, 100);
    let runs = sqlx::query_as::<_, TCNTLabelSchedRun>(
        "SELECT FTLsrRunID, FTLscCode, FDLsrBizDate, FTLsrTriggerBy, FTLsrStatus, FDLsrStartedAt, FDLsrSyncedAt, FTLsrSyncProofJson, FNLsrSyncAttempts, FNLsrCandTotal, FNLsrSkippedDup, FNLsrNorQty, FNLsrPmtQty, FNLsrFailQty, FTLjbJobID, FTLsrErrMsg, FDLsrFinishedAt FROM TCNTLabelSchedRun ORDER BY FDLsrStartedAt DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let focus_run_id = q
        .run_id
        .filter(|s| !s.trim().is_empty())
        .or_else(|| runs.first().map(|r| r.FTLsrRunID.clone()));

    let queue_items = if let Some(ref rid) = focus_run_id {
        sqlx::query_as::<_, TCNTLabelSchedQueue>(
            "SELECT FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc, FTLsqPriType, FTLbsCode, FCLsqOldPrice, FCLsqSalePrice, FTLsqPromoDocNo, FTLsqPriceHash, FNLsqQty, FNLsqBatchPage, FTLsqStatus, FTLsqErrMsg FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ? ORDER BY FNLsqSeq ASC LIMIT 500",
        )
        .bind(rid)
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    Json(json!({
        "runs": runs,
        "latestRun": runs.first(),
        "queueItems": queue_items,
    }))
    .into_response()
}

pub async fn post_reprint_failed_run(
    State(st): State<AppState>,
    Path(run_id): Path<String>,
) -> Response {
    match crate::scheduler_worker::reprint_failed_run(&st.pool, run_id.trim()).await {
        Ok(res) => Json(json!(res)).into_response(),
        Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

pub async fn post_load_failed_to_manual(
    State(st): State<AppState>,
    Path(run_id): Path<String>,
) -> Response {
    let failed_rows = sqlx::query_as::<_, TCNTLabelSchedQueue>(
        "SELECT FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc, FTLsqPriType, FTLbsCode, FCLsqOldPrice, FCLsqSalePrice, FTLsqPromoDocNo, FTLsqPriceHash, FNLsqQty, FNLsqBatchPage, FTLsqStatus, FTLsqErrMsg FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ? AND FTLsqStatus = 'FAILED' ORDER BY FNLsqSeq ASC",
    )
    .bind(run_id.trim())
    .fetch_all(&st.pool)
    .await
    .unwrap_or_default();

    let store_name: String = sqlx::query_scalar::<_, Option<String>>(
        "SELECT FTBchName FROM TCNMBranch_L WHERE FNLngID = 1 LIMIT 1",
    )
    .fetch_optional(&st.pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default();
    let print_date_th = Local::now().format("%d/%m/%Y").to_string();

    let mut manual_lines = Vec::with_capacity(failed_rows.len());
    for row in failed_rows {
        let info_row: Option<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            r#"
            SELECT COALESCE(pl.FTPdtName, b.FTPdtCode), pl.FTPdtNameOth, ul.FTPunName, ps.FTPdtUnitDim
            FROM TCNMPdtBar b
            LEFT JOIN TCNMPdt_L pl ON pl.FTPdtCode = b.FTPdtCode AND pl.FNLngID = 1
            LEFT JOIN TCNMPdtUnit_L ul ON ul.FTPunCode = b.FTPunCode AND ul.FNLngID = 1
            LEFT JOIN TCNMPdtPackSize ps ON ps.FTPdtCode = b.FTPdtCode AND ps.FTPunCode = b.FTPunCode
            WHERE b.FTPdtCode = ? AND b.FTBarCode = ?
            LIMIT 1
            "#,
        )
        .bind(&row.FTPdtCode)
        .bind(&row.FTBarCode)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        let (pdt_name, name_oth, selling_unit, dim_str) = match info_row {
            Some((n, no, u, d)) => (n, no.unwrap_or_default(), u.unwrap_or_default(), d.unwrap_or_default()),
            None => (row.FTPdtCode.clone(), String::new(), String::new(), String::new()),
        };

        let label_size = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&row.FTLbsCode)
        .fetch_optional(&st.pool)
        .await
        .unwrap_or(None);

        let is_promo = row.FTLsqPriType == "2";
        let tmpl_code = if is_promo {
            label_size.as_ref().and_then(|s| s.FTLbsTmplPmt.clone()).unwrap_or_else(|| format!("{}-PMT", row.FTLbsCode))
        } else {
            label_size.as_ref().and_then(|s| s.FTLbsTmplNor.clone()).unwrap_or_else(|| format!("{}-NOR", row.FTLbsCode))
        };

        let (promo_title, promo_intv) = if is_promo {
            if let Some(ref doc_no) = row.FTLsqPromoDocNo {
                let v6: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
                    "SELECT COALESCE(NULLIF(hdl.FTPmhName, ''), hd.FTPmhName, hd.FTPmhDocNo), substr(CAST(hd.FDPmhDStart AS TEXT), 1, 10), substr(CAST(hd.FDPmhDStop AS TEXT), 1, 10) FROM TCNTPmtHD hd LEFT JOIN TCNTPmtHD_L hdl ON hdl.FTAgnCode = hd.FTAgnCode AND hdl.FTPmhDocNo = hd.FTPmhDocNo AND hdl.FNPmhVersion = hd.FNPmhVersion AND hdl.FNLngID = 1 WHERE hd.FTPmhDocNo = ? LIMIT 1",
                )
                .bind(doc_no)
                .fetch_optional(&st.pool)
                .await
                .unwrap_or(None);
                match v6 {
                    Some((nm, ds, de)) => (nm.unwrap_or_else(|| doc_no.clone()), promo_interval(ds.as_deref(), de.as_deref())),
                    None => (doc_no.clone(), String::new()),
                }
            } else {
                (String::new(), String::new())
            }
        } else {
            (String::new(), String::new())
        };

        let has_discount = is_promo && row.FCLsqOldPrice.map(|op| op > row.FCLsqSalePrice + 0.001).unwrap_or(true);
        let is_gift = is_promo && !has_discount;

        manual_lines.push(json!({
            "pdtCode": row.FTPdtCode,
            "barCode": row.FTBarCode,
            "pdtName": pdt_name,
            "price": row.FCLsqSalePrice,
            "priType": row.FTLsqPriType,
            "qty": row.FNLsqQty.max(1),
            "lbsCode": row.FTLbsCode,
            "tmplCode": tmpl_code,
            "promoName": if promo_title.is_empty() { row.FTLsqPromoDocNo.clone() } else { Some(promo_title.clone()) },
            "statusText": "นำเข้าจากค้างพิมพ์ตี 4",
            "ctx": {
                "product": {
                    "code": row.FTPdtCode,
                    "barcode": row.FTBarCode,
                    "name": pdt_name,
                    "name_oth": name_oth,
                    "dim": dim_str,
                    "unit": selling_unit,
                },
                "price": {
                    "value": row.FCLsqSalePrice,
                    "old_value": row.FCLsqOldPrice,
                    "old_has": row.FCLsqOldPrice.is_some(),
                },
                "promotion": {
                    "is_promo": is_promo,
                    "is_discount": has_discount,
                    "is_gift": is_gift,
                    "text": if is_promo { "PROMOTION" } else { "" },
                    "free_text": if is_gift { promo_title.as_str() } else { "" },
                    "name": promo_title,
                    "interval": promo_intv,
                },
                "tag": {
                    "print_date": print_date_th,
                    "pri_status": if is_promo { "2" } else { "1" },
                    "selling_unit": selling_unit,
                    "lab_size": row.FTLbsCode,
                },
                "store": {
                    "name": store_name,
                }
            }
        }));
    }

    Json(json!({
        "ok": true,
        "runId": run_id,
        "count": manual_lines.len(),
        "lines": manual_lines,
    }))
    .into_response()
}

pub async fn get_sync_status(State(st): State<AppState>) -> Response {
    Json(crate::storage::status(&st.pool).await).into_response()
}
pub async fn post_sync_trigger(State(st): State<AppState>) -> Response {
    crate::print_center::request_sync(State(st)).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyAdminPinBody {
    pub pin: String,
    #[serde(default)]
    pub new_pin: String,
}

pub async fn post_verify_admin_pin(
    State(st): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<VerifyAdminPinBody>,
) -> Response {
    let pin_change_required = match crate::storage::verify_and_update_admin_pin(&st.pool, &body.pin, &body.new_pin).await {
        Ok(still_required) => still_required,
        Err((status, msg, required)) => {
            return (
                status,
                Json(json!({ "ok": false, "pinChangeRequired": required, "error": msg })),
            )
                .into_response();
        }
    };
    match crate::auth::grant_admin(&st.pool, &headers).await {
        Ok(()) => Json(json!({ "ok": true, "pinChangeRequired": pin_change_required })).into_response(),
        Err(e) => err_json(StatusCode::UNAUTHORIZED, e),
    }
}

#[cfg(test)]
mod manual_job_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fixture() -> SqlitePool {
        let pool = SqlitePoolOptions::new().max_connections(1)
            .connect("sqlite::memory:").await.unwrap();
        crate::db::ensure_label_schema(&pool).await.unwrap();
        pool
    }

    fn request() -> CreateJobBody {
        serde_json::from_value(json!({
            "bchCode": "00001", "user": "TEST", "lines": [{
                "barcode": "885000000001", "pdtCode": "P1", "qty": 2,
                "ctx": {"product": {"name": "Test"}}, "priType": "2",
                "lbsCode": "CUSTOM", "tmplCode": "CUSTOM-PMT", "salePrice": 10,
                "promo": null
            }]
        })).unwrap()
    }

    #[tokio::test]
    async fn manual_job_preserves_promo_type_without_optional_promo_object() {
        let pool = fixture().await;
        let response = post_create_job(State(AppState { pool: pool.clone() }), Json(request())).await;
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT FTLjdPriStat, FNLjdLabQty FROM TCNTLabelJobDt ORDER BY FNLjdSeq"
        ).fetch_all(&pool).await.unwrap();
        assert_eq!(rows, vec![("2".into(), 1), ("2".into(), 1)]);
    }

    #[tokio::test]
    async fn manual_job_rolls_back_header_and_details_on_log_failure() {
        let pool = fixture().await;
        sqlx::query("CREATE TRIGGER fail_log BEFORE INSERT ON TCNTLabelLog BEGIN SELECT RAISE(ABORT, 'simulated log failure'); END")
            .execute(&pool).await.unwrap();
        let response = post_create_job(State(AppState { pool: pool.clone() }), Json(request())).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("simulated log failure"));
        for table in ["TCNTLabelJob", "TCNTLabelJobDt", "TCNTLabelLog"] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool).await.unwrap();
            assert_eq!(count, 0, "{table} must roll back");
        }
    }
}

#[cfg(test)]
mod event_printer_mapping_tests {
    use super::*;
    async fn fixture() -> SqlitePool {
        let p=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        crate::print_center::ensure_schema(&p).await.unwrap();
        sqlx::raw_sql("CREATE TABLE TCNMLabelPrinter(FTLbsCode TEXT,FTPsvCode TEXT,FTLbpPriType TEXT,FTLbpPrinter TEXT,FTLbpTray TEXT,FTLbpProtocol TEXT,FTLbpStaUse TEXT,FDCreateOn TEXT,FTCreateBy TEXT,FDLastUpdOn TEXT,FTLastUpdBy TEXT,PRIMARY KEY(FTLbsCode,FTPsvCode,FTLbpPriType));").execute(&p).await.unwrap();
        p
    }
    fn payload(events:bool) -> SaveLabelPrinterBody {
        let mut v=json!({"lbsCode":"L1","normalPrinter":"Normal","promoPrinter":"Promo"});
        if events {v["eventRoutes"]=json!({"NEW_PDT":{"printer":"New","tray":"Tray 2","protocol":"WINDOWS"},"PROMO_END_REVERT":{"printer":"Return","tray":"Tray 1","protocol":"WINDOWS"}});}
        serde_json::from_value(v).unwrap()
    }
    #[tokio::test]
    async fn template_routes_are_isolated_and_survive_legacy_saves() {
        let p=fixture().await;
        sqlx::raw_sql("CREATE TABLE TCNMLabelTmpl(FTLtmCode TEXT,FTLbsCode TEXT,FTLtmStaUse TEXT); INSERT INTO TCNMLabelTmpl VALUES('BAR','L1','1'),('PRICE','L1','1');").execute(&p).await.unwrap();
        let body:SaveLabelPrinterBody=serde_json::from_value(json!({"lbsCode":"L1","normalPrinter":"Normal","promoPrinter":"Promo","templateRoutes":{"BAR":{"printer":"Sticker","tray":"Roll","protocol":"TSPL"},"PRICE":{"printer":"Laser","tray":"Tray 2","protocol":"WINDOWS"}}})).unwrap();
        assert!(post_label_printers(State(AppState{pool:p.clone()}),Json(body)).await.status().is_success());
        assert!(post_label_printers(State(AppState{pool:p.clone()}),Json(payload(false))).await.status().is_success());
        let saved:String=sqlx::query_scalar("SELECT value FROM ABPrintSetting WHERE id='templateRoutes:LOCAL:L1'").fetch_one(&p).await.unwrap();
        let v:Value=serde_json::from_str(&saved).unwrap();
        assert_eq!(v["BAR"]["printer"],"Sticker");assert_eq!(v["PRICE"]["tray"],"Tray 2");
        let invalid:SaveLabelPrinterBody=serde_json::from_value(json!({"lbsCode":"L1","templateRoutes":{"OTHER":{"printer":"Wrong","tray":"","protocol":"WINDOWS"}}})).unwrap();
        assert_eq!(post_label_printers(State(AppState{pool:p.clone()}),Json(invalid)).await.status(),StatusCode::BAD_REQUEST);
        assert_eq!(sqlx::query_scalar::<_,String>("SELECT FTLbpPrinter FROM TCNMLabelPrinter WHERE FTLbpPriType='1'").fetch_one(&p).await.unwrap(),"Normal");
    }
    #[tokio::test]
    async fn event_routes_survive_legacy_manual_settings_save() {
        let p=fixture().await;
        assert!(post_label_printers(State(AppState{pool:p.clone()}),Json(payload(true))).await.status().is_success());
        assert!(post_label_printers(State(AppState{pool:p.clone()}),Json(payload(false))).await.status().is_success());
        let saved:String=sqlx::query_scalar("SELECT value FROM ABPrintSetting WHERE id='eventRoutes:LOCAL:L1'").fetch_one(&p).await.unwrap();
        let v:Value=serde_json::from_str(&saved).unwrap();
        assert_eq!(v["NEW_PDT"]["printer"],"New");
        assert_eq!(v["PROMO_END_REVERT"]["tray"],"Tray 1");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMLabelPrinter").fetch_one(&p).await.unwrap(),2);
    }
    #[tokio::test]
    async fn event_routes_failure_rolls_back_manual_mappings_too() {
        let p=fixture().await;
        sqlx::raw_sql("CREATE TRIGGER fail_routes BEFORE INSERT ON ABPrintSetting BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&p).await.unwrap();
        assert!(!post_label_printers(State(AppState{pool:p.clone()}),Json(payload(true))).await.status().is_success());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMLabelPrinter").fetch_one(&p).await.unwrap(),0);
    }
}
