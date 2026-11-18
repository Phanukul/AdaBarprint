#![allow(non_snake_case)]

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, NaiveTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use crate::barcode::validate_barcode;
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

async fn upsert_preprint_from_layout(
    pool: &SqlitePool,
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
) -> (String, Option<String>) {
    let mode = layout_val
        .get("designMode")
        .and_then(|v| v.as_str())
        .map(|s| if s == "preprint" { "preprint" } else { "full" })
        .unwrap_or("full")
        .to_string();

    let Some(pp_obj) = layout_val.get("preprint").and_then(|v| v.as_object()).cloned() else {
        return (mode, None);
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
        return (mode, existing_code);
    }

    let ppr_code = existing_code.unwrap_or_else(|| format!("PPR-{}", lbs_code));
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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let _ = sqlx::query(
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
    .execute(pool)
    .await;

    // Reference FTPprCode from layout metadata instead of duplicating large Base64 blobs in TCNMLabelTmpl
    if let Some(obj) = layout_val.as_object_mut() {
        if let Some(pp) = obj.get_mut("preprint").and_then(|v| v.as_object_mut()) {
            pp.insert("preprintCode".to_string(), json!(ppr_code));
            pp.remove("sheetBgDataUrl");
            pp.remove("cellBgDataUrl");
        }
    }

    (mode, Some(ppr_code))
}

async fn fetch_lan_printers(pool: &SqlitePool) -> Vec<Value> {
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
            let mut ip = "192.168.1.100".to_string();
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
        "SELECT FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode FROM TCNMLabelTmpl WHERE FTLtmStaUse = '1'",
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
    let tmpl_code = format!(
        "{}-DESIGN-{}",
        lbs_code,
        if pri_type == "2" { "PMT" } else { "NOR" }
    );
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

    let mut layout_val = body.layout.clone();
    let (ltm_mode, ppr_code_opt) = upsert_preprint_from_layout(
        &st.pool,
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
    .await;

    let layout_str = serde_json::to_string(&layout_val).unwrap_or_else(|_| "{}".to_string());
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let res = sqlx::query(
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
    .execute(&st.pool)
    .await;

    if let Err(e) = res {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let paper_size_opt = match body.paper_size.as_deref() {
        Some(raw) => Some(lookup_paper_size(&st.pool, Some(raw)).await.0),
        None => None,
    };
    let col_sql = if pri_type == "2" {
        r#"
        UPDATE TCNMLabelSize SET
            FTLbsTmplPmt = ?,
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
            FTLbsTmplNor = ?,
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
    let _ = sqlx::query(col_sql)
        .bind(&tmpl_code)
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
        .execute(&st.pool)
        .await;

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
    let (ltm_mode, ppr_code_opt) = upsert_preprint_from_layout(
        &st.pool,
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
    .await;

    let layout_str = serde_json::to_string(&layout_val).unwrap_or_else(|_| "{}".to_string());
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLabelPrinterBody {
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

    let mappings: Vec<Value> = sizes
        .into_iter()
        .map(|(code, name, grp)| {
            let nor = rows.iter().find(|r| r.0 == code && r.1 == "1");
            let pmt = rows.iter().find(|r| r.0 == code && r.1 == "2");
            json!({
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
    let server_code = body.server_code.unwrap_or_else(|| "LOCAL".to_string());
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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
        .execute(&st.pool)
        .await
        {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }

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
        "SELECT FTSopCode, FTSopApp, FTSopName, FTSopValue, FTSopDesc, FNSopSeq FROM TCNSSysOption ORDER BY FNSopSeq ASC, FTSopCode ASC",
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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    for item in body.options {
        let code = item.code.trim();
        if code.is_empty() {
            continue;
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
        let (paper_size, def_pw, def_ph, paper_win_kind) =
            lookup_paper_size(&st.pool, body.paper_size.as_deref()).await;
        let paper_width_mm = body.paper_width_mm.filter(|v| *v >= 30.0).unwrap_or(def_pw);
        let paper_height_mm = body.paper_height_mm.filter(|v| *v >= 30.0).unwrap_or(def_ph);

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
            "paperSize": paper_win_kind,
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

# ตั้งค่าขนาดกระดาษต่อ Job (A4 / B5 JIS / B5 ISO / Custom)
$targetPaper = $null
$wantPaper = $(if ($cfg.paperSize) { [string]$cfg.paperSize } else { 'A4' })
if ($wantPaper -eq 'B5_JIS' -or $wantPaper -eq 'B5') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.PaperName -match '\bB5\b' -and $_.PaperName -match 'JIS' } | Select-Object -First 1
  if (-not $targetPaper) {
    $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.Kind -eq [System.Drawing.Printing.PaperKind]::B5 -or $_.PaperName -match '\bB5\b' } | Select-Object -First 1
  }
} elseif ($wantPaper -eq 'B5_ISO') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.PaperName -match 'ISO.*B5|B5.*ISO' } | Select-Object -First 1
  if (-not $targetPaper) {
    $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.Kind -eq [System.Drawing.Printing.PaperKind]::B5 -or $_.PaperName -match '\bB5\b' } | Select-Object -First 1
  }
} elseif ($wantPaper -eq 'CUSTOM') {
  $pwMm = $(if ($cfg.paperWidthMm) { [double]$cfg.paperWidthMm } else { 210.0 })
  $phMm = $(if ($cfg.paperHeightMm) { [double]$cfg.paperHeightMm } else { 297.0 })
  $targetPaper = New-Object System.Drawing.Printing.PaperSize('AdaCustom', [int][Math]::Round($pwMm / 0.254), [int][Math]::Round($phMm / 0.254))
}
if (-not $targetPaper -and $wantPaper -eq 'A4') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.Kind -eq [System.Drawing.Printing.PaperKind]::A4 } | Select-Object -First 1
  if (-not $targetPaper) {
    $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.PaperName -match '^\s*A4\b' } | Select-Object -First 1
  }
}
if (-not $targetPaper) {
  $pwMm = $(if ($cfg.paperWidthMm) { [double]$cfg.paperWidthMm } else { 210.0 })
  $phMm = $(if ($cfg.paperHeightMm) { [double]$cfg.paperHeightMm } else { 297.0 })
  $targetPaper = New-Object System.Drawing.Printing.PaperSize($wantPaper, [int][Math]::Round($pwMm / 0.254), [int][Math]::Round($phMm / 0.254))
}
if ($targetPaper) {
  $doc.PrinterSettings.DefaultPageSettings.PaperSize = $targetPaper
  $doc.DefaultPageSettings.PaperSize = $targetPaper
}

# เลือกถาดกระดาษลง DEVMODE ของ Job นี้โดยตรง (ไม่ยุ่ง PrintConfiguration ส่วนกลาง -> ไม่มี race condition)
$matchedTray = $null
$ticketBin = ''
if ($cfg.trayName -and $cfg.trayName.Trim() -ne '') {
  $wantTray = $cfg.trayName.Trim()
  $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -ieq $wantTray } | Select-Object -First 1
  if (-not $matchedTray) {
    $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -like ($wantTray + '*') } | Select-Object -First 1
  }
  if ($matchedTray) {
    $doc.PrinterSettings.DefaultPageSettings.PaperSource = $matchedTray
    $doc.DefaultPageSettings.PaperSource = $matchedTray
    $resolvedTray = $matchedTray.SourceName
    $ticketBin = "DEVMODE:RawKind=" + $matchedTray.RawKind
  }
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

  if ($isVirtual) {
    $bounds = $e.PageBounds
    $g.DrawImage($img, [float]0, [float]0, [float]$bounds.Width, [float]$bounds.Height)
  } else {
    $safePad = 6.25
    $availW = [Math]::Max(100.0, [double]$vcb.Width - ($safePad * 2.0))
    $availH = [Math]::Max(100.0, [double]$vcb.Height - ($safePad * 2.0))
    $ratio = [Math]::Min($availW / [double]$img.Width, $availH / [double]$img.Height)
    $dw = [double]$img.Width * $ratio
    $dh = [double]$img.Height * $ratio
    $dx = [double]$vcb.X + (([double]$vcb.Width - $dw) / 2.0)
    $dy = [double]$vcb.Y + (([double]$vcb.Height - $dh) / 2.0)
    $g.DrawImage($img, [float]$dx, [float]$dy, [float]$dw, [float]$dh)
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
        ps_bytes.extend_from_slice(ps_script.as_bytes());
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
    pub limit: Option<i64>,
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
    let limit = q.limit.unwrap_or(200).clamp(1, 500);
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
                   COALESCE((SELECT CAST(pr.FCPgdPriceRet AS REAL) FROM TCNTPdtPrice4PDT pr WHERE pr.FTPdtCode = b.FTPdtCode AND pr.FTPghDocType = '1'
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
                    ORDER BY pr.FDPghDStart DESC, pr.FDCreateOn DESC LIMIT 1), 0.0) AS FCPdtPrice
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

        if !keyword.is_empty() {
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
            qb.push(" AND EXISTS (SELECT 1 FROM TCNTPdtPmtDT dtp JOIN TCNTPdtPmtHD hdp ON dtp.FTPmhDocNo = hdp.FTPmhDocNo AND dtp.FTBchCode = hdp.FTBchCode WHERE (dtp.FTPmdBarCode = b.FTBarCode OR dtp.FTPmdRefCode = b.FTPdtCode OR dtp.FTPmdBarCode = '*') AND COALESCE(hdp.FTPmhStaApv, '1') = '1' AND COALESCE(hdp.FTPmhStaClosed, '2') != '1'");
            push_range_filter(&mut qb, "hdp.FTBchCode", &bch_from, &bch_to);
            qb.push(" AND (COALESCE(hdp.FDPmhDStart, '') = '' OR substr(hdp.FDPmhDStart, 1, 10) <= ")
                .push_bind(&eff_cmp)
                .push(") AND (COALESCE(hdp.FDPmhDStop, '') = '' OR substr(hdp.FDPmhDStop, 1, 10) >= ")
                .push_bind(&eff_cmp)
                .push("))");
        }

        qb.push(" GROUP BY b.FTPdtCode, b.FTBarCode ORDER BY b.FTPdtCode ASC, b.FTBarCode ASC LIMIT ")
            .push_bind(limit);

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

        let mock_prices: Vec<(String, f64)> = sqlx::query_as(
            "SELECT FTPdtCode, FCPpcPrice FROM TCNMPdtPri WHERE FTPpcPriType = '1' AND FTPpcStaUse = '1'",
        )
        .fetch_all(&st.pool)
        .await
        .unwrap_or_default();
        let mock_price_map: HashMap<String, f64> = mock_prices.into_iter().collect();

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
                let retail_price = if p > 0.0 {
                    p
                } else {
                    mock_price_map.get(&r.FTPdtCode).copied().unwrap_or(0.0)
                };
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
    pub qty: Option<i64>,
    pub bch_code: String,
    pub pri_type: String,
    pub verify: Option<bool>,
    pub override_lbs: Option<String>,
    pub effective_date: Option<String>,
    pub price_group: Option<String>,
}

pub async fn post_build_line(
    State(st): State<AppState>,
    Json(body): Json<BuildLineBody>,
) -> Response {
    if body.barcode.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "barcode is required");
    }
    let qty = body.qty.unwrap_or(1);
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
            WHERE b.FTBarCode = ? AND COALESCE(b.FTBarStaUse, '1') = '1'
            LIMIT 1
            "#,
        )
        .bind(&body.barcode)
        .fetch_optional(accd)
        .await
        .unwrap_or(None);

        let Some(item) = bar_pdt else {
            return err_json(StatusCode::NOT_FOUND, format!("Barcode not found in AdaAccD: {}", body.barcode));
        };

        // Price lookup: TCNTPdtPrice4PDT -> TCNMPdtPackSize -> 0.0
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
              AND (FTPplCode = ? OR FTPplCode = '')
              AND (COALESCE(FDPghDStart, '') = '' OR substr(FDPghDStart, 1, 10) <= ?)
              AND (COALESCE(FDPghDStop, '') = '' OR substr(FDPghDStop, 1, 10) >= ?)
            ORDER BY CASE WHEN FTPplCode = ? THEN 0 ELSE 1 END, FDPghDStart DESC, FTPghTStart DESC
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

        let mut sale_price = if body.pri_type == "2" {
            p4.as_ref()
                .and_then(|r| r.FCPgdPriceWhs)
                .or(item.FCPdtPriceWHS)
                .or_else(|| p4.as_ref().and_then(|r| r.FCPgdPriceRet))
                .or(item.FCPdtPriceRET)
                .unwrap_or(0.0)
        } else {
            p4.as_ref()
                .and_then(|r| r.FCPgdPriceRet)
                .or(item.FCPdtPriceRET)
                .unwrap_or(0.0)
        };
        if sale_price <= 0.0 {
            let mock_pri: Option<(f64,)> = sqlx::query_as(
                "SELECT FCPpcPrice FROM TCNMPdtPri WHERE FTPdtCode = ? AND FTPpcStaUse = '1' ORDER BY CASE WHEN FTBchCode = ? THEN 0 ELSE 1 END LIMIT 1",
            )
            .bind(&item.FTPdtCode)
            .bind(&body.bch_code)
            .fetch_optional(&st.pool)
            .await
            .unwrap_or(None);
            if let Some((mp,)) = mock_pri {
                sale_price = mp;
            }
        }
        let unit_price = sale_price;

        let mut promo = resolve_accd_promotion(accd, &body.barcode, &item.FTPdtCode, on_date, verify)
            .await
            .unwrap_or(None);
        if promo.is_none() {
            if let Ok(Some(local_promo)) = crate::db::resolve_promotion(&st.pool, &body.barcode, on_date, verify).await {
                promo = Some(local_promo);
            } else if let Ok(Some(local_promo_by_code)) = crate::db::resolve_promotion(&st.pool, &item.FTPdtCode, on_date, verify).await {
                promo = Some(local_promo_by_code);
            }
        }
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
        let pri_type_name = if body.pri_type == "2" { "ขายส่ง" } else { "ปกติ" };
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

        let url_tmpl: String = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'UrlTemplate' LIMIT 1",
        )
        .fetch_optional(&st.pool)
        .await
        .ok()
        .flatten()
        .and_then(|r| r.0)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://firster.com/product/{code}/?branch={bch}".to_string());
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

    if let Err(e) = sqlx::query(
        "INSERT INTO TCNTLabelJob (FTLjbJobID, FTLjbBchCode, FTLjbSource, FTLjbPriType, FDLjbFrom, FTLjbStaVfy, FTLjbLbsOvr, FTLjbStatus, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, '1', ?, '4', ?, ?, ?, ?)",
    )
    .bind(&job_id).bind(&body.bch_code).bind(body.source.as_deref().unwrap_or("1")).bind(&first_pri_type)
    .bind(&now).bind(&body.r#override).bind(&now).bind(&created_by).bind(&now).bind(&created_by)
    .execute(&st.pool)
    .await
    {
        return err_json(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let mut group_map: BTreeMap<String, (String, String, i64, bool)> = BTreeMap::new();
    let mut seq: i64 = 0;

    for line in &body.lines {
        let qty = line.qty.unwrap_or(1).max(1);
        let is_promo = line.promo.as_ref().map(|p| {
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
        .fetch_optional(&st.pool)
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

            let _ = sqlx::query(
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
            .execute(&st.pool)
            .await;
        }
    }

    let mut log_entries = Vec::new();
    for (_k, (lbs_code, pri_type, total_qty, is_promo)) in group_map {
        let label_size = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&lbs_code)
        .fetch_optional(&st.pool)
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

        let _ = sqlx::query(
            "INSERT INTO TCNTLabelLog (FTLlgLogID, FTLjbJobID, FTLlgLbsCode, FTLlgPriTyp, FTLlgPsvCode, FTLlgPrinter, FNLlgQtyLabel, FNLlgQtyPage, FDLlgPriOn, FTLlgPriBy, FTLlgStatus, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '4', ?, ?, ?, ?)",
        )
        .bind(&log_id).bind(&job_id).bind(&lbs_code).bind(&pri_type)
        .bind(psv_code).bind(target_prt)
        .bind(total_qty).bind(qty_page).bind(&now).bind(&created_by).bind(&now).bind(&created_by).bind(&now).bind(&created_by)
        .execute(&st.pool)
        .await;

        log_entries.push(json!({
            "lbsCode": lbs_code,
            "priType": pri_type,
            "totalQty": total_qty,
            "qtyPage": qty_page,
        }));
    }

    let job_end = Utc::now().to_rfc3339();
    let _ = sqlx::query("UPDATE TCNTLabelJob SET FDLjbTo = ? WHERE FTLjbJobID = ?")
        .bind(&job_end).bind(&job_id)
        .execute(&st.pool)
        .await;

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
        let mut groups: BTreeMap<String, Vec<(Value, u32)>> = BTreeMap::new();
        for l in lines {
            if !l.get("selected").and_then(|v| v.as_bool()).unwrap_or(false) {
                continue;
            }
            let lbs = l.get("lbsCode").and_then(|v| v.as_str()).unwrap_or("SH-100x38");
            let pri = l.get("priType").and_then(|v| v.as_str()).unwrap_or("1");
            let qty = l.get("qty").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let ctx = l.get("ctx").cloned().unwrap_or(json!({}));
            groups.entry(format!("{}_{}", lbs, pri)).or_default().push((ctx, qty));
        }

        let templates = get_templates();
        let mut parts = Vec::new();
        for (key, lbls) in groups {
            let (lbs_code, pri_type) = key.split_once('_').unwrap_or(("SH-100x38", "1"));
            let group_lbs = sqlx::query_as::<_, TCNMLabelSize>(
                "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
            )
            .bind(lbs_code)
            .fetch_optional(&st.pool)
            .await
            .unwrap_or(None);

            let Some(group_lbs) = group_lbs else { continue };
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
                gap_mm: 2.0,
                speed,
                darkness,
                copies: 1,
            };
            parts.push(generate_commands(brand, &lbls, &tmpl.elements, &opts));
        }
        parts.join("\r\n\r\n")
    } else {
        return err_json(StatusCode::BAD_REQUEST, "ไม่มีข้อมูลสำหรับพิมพ์");
    };

    if command_data.is_empty() {
        return err_json(StatusCode::BAD_REQUEST, "ไม่สามารถสร้างคำสั่งพิมพ์ได้");
    }

    let (success, message, bytes_sent) = if preview_only || brand == PrinterBrand::Samsung {
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
            let ezpl = generate_commands(PrinterBrand::Godex, &[(ctx, 1)], &tmpl.elements, &opts);
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

        let Some(group_lbs) = group_lbs else { continue };
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
        parts.push(generate_commands(PrinterBrand::Godex, &lbls, &tmpl.elements, &opts));
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
    post_print_send(State(st), Json(body)).await
}

// ─── /api/import ─────────────────────────────────────────────────────────────

pub async fn post_import(
    State(st): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    let mut csv_bytes: Option<Vec<u8>> = None;
    let mut table = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            if let Ok(data) = field.bytes().await {
                csv_bytes = Some(data.to_vec());
            }
        } else if name == "table" {
            if let Ok(txt) = field.text().await {
                table = txt;
            }
        }
    }

    let Some(bytes) = csv_bytes else {
        return err_json(StatusCode::BAD_REQUEST, "file is required");
    };
    let supported = [
        "TCNMBranch", "TCNMPdt", "TCNMPdtBar", "TCNMPdtPri",
        "TCNMPmtHD", "TCNMPmtDT", "TCNMPdtLabel", "TCNMLabelSize",
    ];
    if !supported.contains(&table.as_str()) {
        return err_json(
            StatusCode::BAD_REQUEST,
            format!("Unsupported table. Supported: {}", supported.join(", ")),
        );
    }

    let mut rdr = csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(bytes.as_slice());
    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect(),
        Err(e) => return err_json(StatusCode::BAD_REQUEST, format!("Invalid CSV: {}", e)),
    };

    let now = Utc::now().to_rfc3339();
    let mut ok = 0i64;
    let mut fail = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (idx, rec_res) in rdr.records().enumerate() {
        let row_num = idx + 2;
        let record = match rec_res {
            Ok(r) => r,
            Err(e) => {
                fail += 1;
                errors.push(format!("Row {}: {}", row_num, e));
                continue;
            }
        };

        let mut map: HashMap<String, String> = HashMap::new();
        for (i, h) in headers.iter().enumerate() {
            if let Some(val) = record.get(i) {
                if !val.is_empty() {
                    map.insert(h.clone(), val.to_string());
                }
            }
        }

        if table == "TCNMPdtBar" {
            if let Some(bc) = map.get("FTPdbBarCode") {
                let sym = map.get("FTPdbSymbology").map(|s| s.as_str()).unwrap_or("EAN13");
                if let Err(e) = validate_barcode(bc, sym) {
                    fail += 1;
                    errors.push(format!("Row {}: {} (barcode: {})", row_num, e, bc));
                    continue;
                }
            }
        }

        if table == "TCNMPmtDT" {
            if let Some(bc) = map.get("FTBarCode").filter(|s| s.as_str() != "*") {
                if let Err(e) = validate_barcode(bc, "EAN13") {
                    fail += 1;
                    errors.push(format!("Row {}: Barcode invalid in promotion detail: {}", row_num, e));
                    continue;
                }
            }
        }

        let res = match table.as_str() {
            "TCNMBranch" => {
                sqlx::query(
                    "INSERT INTO TCNMBranch (FTBchCode, FTBchName, FTBchStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, 'IMPORT', ?, 'IMPORT') ON CONFLICT(FTBchCode) DO UPDATE SET FTBchName = COALESCE(excluded.FTBchName, FTBchName), FTBchStaUse = excluded.FTBchStaUse, FDLastUpdOn = excluded.FDLastUpdOn, FTLastUpdBy = 'IMPORT'"
                )
                .bind(map.get("FTBchCode").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTBchName"))
                .bind(map.get("FTBchStaUse").map(|s| s.as_str()).unwrap_or("1"))
                .bind(&now).bind(&now)
                .execute(&st.pool).await
            }
            "TCNMPdt" => {
                sqlx::query(
                    "INSERT INTO TCNMPdt (FTPdtCode, FTPdtName, FTPdtNameOth, FTPdtSku, FTPdtBrand, FTPdtCgyName, FTPdtUnit, FTPdtDim, FTPdtStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'IMPORT', ?, 'IMPORT') ON CONFLICT(FTPdtCode) DO UPDATE SET FTPdtName = COALESCE(excluded.FTPdtName, FTPdtName), FTPdtNameOth = COALESCE(excluded.FTPdtNameOth, FTPdtNameOth), FTPdtSku = COALESCE(excluded.FTPdtSku, FTPdtSku), FTPdtBrand = COALESCE(excluded.FTPdtBrand, FTPdtBrand), FTPdtCgyName = COALESCE(excluded.FTPdtCgyName, FTPdtCgyName), FTPdtUnit = COALESCE(excluded.FTPdtUnit, FTPdtUnit), FTPdtDim = COALESCE(excluded.FTPdtDim, FTPdtDim), FDLastUpdOn = excluded.FDLastUpdOn"
                )
                .bind(map.get("FTPdtCode").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTPdtName").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTPdtNameOth"))
                .bind(map.get("FTPdtSku"))
                .bind(map.get("FTPdtBrand"))
                .bind(map.get("FTPdtCgyName"))
                .bind(map.get("FTPdtUnit"))
                .bind(map.get("FTPdtDim"))
                .bind(map.get("FTPdtStaUse").map(|s| s.as_str()).unwrap_or("1"))
                .bind(&now).bind(&now)
                .execute(&st.pool).await
            }
            "TCNMPdtBar" => {
                sqlx::query(
                    "INSERT INTO TCNMPdtBar (FTPdbBarCode, FTPdtCode, FTPdbSymbology, FTPdbStaPri, FTPdbStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, 'IMPORT', ?, 'IMPORT') ON CONFLICT(FTPdbBarCode) DO UPDATE SET FTPdtCode = excluded.FTPdtCode, FTPdbSymbology = excluded.FTPdbSymbology, FTPdbStaPri = excluded.FTPdbStaPri, FDLastUpdOn = excluded.FDLastUpdOn"
                )
                .bind(map.get("FTPdbBarCode").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTPdtCode").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTPdbSymbology").map(|s| s.as_str()).unwrap_or("EAN13"))
                .bind(map.get("FTPdbStaPri").map(|s| s.as_str()).unwrap_or("1"))
                .bind(map.get("FTPdbStaUse").map(|s| s.as_str()).unwrap_or("1"))
                .bind(&now).bind(&now)
                .execute(&st.pool).await
            }
            "TCNMPdtPri" => {
                let price: f64 = map.get("FCPpcPrice").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let unit_price: Option<f64> = map.get("FCPpcUnitPrice").and_then(|s| s.parse().ok());
                sqlx::query(
                    "INSERT INTO TCNMPdtPri (FTPdtCode, FTBchCode, FTPpcPriType, FCPpcPrice, FCPpcUnitPrice, FTPpcStaUse, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, 'IMPORT', ?, 'IMPORT') ON CONFLICT(FTPdtCode, FTBchCode, FTPpcPriType) DO UPDATE SET FCPpcPrice = excluded.FCPpcPrice, FCPpcUnitPrice = excluded.FCPpcUnitPrice, FDLastUpdOn = excluded.FDLastUpdOn"
                )
                .bind(map.get("FTPdtCode").map(|s| s.as_str()).unwrap_or(""))
                .bind(map.get("FTBchCode").map(|s| s.as_str()).unwrap_or("00001"))
                .bind(map.get("FTPpcPriType").map(|s| s.as_str()).unwrap_or("1"))
                .bind(price).bind(unit_price)
                .bind(map.get("FTPpcStaUse").map(|s| s.as_str()).unwrap_or("1"))
                .bind(&now).bind(&now)
                .execute(&st.pool).await
            }
            _ => {
                // Other supported tables fallback
                sqlx::query("SELECT 1").execute(&st.pool).await
            }
        };

        match res {
            Ok(_) => ok += 1,
            Err(e) => {
                fail += 1;
                errors.push(format!("Row {}: {}", row_num, e));
            }
        }
    }

    Json(json!({ "ok": ok, "fail": fail, "errors": errors })).into_response()
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
    if body.width_mm < 30.0 || body.height_mm < 30.0 {
        return err_json(StatusCode::BAD_REQUEST, "ขนาดกระดาษต้องไม่น้อยกว่า 30 mm");
    }
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

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

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = tokio::process::Command::new("curl.exe");
        c.creation_flags(0x08000000); // CREATE_NO_WINDOW
        c
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = tokio::process::Command::new("curl");

    cmd.args(["-sSL", "--fail", "--max-time", "8", target]);
    match cmd.output().await {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {
            if let Some(mime) = sniff_image_mime(&out.stdout) {
                axum::http::Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime)
                    .header(header::CACHE_CONTROL, "public, max-age=300")
                    .body(axum::body::Body::from(out.stdout))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            } else {
                err_json(StatusCode::BAD_GATEWAY, "remote resource is not a supported image")
            }
        }
        Ok(_) => err_json(StatusCode::BAD_GATEWAY, "failed to fetch remote image"),
        Err(e) => err_json(StatusCode::INTERNAL_SERVER_ERROR, format!("proxy exec failed: {}", e)),
    }
}
