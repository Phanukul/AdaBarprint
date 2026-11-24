use crate::{models::TCNMLabelSize, Assets};
use serde_json::{json, Value};
use std::{path::{Path, PathBuf}, time::Duration};

fn browser() -> Option<PathBuf> {
    for key in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
        if let Some(base) = std::env::var_os(key) {
            for exe in ["Microsoft/Edge/Application/msedge.exe", "Google/Chrome/Application/chrome.exe"] {
                let path = Path::new(&base).join(exe);
                if path.is_file() { return Some(path); }
            }
        }
    }
    None
}

/// Rasterize with the same React label and barcode capture as manual Windows printing.
pub async fn render(size: &TCNMLabelSize, contexts: &[Value], printer: &str) -> Result<String, String> {
    let exe = browser().ok_or("ต้องติดตั้ง Microsoft Edge หรือ Chrome สำหรับแบบป้ายเดิม")?;
    let bundle = Assets::get("legacy-print.js").ok_or("ไม่พบตัวแสดงป้ายเดิม กรุณา build เว็บก่อน")?;
    let dir = std::env::temp_dir().join(format!("AdaBarprint-preview-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
    let result = async {
        let data = json!({"size":size, "contexts":contexts, "thermal": size.FTLbsPaperSize.as_deref()==Some("ROLL") && printer.to_uppercase().contains("EZ520")});
        let html = format!(r#"<!doctype html><meta charset="utf-8"><div id="page"></div><script type="application/json" id="job">{}</script><script src="render.js"></script>"#, data.to_string().replace('<', "\\u003c"));
        let page = dir.join("page.html");
        std::fs::write(&page, html).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("render.js"), bundle.data).map_err(|e| e.to_string())?;
        let mut cmd = tokio::process::Command::new(exe);
        cmd.creation_flags(0x08000000).kill_on_drop(true)
            .args(["--headless", "--disable-extensions", "--no-first-run", "--disable-background-networking", "--virtual-time-budget=15000", "--dump-dom"])
            .arg(format!("--user-data-dir={}",dir.join("profile").display())).arg(page);
        let output = tokio::time::timeout(Duration::from_secs(60), cmd.output()).await
            .map_err(|_| "สร้างภาพป้ายเดิมเกินเวลา".to_string())?.map_err(|e|e.to_string())?;
        let dom = String::from_utf8_lossy(&output.stdout);
        let png = dom.split_once("<output>").and_then(|(_,s)|s.split_once("</output>"))
            .map(|(s,_)|s).filter(|s|s.starts_with("data:image/png;base64,iVBOR"));
        png.filter(|_|output.status.success()).map(str::to_owned).ok_or_else(||"สร้างภาพป้ายเดิมไม่สำเร็จ".into())
    }.await;
    let _ = std::fs::remove_dir_all(dir);
    result
}
