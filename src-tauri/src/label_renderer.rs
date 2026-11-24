use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::barcode::encode_barcode_modules;
use crate::models::{TCNMLabelPrinter, TCNMLabelSize, TCNMLabelTmpl, TCNTLabelSchedQueue};
use crate::print_drivers::{generate_commands, tcp_send, tcp_test, DriverOptions, PrinterBrand};
use crate::templates::{get_templates, TemplateElement};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintDispatchSummary {
    pub job_id: Option<String>,
    pub total_ready: usize,
    pub printed_normal: i64,
    pub printed_promo: i64,
    pub failed_count: i64,
    pub run_status: String, // SUCCESS | PARTIAL | FAILED | SKIPPED_EMPTY
    pub err_code: Option<String>,
    pub err_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPrinterRoute {
    pub lbs_code: String,
    pub pri_type: String,
    pub protocol: String, // PDF/WINDOWS vs ZPL/EZPL/TSPL/...
    pub printer_name: String,
    pub tray_name: String,
    pub lan_ip: Option<String>,
    pub lan_port: u16,
    pub dpi: u32,
}

fn gen_guid() -> String {
    Uuid::new_v4().to_string().replace('-', "").to_uppercase()
}

pub async fn resolve_printer_route(
    pool: &SqlitePool,
    lbs_code: &str,
    pri_type: &str,
) -> ResolvedPrinterRoute {
    let mapping = sqlx::query_as::<_, TCNMLabelPrinter>(
        r#"
        SELECT FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse,
               FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy
        FROM TCNMLabelPrinter
        WHERE FTLbsCode = ? AND FTLbpPriType = ? AND COALESCE(FTLbpStaUse, '1') = '1'
        ORDER BY CASE WHEN FTPsvCode = 'LOCAL' THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(lbs_code)
    .bind(pri_type)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let default_prt: String = sqlx::query_scalar::<_, Option<String>>(
        "SELECT FTSopValue FROM TCNSSysOption WHERE FTSopCode = 'DefaultPrinter' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default();

    let (printer_name, tray_name, protocol) = match mapping {
        Some(m) => (
            m.FTLbpPrinter.filter(|s| !s.trim().is_empty()).unwrap_or(default_prt),
            m.FTLbpTray.unwrap_or_default(),
            m.FTLbpProtocol,
        ),
        None => (default_prt, String::new(), "PDF".to_string()),
    };

    // Use the same persisted printer schema and port metadata as Manual.
    let printers = crate::routes::fetch_lan_printers(pool).await;
    let lan = printers.iter().find(|p| {
        p.get("id").and_then(Value::as_str) == Some(printer_name.as_str())
            || p.get("name").and_then(Value::as_str) == Some(printer_name.as_str())
    });
    let (lan_ip, lan_port, resolved_proto, dpi) = if let Some(lp) = lan {
        let ip = lp["ip"].as_str().unwrap_or_default().to_string();
        (
            if !ip.is_empty() { Some(ip) } else { None },
            lp["port"].as_u64().and_then(|p| u16::try_from(p).ok()).unwrap_or(9100),
            lp["brand"].as_str().unwrap_or(&protocol).to_string(),
            lp["dpi"].as_u64().and_then(|d| u32::try_from(d).ok()).unwrap_or(203),
        )
    } else {
        (None, 9100, protocol, 203)
    };

    ResolvedPrinterRoute {
        lbs_code: lbs_code.to_string(),
        pri_type: pri_type.to_string(),
        protocol: resolved_proto,
        printer_name,
        tray_name,
        lan_ip,
        lan_port,
        dpi,
    }
}

async fn load_template_elements(pool: &SqlitePool, lbs: Option<&TCNMLabelSize>, lbs_code: &str, pri_type: &str) -> (String, Vec<TemplateElement>) {
    let tmpl_code = lbs
        .and_then(|s| if pri_type == "2" { s.FTLbsTmplPmt.clone() } else { s.FTLbsTmplNor.clone() })
        .unwrap_or_else(|| format!("{}-{}", lbs_code, if pri_type == "2" { "PMT" } else { "NOR" }));

    if let Ok(Some(row)) = sqlx::query_as::<_, TCNMLabelTmpl>(
        "SELECT FTLtmCode, FTLbsCode, FTLtmPriType, FTLtmName, FTLtmLayout, FTLtmStaUse, FTLtmMode, FTPprCode FROM TCNMLabelTmpl WHERE FTLtmCode = ? OR (FTLbsCode = ? AND FTLtmPriType = ?) ORDER BY CASE WHEN FTLtmCode = ? THEN 0 ELSE 1 END LIMIT 1",
    )
    .bind(&tmpl_code)
    .bind(lbs_code)
    .bind(pri_type)
    .bind(&tmpl_code)
    .fetch_optional(pool)
    .await
    {
        if let Some(ref layout_json) = row.FTLtmLayout {
            if let Ok(val) = serde_json::from_str::<Value>(layout_json) {
                if let Some(els) = val.get("elements").cloned() {
                    if let Ok(parsed) = serde_json::from_value::<Vec<TemplateElement>>(els) {
                        return (row.FTLtmCode, parsed);
                    }
                }
            }
        }
    }

    let builtins = get_templates();
    let matched = builtins
        .iter()
        .find(|t| t.code == tmpl_code)
        .unwrap_or(&builtins[0]);
    (matched.code.clone(), matched.elements.clone())
}

pub(crate) async fn build_label_context(
    pool: &SqlitePool,
    row: &TCNTLabelSchedQueue,
    pdt_name: &str,
    store_name: &str,
    biz_date: &str,
) -> Value {
    let is_promo = row.FTLsqPriType == "2";
    let print_date_th = chrono::NaiveDate::parse_from_str(biz_date, "%Y-%m-%d")
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| biz_date.to_string());

    let unit_dim: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT pl.FTPdtNameOth, ul.FTPunName, ps.FTPdtUnitDim
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
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let (name_oth, selling_unit, dim_str) = match unit_dim {
        Some((no, su, dm)) => (no.unwrap_or_default(), su.unwrap_or_default(), dm.unwrap_or_default()),
        None => (String::new(), String::new(), String::new()),
    };

    let (promo_name, promo_interval) = if is_promo {
        if let Some(ref doc_no) = row.FTLsqPromoDocNo {
            let v6: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT COALESCE(NULLIF(hdl.FTPmhName, ''), hd.FTPmhName, hd.FTPmhDocNo), substr(CAST(hd.FDPmhDStart AS TEXT), 1, 10), substr(CAST(hd.FDPmhDStop AS TEXT), 1, 10) FROM TCNTPmtHD hd LEFT JOIN TCNTPmtHD_L hdl ON hdl.FTAgnCode = hd.FTAgnCode AND hdl.FTPmhDocNo = hd.FTPmhDocNo AND hdl.FNPmhVersion = hd.FNPmhVersion AND hdl.FNLngID = 1 WHERE hd.FTPmhDocNo = ? LIMIT 1",
            )
            .bind(doc_no)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
            let pm = if v6.is_some() {
                v6
            } else {
                sqlx::query_as(
                    "SELECT COALESCE(hdl.FTPmhName, hd.FTPmhDocNo), substr(CAST(hd.FDPmhDStart AS TEXT), 1, 10), substr(CAST(hd.FDPmhDStop AS TEXT), 1, 10) FROM TCNTPdtPmtHD hd LEFT JOIN TCNTPdtPmtHD_L hdl ON hdl.FTBchCode = hd.FTBchCode AND hdl.FTPmhDocNo = hd.FTPmhDocNo AND hdl.FNLngID = 1 WHERE hd.FTPmhDocNo = ? LIMIT 1",
                )
                .bind(doc_no)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
            };
            match pm {
                Some((nm, ds, de)) => (
                    nm.unwrap_or_else(|| doc_no.clone()),
                    crate::db::promo_interval(ds.as_deref(), de.as_deref()),
                ),
                None => (doc_no.clone(), String::new()),
            }
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    let sym = if row.FTBarCode.len() == 13 && row.FTBarCode.chars().all(|c| c.is_ascii_digit()) {
        "EAN13"
    } else {
        "CODE128"
    };
    let bar_bits = encode_barcode_modules(&row.FTBarCode, sym);
    let has_discount = is_promo && row.FCLsqOldPrice.map(|op| op > row.FCLsqSalePrice + 0.001).unwrap_or(true);
    let is_gift = is_promo && !has_discount;

    json!({
        "product": {
            "code": row.FTPdtCode,
            "barcode": row.FTBarCode,
            "bar_bits": bar_bits,
            "name": pdt_name,
            "name_oth": name_oth,
            "dim": dim_str,
            "sub_dept": "",
            "reple_type": ""
        },
        "price": {
            "value": format!("{:.2}", row.FCLsqSalePrice),
            "old_value": row.FCLsqOldPrice.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            "old_has": row.FCLsqOldPrice.is_some()
        },
        "promotion": {
            "is_promo": is_promo,
            "is_discount": has_discount,
            "is_gift": is_gift,
            "text": if is_promo { "PROMOTION" } else { "" },
            "free_text": if is_gift { promo_name.as_str() } else { "" },
            "name": promo_name,
            "interval": promo_interval
        },
        "tag": {
            "print_date": print_date_th,
            "selling_unit": selling_unit,
            "pri_per_unit": "",
            "pri_status": if is_promo { "2" } else { "1" },
            "sub_dept": "",
            "phasing": ""
        },
        "store": {
            "name": store_name
        }
    })
}

/// Check printer connectivity (`Wake Ping`) before sending print chunks.
pub async fn check_printer_ready(route: &ResolvedPrinterRoute) -> Result<(), (String, String)> {
    if route.printer_name.trim().is_empty() {
        return Err(("E-PRT-001".to_string(), format!("ยังไม่ได้ตั้งค่าเครื่องพิมพ์สำหรับป้าย {} ประเภท {}", route.lbs_code, route.pri_type)));
    }
    if let Some(ref ip) = route.lan_ip {
        let (ok, msg, _) = tcp_test(ip, route.lan_port).await;
        if !ok {
            return Err(("E-PRT-001".to_string(), format!("เครื่องพิมพ์ LAN {}:{} ไม่ตอบสนอง: {}", ip, route.lan_port, msg)));
        }
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("powershell");
        cmd.creation_flags(0x08000000);
        let safe_name = route.printer_name.replace('\'', "''");
        let ps = format!(
            "$p = Get-CimInstance Win32_Printer -Filter \"Name='{}'\" -ErrorAction SilentlyContinue; \
             if (-not $p) {{ Write-Output 'NOT_FOUND'; exit 2 }}; \
             if ($p.WorkOffline -eq $true -or $p.PrinterStatus -eq 7 -or $p.DetectedErrorState -in 4,5,6,7,8,9,10) {{ \
               Write-Output ('OFFLINE:' + $p.PrinterStatus + ':' + $p.DetectedErrorState); exit 3 \
             }}; \
             Write-Output 'OK'",
            safe_name
        );
        if let Ok(out) = cmd.args(["-NoProfile", "-NonInteractive", "-Command", &ps]).output().await {
            let txt = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if txt == "OK" {
                return Ok(());
            }
            return Err(("E-PRT-002".to_string(), format!("เครื่องพิมพ์ Windows '{}' ไม่พร้อม ({})", route.printer_name, txt)));
        }
    }
    Ok(())
}

/// Purge stale AdaBarprint_Auto print jobs from Windows Spooler when mid-print failure happens (prevents double printing).
pub async fn cancel_stale_spooler_jobs(printer_name: &str) {
    if printer_name.trim().is_empty() {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("powershell");
        cmd.creation_flags(0x08000000);
        let safe = printer_name.replace('\'', "''");
        let ps = format!(
            "Get-PrintJob -PrinterName '{}' -ErrorAction SilentlyContinue | Where-Object {{ $_.DocumentName -like 'AdaBarprint_Auto*' }} | Remove-PrintJob -ErrorAction SilentlyContinue",
            safe
        );
        let _ = cmd.args(["-NoProfile", "-NonInteractive", "-Command", &ps]).output().await;
    }
}

/// Prepare QR matrices and image bytes before opening the Windows print document.
#[cfg(target_os = "windows")]
async fn windows_artwork(elements: &[TemplateElement], items: &[Value]) -> Result<Vec<Value>, String> {
    use base64::Engine;
    let mut cache = std::collections::HashMap::<(String,String),Value>::new();
    let mut output = Vec::new();
    for item in items {
        let mut art = serde_json::Map::new();
        for (index, el) in elements.iter().enumerate() {
            if let Some(when) = &el.when {
                let flag = when.path.split('.').try_fold(item, |v,k| v.get(k)).and_then(Value::as_bool);
                if flag != Some(when.eq) { continue; }
            }
            if el.el_type == "barcode" {
                let path = el.field.as_deref().unwrap_or("product.barcode");
                let value = match path.split('.').try_fold(item, |v,k| v.get(k)) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    _ => String::new(),
                };
                let bits = encode_barcode_modules(&value, el.symbology.as_deref().unwrap_or("EAN13"));
                if bits.is_empty() { return Err(format!("ข้อมูลบาร์โค้ดว่างหรือเข้ารหัสไม่ได้ ({path})")); }
                art.insert(index.to_string(), json!({"bits":bits,"value":value}));
                continue;
            }
            if !matches!(el.el_type.as_str(),"image"|"qrcode") { continue; }
            let value = el.src.as_deref().or_else(||el.field.as_deref()
                .and_then(|path|path.split('.').try_fold(item,|parent,key|parent.get(key)))
                .and_then(Value::as_str)).or(el.text.as_deref()).unwrap_or("").trim();
            if value.is_empty() {
                if el.el_type == "qrcode" { return Err("ข้อมูล QR Code ว่าง".into()); }
                continue;
            }
            let key = (el.el_type.clone(),value.to_owned());
            if !cache.contains_key(&key) {
                let data = if el.el_type=="qrcode" {
                    let qr = qrcode::QrCode::new(value.as_bytes()).map_err(|e|e.to_string())?;
                    let width = qr.width();
                    let rows:Vec<String> = qr.to_colors().chunks(width).map(|row|row.iter().map(|c|
                        if *c==qrcode::Color::Dark {'1'} else {'0'}).collect()).collect();
                    json!(rows)
                } else if value.starts_with("data:image/") {
                    let (meta,bytes)=value.split_once(',').ok_or("ข้อมูลรูปภาพไม่ครบ")?;
                    if !meta.ends_with(";base64") {return Err("รูปภาพต้องใช้ base64".into());}
                    json!(bytes)
                } else {
                    let response=crate::routes::get_image_proxy(axum::extract::Query(crate::routes::ImageProxyQuery{url:value.to_owned()})).await;
                    if !response.status().is_success(){return Err("อ่านรูปสินค้าสำหรับงานพิมพ์ไม่ได้".into());}
                    let bytes=axum::body::to_bytes(response.into_body(),16*1024*1024).await.map_err(|e|e.to_string())?;
                    json!(base64::engine::general_purpose::STANDARD.encode(bytes))
                };
                cache.insert(key.clone(),data);
            }
            art.insert(index.to_string(),cache[&key].clone());
        }
        output.push(Value::Object(art));
    }
    Ok(output)
}

/// Headless Windows GDI page chunk print directly from `TemplateElement` + product `ctx` (no React UI / browser needed!).
#[cfg(target_os = "windows")]
pub(crate) async fn send_windows_headless_page(
    route: &ResolvedPrinterRoute,
    lbs: Option<&TCNMLabelSize>,
    elements: &[TemplateElement],
    page_items: &[Value],
    doc_name: &str,
) -> Result<(), String> {
    let artwork = windows_artwork(elements,page_items).await?;
    let job_uid = Uuid::new_v4().to_string();
    let tmp_dir = std::env::temp_dir().join(format!("ada_auto_prt_{}", job_uid));
    let _ = std::fs::create_dir_all(&tmp_dir);

    let export_dir = crate::db::resolve_exports_dir();
    let _ = std::fs::create_dir_all(&export_dir);
    let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let safe_doc: String = doc_name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
    let clean_export = std::fs::canonicalize(&export_dir)
        .unwrap_or(export_dir)
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string();
    let pdf_out = std::path::PathBuf::from(clean_export)
        .join(format!("{}_{}_{}.pdf", safe_doc, ts, &job_uid[..6]));

    let payload = json!({
        "printerName": route.printer_name,
        "trayName": route.tray_name,
        "docName": doc_name,
        "pdfTarget": pdf_out.to_string_lossy(),
        "paperSize": lbs.and_then(|s| s.FTLbsPaperSize.as_deref()).unwrap_or("A4"),
        "paperWidthMm": lbs.and_then(|s| s.FCLbsPaperWidth).unwrap_or(210.0),
        "paperHeightMm": lbs.and_then(|s| s.FCLbsPaperHeight).unwrap_or(297.0),
        "cols": lbs.map(|s| s.FNLbsCol).unwrap_or(2).max(1),
        "rows": lbs.map(|s| s.FNLbsRow).unwrap_or(7).max(1),
        "cellW": lbs.map(|s| s.FCLbsWidth).unwrap_or(100.0),
        "cellH": lbs.map(|s| s.FCLbsHeight).unwrap_or(38.0),
        "magTop": lbs.and_then(|s| s.FCLbsMagTop).unwrap_or(3.0),
        "magLeft": lbs.and_then(|s| s.FCLbsMagLeft).unwrap_or(3.0),
        "gapH": lbs.and_then(|s| s.FCLbsGapH).unwrap_or(2.0),
        "gapV": lbs.and_then(|s| s.FCLbsGapV).unwrap_or(2.0),
        "offsetX": lbs.and_then(|s| s.FCLbsOffsetX).unwrap_or(0.0),
        "offsetY": lbs.and_then(|s| s.FCLbsOffsetY).unwrap_or(0.0),
        "elements": elements,
        "artwork": artwork,
        "preprinted": elements.iter().any(|el|el.preprinted_paper==Some(true)),
        "items": page_items
    });
    let payload_path = tmp_dir.join("page.json");
    std::fs::write(&payload_path, serde_json::to_string(&payload).unwrap_or_default())
        .map_err(|e| e.to_string())?;

    let ps_script = r#"
$ErrorActionPreference = 'Stop'
$cfg = Get-Content -Raw -Encoding UTF8 $args[0] | ConvertFrom-Json
Add-Type -AssemblyName System.Drawing
$doc = New-Object System.Drawing.Printing.PrintDocument
$doc.PrinterSettings.PrinterName = $cfg.printerName
if (-not $doc.PrinterSettings.IsValid) { throw ('Printer invalid: ' + $cfg.printerName) }
$doc.DocumentName = $cfg.docName
$doc.PrintController = New-Object System.Drawing.Printing.StandardPrintController
$doc.OriginAtMargins = $false

$lowerName = $cfg.printerName.ToLower()
$isVirtual = ($lowerName -like '*print to pdf*' -or $lowerName -like '*xps*' -or $lowerName -like '*adobe pdf*')
if ($isVirtual -and $cfg.pdfTarget) {
  $doc.PrinterSettings.PrintToFile = $true
  $doc.PrinterSettings.PrintFileName = [string]$cfg.pdfTarget
}

$targetPaper = $null
$wantPaper = $(if ($cfg.paperSize) { [string]$cfg.paperSize } else { 'A4' })
$pwMm = $(if ($cfg.paperWidthMm) { [double]$cfg.paperWidthMm } else { 210.0 })
$phMm = $(if ($cfg.paperHeightMm) { [double]$cfg.paperHeightMm } else { 297.0 })
if ($wantPaper -eq 'B5_JIS' -or $wantPaper -eq 'B5') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.PaperName -match '\bB5\b' -and $_.PaperName -match 'JIS' } | Select-Object -First 1
} elseif ($wantPaper -eq 'B5_ISO') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.PaperName -match 'ISO.*B5|B5.*ISO' } | Select-Object -First 1
} elseif ($wantPaper -eq 'LETTER') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.Kind -eq [System.Drawing.Printing.PaperKind]::Letter } | Select-Object -First 1
} elseif ($wantPaper -eq 'A4') {
  $targetPaper = $doc.PrinterSettings.PaperSizes | Where-Object { $_.Kind -eq [System.Drawing.Printing.PaperKind]::A4 -or $_.PaperName -match '^\s*A4\b' } | Select-Object -First 1
}
if (-not $targetPaper) {
  $targetPaper = New-Object System.Drawing.Printing.PaperSize($wantPaper, [int][Math]::Round($pwMm / 0.254), [int][Math]::Round($phMm / 0.254))
}
if ($targetPaper) {
  $doc.PrinterSettings.DefaultPageSettings.PaperSize = $targetPaper
  $doc.DefaultPageSettings.PaperSize = $targetPaper
}

$matchedTray = $null
if ($cfg.trayName -and $cfg.trayName.Trim() -ne '') {
  $wantTray = $cfg.trayName.Trim()
  $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -ieq $wantTray } | Select-Object -First 1
  if (-not $matchedTray) {
    $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -like ($wantTray + '*') } | Select-Object -First 1
  }
  if ($matchedTray) {
    $doc.PrinterSettings.DefaultPageSettings.PaperSource = $matchedTray
    $doc.DefaultPageSettings.PaperSource = $matchedTray
  }
}

$doc.add_QueryPageSettings({
  param($sender, $e)
  if ($targetPaper) { $e.PageSettings.PaperSize = $targetPaper }
  if ($matchedTray) { $e.PageSettings.PaperSource = $matchedTray }
})

function Get-DigStr($obj, [string]$path) {
  if (-not $path) { return '' }
  $cur = $obj
  foreach ($seg in $path.Split('.')) {
    if ($null -eq $cur) { return '' }
    $prop = $cur.PSObject.Properties[$seg]
    if ($null -eq $prop) { return '' }
    $cur = $prop.Value
  }
  if ($null -eq $cur) { return '' }
  return [string]$cur
}

function Test-When($obj, $whenObj) {
  if ($null -eq $whenObj) { return $true }
  $p = [string]$whenObj.path
  $cur = $obj
  foreach ($seg in $p.Split('.')) {
    if ($null -eq $cur) { return $false }
    $prop = $cur.PSObject.Properties[$seg]
    if ($null -eq $prop) { return $false }
    $cur = $prop.Value
  }
  $b = [bool]$cur
  return ($b -eq [bool]$whenObj.eq)
}

function Resolve-Brush($hex, $defaultBrush) {
  if ($hex -and $hex -match '^#[0-9A-Fa-f]{6}$') {
    $c = [System.Drawing.ColorTranslator]::FromHtml($hex)
    return New-Object System.Drawing.SolidBrush($c)
  }
  return $defaultBrush
}

$mmToDisplay = 1.0 / 0.254
$script:itemOffset = 0
$perPage = [Math]::Max(1, [int]$cfg.cols * [int]$cfg.rows)
$doc.add_PrintPage({
  param($sender, $e)
  $g = $e.Graphics
  $g.PageUnit = [System.Drawing.GraphicsUnit]::Display
  $hmX = [double]$e.PageSettings.HardMarginX
  $hmY = [double]$e.PageSettings.HardMarginY
  $cols = [int]$cfg.cols
  $allItems = @($cfg.items)
  $endIdx = [Math]::Min($allItems.Count, $script:itemOffset + $perPage)
  $els = @($cfg.elements)
  $offX = [double]$cfg.offsetX
  $offY = [double]$cfg.offsetY

  for ($idx = $script:itemOffset; $idx -lt $endIdx; $idx++) {
    $i = $idx - $script:itemOffset
    $it = $allItems[$idx]
    $c = $i % $cols
    $r = [Math]::Floor($i / $cols)
    $cellX = ($offX + [double]$cfg.magLeft + $c * ([double]$cfg.cellW + [double]$cfg.gapH)) * $mmToDisplay - $hmX
    $cellY = ($offY + [double]$cfg.magTop + $r * ([double]$cfg.cellH + [double]$cfg.gapV)) * $mmToDisplay - $hmY
    $cellWpx = [float]([double]$cfg.cellW * $mmToDisplay)
    $cellHpx = [float]([double]$cfg.cellH * $mmToDisplay)

    if ($els.Count -gt 0) {
      for ($ei=0; $ei -lt $els.Count; $ei++) {
        $el=$els[$ei]
        if (-not (Test-When $it $el.when)) { continue }
        $ex = [double]($(if ($null -ne $el.x) { $el.x } else { 0 })) * $mmToDisplay + $cellX
        $ey = [double]($(if ($null -ne $el.y) { $el.y } else { 0 })) * $mmToDisplay + $cellY
        $ew = [double]($(if ($null -ne $el.width) { $el.width } else { $cfg.cellW })) * $mmToDisplay
        $eh = [double]($(if ($null -ne $el.height) { $el.height } else { 6 })) * $mmToDisplay
        $tp = [string]$el.type

        if ($tp -eq 'qrcode') {
          $entry=$cfg.artwork[$idx].PSObject.Properties[[string]$ei]
          if (!$entry) { continue }
          $matrix=@($entry.Value)
          $side=[Math]::Min($ew,$eh)
          $step=$side/($matrix.Count+8)
          $left=$ex+($ew-$side)/2
          $top=$ey+($eh-$side)/2
          $g.FillRectangle([Drawing.Brushes]::White,[single]$left,[single]$top,[single]$side,[single]$side)
          for ($y=0;$y -lt $matrix.Count;$y++) {
            for ($x=0;$x -lt $matrix.Count;$x++) {
              if ($matrix[$y][$x] -eq '1') {
                $g.FillRectangle([Drawing.Brushes]::Black,[single]($left+($x+4)*$step),[single]($top+($y+4)*$step),[single]$step,[single]$step)
              }
            }
          }
        } elseif ($tp -eq 'image') {
          $entry=$cfg.artwork[$idx].PSObject.Properties[[string]$ei]
          if (!$entry) { continue }
          $stream=[IO.MemoryStream]::new([Convert]::FromBase64String($entry.Value))
          try {
            $img=[Drawing.Image]::FromStream($stream)
            try {
              $ratio=[Math]::Min($ew/$img.Width,$eh/$img.Height)
              $w=$ratio*$img.Width
              $h=$ratio*$img.Height
              $g.DrawImage($img,[single]($ex+($ew-$w)/2),[single]($ey+($eh-$h)/2),[single]$w,[single]$h)
            } finally {$img.Dispose()}
          } finally {$stream.Dispose()}
        } elseif ($tp -eq 'icon' -and $el.iconName -eq 'heart') {
          $path=[Drawing.Drawing2D.GraphicsPath]::new()
          $path.AddBezier(12,21,10,19,2,13,2,8)
          $path.AddBezier(2,8,2,2,8,1,12,5)
          $path.AddBezier(12,5,16,1,22,2,22,8)
          $path.AddBezier(22,8,22,13,14,19,12,21)
          $side=[Math]::Min($ew,$eh)
          $matrix=[Drawing.Drawing2D.Matrix]::new([single]($side/24),0,0,[single]($side/24),[single]($ex+($ew-$side)/2),[single]($ey+($eh-$side)/2))
          $path.Transform($matrix)
          $pen=[Drawing.Pen]::new([Drawing.ColorTranslator]::FromHtml($el.color),[single]($side/12))
          try {$g.DrawPath($pen,$path)} finally {$pen.Dispose();$matrix.Dispose();$path.Dispose()}
        } elseif ($tp -eq 'rect') {
          if ($el.fill -and $el.fill -ne '#FFFFFF') {
            $br = Resolve-Brush $el.fill ([System.Drawing.Brushes]::LightYellow)
            $g.FillRectangle($br, [float]$ex, [float]$ey, [float]$ew, [float]$eh)
            if ($el.fill -match '^#[0-9A-Fa-f]{6}$') { $br.Dispose() }
          }
          $g.DrawRectangle([System.Drawing.Pens]::LightGray, [float]$ex, [float]$ey, [float]$ew, [float]$eh)
        } elseif ($tp -eq 'line') {
          $lineColor = [System.Drawing.Color]::Black
          if ($el.color) { try { $lineColor = [System.Drawing.ColorTranslator]::FromHtml([string]$el.color) } catch {} }
          $lineWidth = [float]($(if ($null -ne $el.lineWidth) { [double]$el.lineWidth } else { 0.2 })) * $mmToDisplay
          $pen = New-Object System.Drawing.Pen($lineColor, [Math]::Max(0.1, $lineWidth))
          $g.DrawLine($pen, [float]$ex, [float]$ey, [float]($ex + $ew), [float]($ey + $eh))
          $pen.Dispose()
        } elseif ($tp -eq 'barcode') {
          $code = $cfg.artwork[$idx].PSObject.Properties[[string]$ei].Value
          $bits = [string]$code.bits
          $barVal = [string]$code.value
          if ($bits.Length -gt 0) {
            $barH = if ($el.showText -ne $false) { $eh - 12.0 } else { $eh }
            $dot = 100.0 / $g.DpiX
            $moduleDots = [Math]::Floor($ew / ($dot * ($bits.Length + 22)))
            if ($moduleDots -lt 1 -or $barH -le 0) { throw "Barcode field too small: $barVal" }
            $modW = $moduleDots * $dot
            $start = [Math]::Round(($ex + ($ew - $bits.Length * $modW) / 2) / $dot) * $dot
            $g.FillRectangle([System.Drawing.Brushes]::White,[float]$ex,[float]$ey,[float]$ew,[float]$barH)
            foreach ($run in [regex]::Matches($bits,'1+')) {
              $g.FillRectangle([System.Drawing.Brushes]::Black,[float]($start + $run.Index * $modW),[float]$ey,[float]($run.Length * $modW),[float]$barH)
            }
            if ($barVal -and ($el.showText -ne $false)) {
              $fb = New-Object System.Drawing.Font('Consolas', 7)
              $sf = New-Object System.Drawing.StringFormat
              $sf.Alignment = [System.Drawing.StringAlignment]::Center
              $g.DrawString($barVal, $fb, [System.Drawing.Brushes]::Black, [System.Drawing.RectangleF]::new([float]$ex, [float]($ey + $barH + 1.0), [float]$ew, 12.0), $sf)
              $sf.Dispose(); $fb.Dispose()
            }
          }
        } elseif ($tp -eq 'price' -or $tp -eq 'text' -or $tp -eq 'static' -or $tp -eq 'date' -or $tp -eq 'time') {
          $txt = ''
          if ($tp -eq 'static') { $txt = [string]$el.text }
          elseif ($tp -eq 'date') { $txt = [string](Get-DigStr $it 'tag.print_date'); if (-not $txt) { $txt = (Get-Date -Format 'dd/MM/yyyy') } }
          elseif ($tp -eq 'time') { $txt = (Get-Date -Format 'HH:mm') }
          else {
            $raw = [string](Get-DigStr $it ([string]$el.field))
            if ($raw) {
              $pre = $(if ($el.prefix) { [string]$el.prefix } else { '' })
              $suf = $(if ($el.suffix) { [string]$el.suffix } else { '' })
              $txt = $pre + $raw + $suf
            }
          }
          if ($txt) {
            $pt = [float][Math]::Max(5.5, $(if ($el.fontSize) { [double]$el.fontSize } else { 8.0 }))
            $style = [System.Drawing.FontStyle]::Regular
            if ($tp -eq 'price' -or [string]$el.field -eq 'product.name' -or [string]$el.color -eq '#CC0000') {
              $style = [System.Drawing.FontStyle]::Bold
            }
            if ($el.strike -eq $true) {
              $style = $style -bor [System.Drawing.FontStyle]::Strikeout
            }
            $f = New-Object System.Drawing.Font('Tahoma', $pt, $style)
            $br = Resolve-Brush ([string]$el.color) ([System.Drawing.Brushes]::Black)
            $sf = New-Object System.Drawing.StringFormat
            if ($el.align -eq 'center') { $sf.Alignment = [System.Drawing.StringAlignment]::Center }
            elseif ($el.align -eq 'right') { $sf.Alignment = [System.Drawing.StringAlignment]::Far }
            $g.DrawString($txt, $f, $br, [System.Drawing.RectangleF]::new([float]$ex, [float]$ey, [float]$ew, [float][Math]::Max($eh, $pt * 2.2)), $sf)
            $sf.Dispose(); $f.Dispose()
            if ($el.color -and $el.color -match '^#[0-9A-Fa-f]{6}$') { $br.Dispose() }
          }
        }
      }
    }
    if (!$cfg.preprinted) { $g.DrawRectangle([System.Drawing.Pens]::Gainsboro, [float]$cellX, [float]$cellY, $cellWpx, $cellHpx) }
  }
  $script:itemOffset = $endIdx
  $e.HasMorePages = ($script:itemOffset -lt $allItems.Count)
})
$doc.Print()
"#;
    let ps_path = tmp_dir.join("print_page.ps1");
    let mut ps_bytes = vec![0xEFu8, 0xBB, 0xBF];
    ps_bytes.extend_from_slice(ps_script.as_bytes());
    std::fs::write(&ps_path, &ps_bytes).map_err(|e| e.to_string())?;

    let mut cmd = tokio::process::Command::new("powershell");
    cmd.creation_flags(0x08000000);
    let out = cmd
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            ps_path.to_str().unwrap_or_default(),
            payload_path.to_str().unwrap_or_default(),
        ])
        .output()
        .await;
    let _ = std::fs::remove_dir_all(&tmp_dir);

    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Metadata used to generate the Morning Cover Sheet (1 summary sheet on open-store printer).
#[derive(Debug, Clone)]
pub struct CoverSheetInfo {
    pub run_id: String,
    pub job_id: Option<String>,
    pub biz_date: String,
    pub trigger_by: String,
    pub store_name: String,
    pub lbs_code: String,
    pub nor_qty: i64,
    pub pmt_qty: i64,
    pub skipped_dup: usize,
    pub fail_qty: i64,
    pub summary_lines: Vec<String>,
}

/// Print 1 Cover Sheet summary page when `FTLscStaCoverSheet == '1'` (`FR-05` / `TC-07`).
pub async fn print_cover_sheet(pool: &SqlitePool, info: &CoverSheetInfo) -> Result<(), String> {
    let route = resolve_printer_route(pool, &info.lbs_code, "1").await;
    if route.printer_name.trim().is_empty() {
        return Ok(());
    }
    if let Some(ref ip) = route.lan_ip {
        // Thermal LAN printer: skip A4 cover sheet so roll stock isn't wasted, or send brief 1-tag receipt
        let header_line = format!(
            "^XA^CI28^PW600^LL300^FO20,20^A0N,28,24^FDAdaBarprint AUTO SUMMARY^FS^FO20,65^A0N,22,20^FDDate: {} | By: {}^FS^FO20,105^A0N,24,22^FDNor: {} | Pmt: {} | Fail: {}^FS^FO20,145^A0N,20,18^FDRun: {}^FS^XZ",
            info.biz_date,
            info.trigger_by,
            info.nor_qty,
            info.pmt_qty,
            info.fail_qty,
            &info.run_id[..8.min(info.run_id.len())]
        );
        let _ = tcp_send(ip, route.lan_port, &header_line).await;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        if check_printer_ready(&route).await.is_err() {
            return Ok(());
        }
        let job_uid = Uuid::new_v4().to_string();
        let tmp_dir = std::env::temp_dir().join(format!("ada_cover_{}", job_uid));
        let _ = std::fs::create_dir_all(&tmp_dir);

        let export_dir = crate::db::resolve_exports_dir();
        let _ = std::fs::create_dir_all(&export_dir);
        let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let clean_export = std::fs::canonicalize(&export_dir)
            .unwrap_or(export_dir)
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_string();
        let pdf_out = std::path::PathBuf::from(clean_export)
            .join(format!("AdaCoverSheet_{}_{}.pdf", ts, &job_uid[..6]));

        let payload = json!({
            "printerName": route.printer_name,
            "trayName": route.tray_name,
            "pdfTarget": pdf_out.to_string_lossy(),
            "storeName": info.store_name,
            "bizDate": info.biz_date,
            "printedAt": Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
            "runId": info.run_id,
            "jobId": info.job_id.as_deref().unwrap_or("-"),
            "triggerBy": info.trigger_by,
            "norQty": info.nor_qty,
            "pmtQty": info.pmt_qty,
            "skippedDup": info.skipped_dup,
            "failQty": info.fail_qty,
            "lines": info.summary_lines,
        });
        let payload_path = tmp_dir.join("cover.json");
        std::fs::write(&payload_path, serde_json::to_string(&payload).unwrap_or_default())
            .map_err(|e| e.to_string())?;

        let ps_script = r#"
$ErrorActionPreference = 'Stop'
$cfg = Get-Content -Raw -Encoding UTF8 $args[0] | ConvertFrom-Json
Add-Type -AssemblyName System.Drawing
$doc = New-Object System.Drawing.Printing.PrintDocument
$doc.PrinterSettings.PrinterName = $cfg.printerName
if (-not $doc.PrinterSettings.IsValid) { exit 0 }
$doc.DocumentName = "AdaBarprint_CoverSheet_" + $cfg.bizDate
$doc.PrintController = New-Object System.Drawing.Printing.StandardPrintController

$lowerName = $cfg.printerName.ToLower()
if (($lowerName -like '*print to pdf*' -or $lowerName -like '*xps*' -or $lowerName -like '*adobe pdf*') -and $cfg.pdfTarget) {
  $doc.PrinterSettings.PrintToFile = $true
  $doc.PrinterSettings.PrintFileName = [string]$cfg.pdfTarget
}

if ($cfg.trayName -and $cfg.trayName.Trim() -ne '') {
  $m = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -ieq $cfg.trayName.Trim() } | Select-Object -First 1
  if ($m) { $doc.DefaultPageSettings.PaperSource = $m }
}

$doc.add_PrintPage({
  param($sender, $e)
  $g = $e.Graphics
  $fTitle = New-Object System.Drawing.Font('Tahoma', 16, [System.Drawing.FontStyle]::Bold)
  $fSub   = New-Object System.Drawing.Font('Tahoma', 11, [System.Drawing.FontStyle]::Bold)
  $fBody  = New-Object System.Drawing.Font('Tahoma', 10)
  $fMono  = New-Object System.Drawing.Font('Consolas', 9.5)
  $b = [System.Drawing.Brushes]::Black

  $x = 45.0; $y = 45.0
  $g.DrawRectangle([System.Drawing.Pens]::DarkSlateGray, $x, $y, 700, 120)
  $g.DrawString("AdaBarprint - ใบปะหน้าสรุปงานพิมพ์ป้ายอัตโนมัติ (Cover Sheet)", $fTitle, $b, ($x + 16), ($y + 16))
  $g.DrawString(("สาขา / ร้าน: " + $cfg.storeName + "  |  วันที่ธุรกิจ: " + $cfg.bizDate), $fSub, $b, ($x + 16), ($y + 52))
  $g.DrawString(("เวลาพิมพ์: " + $cfg.printedAt + "  |  สั่งงานโดย: " + $cfg.triggerBy + "  |  RunID: " + $cfg.runId), $fBody, $b, ($x + 16), ($y + 82))

  $y = 185.0
  $g.FillRectangle([System.Drawing.Brushes]::WhiteSmoke, $x, $y, 700, 65)
  $g.DrawRectangle([System.Drawing.Pens]::Gray, $x, $y, 700, 65)
  $summary = ("ป้ายปกติ: " + $cfg.norQty + " ดวง    |    ป้ายโปรโมชัน: " + $cfg.pmtQty + " ดวง    |    ข้ามรายการเดิม: " + $cfg.skippedDup + "    |    ค้างพิมพ์ (FAILED): " + $cfg.failQty)
  $g.DrawString($summary, $fSub, $b, ($x + 16), ($y + 22))

  $y = 272.0
  $g.DrawString("ตัวอย่างรายการในรอบนี้ (สูงสุด 22 รายการแรก):", $fSub, $b, $x, $y)
  $y += 26.0
  foreach ($ln in @($cfg.lines | Select-Object -First 22)) {
    $g.DrawString([string]$ln, $fMono, $b, $x, $y)
    $y += 22.0
  }
  $fTitle.Dispose(); $fSub.Dispose(); $fBody.Dispose(); $fMono.Dispose()
  $e.HasMorePages = $false
})
$doc.Print()
"#;
        let ps_path = tmp_dir.join("cover.ps1");
        let mut ps_bytes = vec![0xEFu8, 0xBB, 0xBF];
        ps_bytes.extend_from_slice(ps_script.as_bytes());
        let _ = std::fs::write(&ps_path, &ps_bytes);

        let mut cmd = tokio::process::Command::new("powershell");
        cmd.creation_flags(0x08000000);
        let out = cmd
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-STA",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                ps_path.to_str().unwrap_or_default(),
                payload_path.to_str().unwrap_or_default(),
            ])
            .output()
            .await;
        let _ = std::fs::remove_dir_all(&tmp_dir);
        if let Ok(o) = out {
            if !o.status.success() {
                return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
            }
        }
    }
    Ok(())
}

pub type ChunkSenderFn<'a> = Box<
    dyn FnMut(&ResolvedPrinterRoute, i64, Vec<&TCNTLabelSchedQueue>, Option<&TCNMLabelSize>, &[TemplateElement], Vec<Value>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + 'a,
>;

/// Real hardware/driver chunk sender (TCP per item or Windows headless page chunk).
pub fn default_chunk_sender(biz_date: String, run_id: String) -> ChunkSenderFn<'static> {
    Box::new(move |route, page_no, chunk_rows, lbs, elements, ctxs| {
        let route = route.clone();
        let lbs_cloned = lbs.cloned();
        let elements = elements.to_vec();
        let qtys: Vec<u32> = chunk_rows.iter().map(|r| r.FNLsqQty.max(1) as u32).collect();
        let doc_name = format!("AdaBarprint_Auto_{}_{}_P{}", biz_date, &run_id[..6.min(run_id.len())], page_no);
        Box::pin(async move {
            if let Some(ref ip) = route.lan_ip {
                let brand = PrinterBrand::from_str(&route.protocol).unwrap_or(PrinterBrand::Zebra);
                let opts = DriverOptions {
                    dpi: route.dpi,
                    label_width_mm: lbs_cloned.as_ref().map(|s| s.FCLbsWidth).unwrap_or(100.0),
                    label_height_mm: lbs_cloned.as_ref().map(|s| s.FCLbsHeight).unwrap_or(38.0),
                    gap_mm: lbs_cloned.as_ref().and_then(|s| s.FCLbsGapV).unwrap_or(2.0).max(0.0),
                    ..Default::default()
                };
                let pairs: Vec<(Value, u32)> = ctxs
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| (c, qtys.get(i).copied().unwrap_or(1)))
                    .collect();
                let cmd_str = generate_commands(brand, &pairs, &elements, &opts)?;
                let (ok, msg, _) = tcp_send(ip, route.lan_port, &cmd_str).await;
                if ok {
                    Ok(())
                } else {
                    Err(msg)
                }
            } else {
                #[cfg(target_os = "windows")]
                {
                    if let Err((_c, e)) = check_printer_ready(&route).await {
                        return Err(e);
                    }
                    let mut expanded = Vec::new();
                    for (i, c) in ctxs.into_iter().enumerate() {
                        let copies = qtys.get(i).copied().unwrap_or(1);
                        for _ in 0..copies {
                            expanded.push(c.clone());
                        }
                    }
                    send_windows_headless_page(&route, lbs_cloned.as_ref(), &elements, &expanded, &doc_name).await
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = (doc_name, qtys);
                    Ok(())
                }
            }
        })
    })
}

/// Dispatch all `READY` (or `FAILED` when reprinting) items of a run in safe small chunks,
/// updating `TCNTLabelSchedQueue`, `TCNTLabelSchedWatermark`, `TCNTLabelJob`, `TCNTLabelJobDt`, `TCNTLabelLog`.
pub async fn dispatch_run_queue<'a>(
    pool: &SqlitePool,
    run_id: &str,
    biz_date: &str,
    reprint_only: bool,
    mut sender: ChunkSenderFn<'a>,
) -> Result<PrintDispatchSummary, sqlx::Error> {
    let target_status = if reprint_only { "FAILED" } else { "READY" };
    let queue_rows: Vec<TCNTLabelSchedQueue> = sqlx::query_as(
        r#"
        SELECT FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc, FTLsqPriType,
               FTLbsCode, FCLsqOldPrice, FCLsqSalePrice, FTLsqPromoDocNo, FTLsqPriceHash,
               FNLsqQty, FNLsqBatchPage, FTLsqStatus, FTLsqErrMsg
        FROM TCNTLabelSchedQueue
        WHERE FTLsrRunID = ? AND FTLsqStatus = ?
        ORDER BY FTLsqPriType ASC, FTLbsCode ASC, FNLsqSeq ASC
        "#,
    )
    .bind(run_id)
    .bind(target_status)
    .fetch_all(pool)
    .await?;

    if queue_rows.is_empty() {
        return Ok(PrintDispatchSummary {
            job_id: None,
            total_ready: 0,
            printed_normal: 0,
            printed_promo: 0,
            failed_count: 0,
            run_status: "SKIPPED_EMPTY".to_string(),
            err_code: None,
            err_msg: None,
        });
    }

    let branch = crate::storage::configured_branch(pool).await;
    let store_name: String = sqlx::query_scalar(
        "SELECT COALESCE(FTBchName,'AdaSoft Store') FROM TCNMBranch_L WHERE FNLngID=1 AND (?='' OR FTBchCode=?) ORDER BY FTBchCode LIMIT 1",
    ).bind(&branch).bind(&branch).fetch_optional(pool).await.ok().flatten().unwrap_or_else(|| "AdaSoft Store".into());
    let url_template = crate::routes::product_url_template(pool).await;

    let job_id = gen_guid();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query(
        "INSERT INTO TCNTLabelJob (FTLjbJobID, FTLjbBchCode, FTLjbSource, FTLjbPriType, FDLjbFrom, FTLjbStaVfy, FTLjbStatus, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, '', '2', '1', ?, '1', '4', ?, 'AUTO_0400', ?, 'AUTO_0400')",
    )
    .bind(&job_id).bind(&now).bind(&now).bind(&now)
    .execute(pool)
    .await;

    // Group by (pri_type, lbs_code) so tray 1 Normal finishes cleanly before tray 2 Promo
    let mut groups: BTreeMap<(String, String), Vec<TCNTLabelSchedQueue>> = BTreeMap::new();
    for q in queue_rows {
        groups
            .entry((q.FTLsqPriType.clone(), q.FTLbsCode.clone()))
            .or_default()
            .push(q);
    }

    // Lookup names once
    let mut name_cache: HashMap<String, String> = HashMap::new();
    let mut printed_nor = 0i64;
    let mut printed_pmt = 0i64;
    let mut failed_cnt = 0i64;
    let mut last_err_code: Option<String> = None;
    let mut last_err_msg: Option<String> = None;
    let mut job_dt_seq = 0i64;

    for ((pri_type, lbs_code), rows) in groups {
        let route = resolve_printer_route(pool, &lbs_code, &pri_type).await;
        let lbs_meta = sqlx::query_as::<_, TCNMLabelSize>(
            "SELECT FTLbsCode, FTLbsGroup, FTLbsName, FCLbsWidth, FCLbsHeight, FNLbsCol, FNLbsRow, FCLbsMagTop, FCLbsMagLeft, FCLbsMagBottom, FCLbsMagRight, FCLbsGapH, FCLbsGapV, FNLbsQtyPageNor, FNLbsQtyPagePmt, FTLbsTmplNor, FTLbsTmplPmt, FTLbsHhtCode, FTLbsStaUse, FTLbsPaperSize, FCLbsPaperWidth, FCLbsPaperHeight, FCLbsOffsetX, FCLbsOffsetY, FTPprCode, FTLbsStdTmplNor, FTLbsStdTmplPmt FROM TCNMLabelSize WHERE FTLbsCode = ?",
        )
        .bind(&lbs_code)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        let (tmpl_code, elements) = load_template_elements(pool, lbs_meta.as_ref(), &lbs_code, &pri_type).await;
        let chunk_size: usize = if route.lan_ip.is_some() {
            1 // Thermal LAN: 1 label per ACK for exact partial recovery
        } else {
            let cols = lbs_meta.as_ref().map(|s| s.FNLbsCol).unwrap_or(2).max(1);
            let rows_cnt = lbs_meta.as_ref().map(|s| s.FNLbsRow).unwrap_or(7).max(1);
            (cols * rows_cnt).max(1) as usize
        };

        let mut aborted_group_err: Option<String> = None;
        let mut group_printed = 0i64;
        let mut group_pages_ok = 0i64;

        for (page_idx, chunk) in rows.chunks(chunk_size).enumerate() {
            let page_no = (page_idx + 1) as i64;
            if let Some(ref err) = aborted_group_err {
                // Remaining chunks after printer lost -> immediately mark FAILED
                for r in chunk {
                    failed_cnt += 1;
                    let _ = sqlx::query(
                        "UPDATE TCNTLabelSchedQueue SET FNLsqBatchPage = ?, FTLsqStatus = 'FAILED', FTLsqErrMsg = ? WHERE FTLsrRunID = ? AND FNLsqSeq = ?",
                    )
                    .bind(page_no).bind(err).bind(run_id).bind(r.FNLsqSeq)
                    .execute(pool)
                    .await;
                }
                continue;
            }

            let mut ctxs = Vec::with_capacity(chunk.len());
            let mut refs = Vec::with_capacity(chunk.len());
            for r in chunk {
                let pdt_name = if let Some(n) = name_cache.get(&r.FTPdtCode) {
                    n.clone()
                } else {
                    let fetched = sqlx::query_scalar::<_, Option<String>>(
                        "SELECT FTPdtName FROM TCNMPdt_L WHERE FTPdtCode = ? AND FNLngID = 1 LIMIT 1",
                    )
                    .bind(&r.FTPdtCode)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten()
                    .flatten()
                    .unwrap_or_else(|| r.FTPdtCode.clone());
                    name_cache.insert(r.FTPdtCode.clone(), fetched.clone());
                    fetched
                };
                let mut ctx = build_label_context(pool, r, &pdt_name, &store_name, biz_date).await;
                ctx["product"]["url"] = json!(url_template.replace("{code}", &r.FTPdtCode).replace("{bch}", &branch));
                ctxs.push(ctx);
                refs.push(r);
            }

            match sender(&route, page_no, refs, lbs_meta.as_ref(), &elements, ctxs).await {
                Ok(()) => {
                    group_pages_ok += 1;
                    for r in chunk {
                        group_printed += 1;
                        if pri_type == "2" {
                            printed_pmt += 1;
                        } else {
                            printed_nor += 1;
                        }
                        let _ = sqlx::query(
                            "UPDATE TCNTLabelSchedQueue SET FNLsqBatchPage = ?, FTLsqStatus = 'PRINTED', FTLsqErrMsg = NULL WHERE FTLsrRunID = ? AND FNLsqSeq = ?",
                        )
                        .bind(page_no).bind(run_id).bind(r.FNLsqSeq)
                        .execute(pool)
                        .await;

                        // Update Watermark ONLY after physical print chunk succeeded!
                        let _ = sqlx::query(
                            r#"
                            INSERT INTO TCNTLabelSchedWatermark (FTBarCode, FTLbsCode, FTLswLastHash, FTLswLastPriType, FCLswLastPrice, FDLswPrintedAt, FTLsrRunID)
                            VALUES (?, ?, ?, ?, ?, ?, ?)
                            ON CONFLICT(FTBarCode, FTLbsCode) DO UPDATE SET
                                FTLswLastHash = excluded.FTLswLastHash,
                                FTLswLastPriType = excluded.FTLswLastPriType,
                                FCLswLastPrice = excluded.FCLswLastPrice,
                                FDLswPrintedAt = excluded.FDLswPrintedAt,
                                FTLsrRunID = excluded.FTLsrRunID
                            "#,
                        )
                        .bind(&r.FTBarCode).bind(&r.FTLbsCode).bind(&r.FTLsqPriceHash).bind(&r.FTLsqPriType)
                        .bind(r.FCLsqSalePrice).bind(&now).bind(run_id)
                        .execute(pool)
                        .await;

                        job_dt_seq += 1;
                        let pdt_name = name_cache.get(&r.FTPdtCode).map(|s| s.as_str()).unwrap_or("");
                        let _ = sqlx::query(
                            "INSERT INTO TCNTLabelJobDt (FTLjbJobID, FNLjdSeq, FTLjdBarCode, FTLjdPdtCode, FTLjdName, FCLjdSetPrice, FCLjdSalePrice, FTLjdLabSize, FNLjdLabQty, FNLjdQtyOrg, FNLjdPage, FTLjdPriTyp, FTLjdTmpl, FTLjdStaSel, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '1', ?, 'AUTO_0400', ?, 'AUTO_0400')",
                        )
                        .bind(&job_id).bind(job_dt_seq).bind(&r.FTBarCode).bind(&r.FTPdtCode).bind(pdt_name)
                        .bind(r.FCLsqOldPrice.unwrap_or(r.FCLsqSalePrice)).bind(r.FCLsqSalePrice)
                        .bind(&r.FTLbsCode).bind(r.FNLsqQty).bind(r.FNLsqQty).bind(page_no)
                        .bind(&r.FTLsqPriType).bind(&tmpl_code).bind(&now).bind(&now)
                        .execute(pool)
                        .await;
                    }
                }
                Err(e) => {
                    if route.lan_ip.is_none() {
                        cancel_stale_spooler_jobs(&route.printer_name).await;
                    }
                    let code = if route.lan_ip.is_some() { "E-PRT-001" } else { "E-PRT-002" };
                    last_err_code = Some(code.to_string());
                    last_err_msg = Some(e.clone());
                    aborted_group_err = Some(e.clone());
                    for r in chunk {
                        failed_cnt += 1;
                        let _ = sqlx::query(
                            "UPDATE TCNTLabelSchedQueue SET FNLsqBatchPage = ?, FTLsqStatus = 'FAILED', FTLsqErrMsg = ? WHERE FTLsrRunID = ? AND FNLsqSeq = ?",
                        )
                        .bind(page_no).bind(&e).bind(run_id).bind(r.FNLsqSeq)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }

        // Write TCNTLabelLog entry for this tray/group
        let log_id = gen_guid();
        let _ = sqlx::query(
            "INSERT INTO TCNTLabelLog (FTLlgLogID, FTLjbJobID, FTLlgLbsCode, FTLlgPriTyp, FTLlgPsvCode, FTLlgPrinter, FNLlgQtyLabel, FNLlgQtyPage, FDLlgPriOn, FTLlgPriBy, FTLlgStatus, FTLlgErrMsg, FDCreateOn, FTCreateBy, FDLastUpdOn, FTLastUpdBy) VALUES (?, ?, ?, ?, 'LOCAL', ?, ?, ?, ?, 'AUTO_0400', ?, ?, ?, 'AUTO_0400', ?, 'AUTO_0400')",
        )
        .bind(&log_id).bind(&job_id).bind(&lbs_code).bind(&pri_type).bind(&route.printer_name)
        .bind(group_printed).bind(group_pages_ok).bind(&now)
        .bind(if aborted_group_err.is_some() { "2" } else { "4" })
        .bind(aborted_group_err.as_deref().unwrap_or(""))
        .bind(&now).bind(&now)
        .execute(pool)
        .await;

        // Tray switching guard delay (900ms) if both Normal and Promo run on same printer
        tokio::time::sleep(std::time::Duration::from_millis(900)).await;
    }

    let finish_now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query("UPDATE TCNTLabelJob SET FDLjbTo = ? WHERE FTLjbJobID = ?")
        .bind(&finish_now).bind(&job_id)
        .execute(pool)
        .await;

    let total_ok = printed_nor + printed_pmt;
    let run_status = if failed_cnt == 0 {
        "SUCCESS"
    } else if total_ok > 0 {
        "PARTIAL"
    } else {
        "FAILED"
    };

    Ok(PrintDispatchSummary {
        job_id: Some(job_id),
        total_ready: (total_ok + failed_cnt) as usize,
        printed_normal: printed_nor,
        printed_promo: printed_pmt,
        failed_count: failed_cnt,
        run_status: run_status.to_string(),
        err_code: last_err_code,
        err_msg: last_err_msg,
    })
}

/// Purge old queue/run records older than `retention_days` (keeps `TCNTLabelSchedWatermark` intact!).
pub async fn cleanup_old_autoprint_data(pool: &SqlitePool, retention_days: i64) {
    let days = retention_days.max(7);
    let cutoff = (Local::now() - chrono::Duration::days(days))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let _ = sqlx::query(
        "DELETE FROM TCNTLabelSchedQueue WHERE FTLsrRunID IN (SELECT FTLsrRunID FROM TCNTLabelSchedRun WHERE FDLsrStartedAt < ? AND FTLsrStatus != 'PARTIAL')",
    )
    .bind(&cutoff)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM TCNTLabelSchedRun WHERE FDLsrStartedAt < ? AND FTLsrStatus != 'PARTIAL'",
    )
    .bind(&cutoff)
    .execute(pool)
    .await;
}

#[cfg(test)]
mod tests {
    #[cfg(target_os="windows")]
    #[tokio::test]
    async fn windows_qr_uses_each_products_field_and_a_full_module_matrix() {
        let elements = serde_json::from_value::<Vec<super::TemplateElement>>(
            serde_json::json!([{"type":"qrcode","field":"product.url"}])).unwrap();
        let input = [serde_json::json!({"product":{"url":"https://example.test/P01"}}),
                     serde_json::json!({"product":{"url":"https://example.test/P02"}})];
        let artwork = super::windows_artwork(&elements,&input).await.unwrap();
        assert_ne!(artwork[0],artwork[1]);
        let rows = artwork[0]["0"].as_array().unwrap();
        assert!(rows.len()>=21);
        assert!(rows.iter().all(|r|r.as_str().unwrap().len()==rows.len()));
        assert!(rows[0].as_str().unwrap().starts_with("1111111"));
    }

    use super::*;
    use crate::db::ensure_label_schema;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    #[tokio::test]
    async fn tc08_partial_print_5_of_20_logs_and_resumes_only_remaining_15() {
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMPdt (FTPdtCode TEXT PRIMARY KEY)",
            "CREATE TABLE IF NOT EXISTS TCNMPdt_L (FTPdtCode TEXT, FNLngID INTEGER, FTPdtName TEXT, PRIMARY KEY (FTPdtCode, FNLngID))",
            "CREATE TABLE IF NOT EXISTS TCNMBranch_L (FTBchCode TEXT, FNLngID INTEGER, FTBchName TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPrinter (FTPrnCode TEXT PRIMARY KEY, FTPrnType TEXT, FTPrnSrcType TEXT, FTSppCode TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPrinter_L (FTPrnCode TEXT, FNLngID INTEGER, FTPrnName TEXT)",
            "CREATE TABLE IF NOT EXISTS TSysPortPrn (FTSppCode TEXT, FTSppValue TEXT, FTSppRef TEXT, FTSppStaUse TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTPdtCode TEXT, FDPghDStart TEXT, FTPplCode TEXT)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        ensure_label_schema(&pool).await.unwrap();

        sqlx::query("INSERT INTO TCNMPrinter VALUES ('LAN1', 'ZEBRA', '2', 'PORT1')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TSysPortPrn VALUES ('PORT1', '192.168.1.99:9100', '{}', '1')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMLabelPrinter (FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpProtocol, FTLbpStaUse) VALUES ('ADA-100x38', 'LOCAL', '1', 'LAN1', 'ZEBRA', '1')")
            .execute(&pool).await.unwrap();

        let route = resolve_printer_route(&pool, "ADA-100x38", "1").await;
        assert_eq!(route.lan_ip.as_deref(), Some("192.168.1.99"));
        assert_eq!(route.lan_port, 9100);
        assert_eq!(route.protocol, "ZEBRA");

        let run_id = "RUN-PARTIAL-20";
        sqlx::query("INSERT INTO TCNTLabelSchedRun (FTLsrRunID, FTLscCode, FDLsrBizDate, FTLsrTriggerBy, FTLsrStatus, FDLsrStartedAt) VALUES (?, 'SCH-0400', '2026-11-19', 'TIMER', 'RUNNING', '2026-11-19 04:00:00')")
            .bind(run_id).execute(&pool).await.unwrap();

        for i in 1..=20i64 {
            let pdt = format!("P{:03}", i);
            let bar = format!("885000000{:03}", i);
            sqlx::query("INSERT OR IGNORE INTO TCNMPdt_L (FTPdtCode, FNLngID, FTPdtName) VALUES (?, 1, ?)").bind(&pdt).bind(format!("สินค้า {}", i)).execute(&pool).await.unwrap();
            sqlx::query(
                "INSERT INTO TCNTLabelSchedQueue (FTLsrRunID, FNLsqSeq, FTPdtCode, FTBarCode, FTLsqTriggerSrc, FTLsqPriType, FTLbsCode, FCLsqSalePrice, FTLsqPriceHash, FNLsqQty, FTLsqStatus) VALUES (?, ?, ?, ?, 'PRICE_CHG', '1', 'ADA-100x38', 15.0, ?, 1, 'READY')",
            )
            .bind(run_id).bind(i).bind(&pdt).bind(&bar).bind(format!("HASH-{}", i))
            .execute(&pool).await.unwrap();
        }

        // Mock sender succeeds on items 1..=5 then printer is lost on item 6!
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_ref = counter.clone();
        let mock_fail_at_6: ChunkSenderFn<'_> = Box::new(move |_r, _p, _rows, _l, _e, _c| {
            let n = counter_ref.fetch_add(1, Ordering::SeqCst) + 1;
            Box::pin(async move {
                if n <= 5 {
                    Ok(())
                } else {
                    Err("Printer Lost after 5 labels".to_string())
                }
            })
        });

        let sum1 = dispatch_run_queue(&pool, run_id, "2026-11-19", false, mock_fail_at_6).await.unwrap();
        assert_eq!(sum1.run_status, "PARTIAL");
        assert_eq!(sum1.printed_normal, 5);
        assert_eq!(sum1.failed_count, 15);

        let printed_cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ? AND FTLsqStatus = 'PRINTED'").bind(run_id).fetch_one(&pool).await.unwrap();
        let failed_cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelSchedQueue WHERE FTLsrRunID = ? AND FTLsqStatus = 'FAILED'").bind(run_id).fetch_one(&pool).await.unwrap();
        let wm_cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM TCNTLabelSchedWatermark").fetch_one(&pool).await.unwrap();
        assert_eq!(printed_cnt.0, 5);
        assert_eq!(failed_cnt.0, 15);
        assert_eq!(wm_cnt.0, 5);

        // Now reprint failed only -> MUST call sender strictly 15 times (6..20), never reprinting 1..5!
        let reprint_calls = Arc::new(AtomicUsize::new(0));
        let reprint_ref = reprint_calls.clone();
        let mock_recover: ChunkSenderFn<'_> = Box::new(move |_r, _p, _rows, _l, _e, _c| {
            reprint_ref.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        });
        let sum2 = dispatch_run_queue(&pool, run_id, "2026-11-19", true, mock_recover).await.unwrap();
        assert_eq!(sum2.run_status, "SUCCESS");
        assert_eq!(sum2.printed_normal, 15);
        assert_eq!(sum2.failed_count, 0);
        assert_eq!(reprint_calls.load(Ordering::SeqCst), 15);
    }

    #[tokio::test]
    async fn tc07_split_tray_routing_and_barcode_modules() {
        assert_eq!(encode_barcode_modules("8850001000019", "EAN13").len(), 95);
        assert_eq!(encode_barcode_modules("8850001000014", "EAN13"), encode_barcode_modules("8850001000014", "CODE128"));
        assert!(encode_barcode_modules("SKU-2026", "CODE128").len() >= 33);

        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        for ddl in [
            "CREATE TABLE IF NOT EXISTS TCNMPdt (FTPdtCode TEXT PRIMARY KEY)",
            "CREATE TABLE IF NOT EXISTS TCNMPdt_L (FTPdtCode TEXT, FNLngID INTEGER, FTPdtName TEXT, PRIMARY KEY (FTPdtCode, FNLngID))",
            "CREATE TABLE IF NOT EXISTS TCNMBranch_L (FTBchCode TEXT, FNLngID INTEGER, FTBchName TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPrinter (FTPrnCode TEXT PRIMARY KEY, FTPrnType TEXT, FTPrnSrcType TEXT, FTSppCode TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNMPrinter_L (FTPrnCode TEXT, FNLngID INTEGER, FTPrnName TEXT)",
            "CREATE TABLE IF NOT EXISTS TSysPortPrn (FTSppCode TEXT, FTSppValue TEXT, FTSppRef TEXT, FTSppStaUse TEXT)",
            "CREATE TABLE IF NOT EXISTS TCNTPdtPrice4PDT (FTAgnCode TEXT, FTPdtCode TEXT, FDPghDStart TEXT, FTPplCode TEXT)",
        ] {
            sqlx::query(ddl).execute(&pool).await.unwrap();
        }
        ensure_label_schema(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMLabelPrinter (FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse) VALUES ('ADA-100x38', 'LOCAL', '1', 'WinPrinterA4', 'Tray 1', 'WINDOWS', '1')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO TCNMLabelPrinter (FTLbsCode, FTPsvCode, FTLbpPriType, FTLbpPrinter, FTLbpTray, FTLbpProtocol, FTLbpStaUse) VALUES ('ADA-100x38', 'LOCAL', '2', 'WinPrinterA4', 'Tray 2', 'WINDOWS', '1')")
            .execute(&pool).await.unwrap();

        let r_nor = resolve_printer_route(&pool, "ADA-100x38", "1").await;
        let r_pmt = resolve_printer_route(&pool, "ADA-100x38", "2").await;
        assert_eq!(r_nor.printer_name, "WinPrinterA4");
        assert_eq!(r_nor.tray_name, "Tray 1");
        assert_eq!(r_pmt.tray_name, "Tray 2");
    }
}
