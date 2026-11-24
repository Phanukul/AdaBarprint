use crate::models::TCNMLabelSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PrintableArea {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl PrintableArea {
    pub(crate) fn warning(
        &self,
        size: &TCNMLabelSize,
        legacy: bool,
        count: usize,
    ) -> Option<String> {
        if count == 0 {
            return None;
        }
        let cols = size.FNLbsCol.max(1) as usize;
        let rows = size.FNLbsRow.max(1) as usize;
        let used = count.min(cols.saturating_mul(rows));
        // Legacy raster pages clip translated artwork inside fixed cells.
        // The Windows vector renderer translates the whole cell.
        let margin = if legacy { 0.0 } else { 3.0 };
        let gap = if legacy { 0.0 } else { 2.0 };
        let x = size.FCLbsMagLeft.unwrap_or(margin)
            + if legacy {
                0.0
            } else {
                size.FCLbsOffsetX.unwrap_or_default()
            };
        let y = size.FCLbsMagTop.unwrap_or(margin)
            + if legacy {
                0.0
            } else {
                size.FCLbsOffsetY.unwrap_or_default()
            };
        let col = used.min(cols);
        let row = (used - 1) / cols + 1;
        let right =
            x + size.FCLbsWidth * col as f64 + size.FCLbsGapH.unwrap_or(gap) * (col - 1) as f64;
        let bottom =
            y + size.FCLbsHeight * row as f64 + size.FCLbsGapV.unwrap_or(gap) * (row - 1) as f64;
        let mut issues = Vec::new();
        for (amount, side, index) in [
            (self.left - x, "ซ้าย คอลัมน์", 1),
            (right - self.right, "ขวา คอลัมน์", col),
            (self.top - y, "บน แถว", 1),
            (bottom - self.bottom, "ล่าง แถว", row),
        ] {
            if amount > 0.01 {
                issues.push(format!("{side} {index} เกิน {amount:.2} มม."));
            }
        }
        if issues.is_empty() {
            None
        } else {
            Some(format!(
                "{} พ้นพื้นที่พิมพ์: {} กรุณาปรับที่ ขนาดและกระดาษ",
                size.FTLbsName,
                issues.join(" · ")
            ))
        }
    }
}

pub(crate) async fn read(
    printer: &str,
    tray: &str,
    ip: &str,
    size: &TCNMLabelSize,
) -> Result<Option<PrintableArea>, String> {
    if !ip.is_empty() {
        return Ok(None);
    }
    if size.FTLbsPaperSize.as_deref() == Some("ROLL") {
        let width = size
            .FCLbsPaperWidth
            .filter(|v| *v > 0.0)
            .ok_or("กรุณาระบุความกว้างกระดาษม้วน")?;
        let height = size
            .FCLbsPaperHeight
            .filter(|v| *v > 0.0)
            .ok_or("กรุณาระบุความยาวกระดาษต่อหน้า")?;
        return Ok(Some(PrintableArea {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: height,
        }));
    }
    query_driver(printer, tray, size).await.map(Some)
}

#[cfg(not(windows))]
async fn query_driver(_: &str, _: &str, _: &TCNMLabelSize) -> Result<PrintableArea, String> {
    Err("ต้องใช้ไดรเวอร์ Windows เพื่อตรวจพื้นที่พิมพ์".into())
}

#[cfg(windows)]
async fn query_driver(
    printer: &str,
    tray: &str,
    size: &TCNMLabelSize,
) -> Result<PrintableArea, String> {
    use std::time::Duration;
    let cfg = serde_json::json!({"printerName":printer,"trayName":tray,"paperSize":size.FTLbsPaperSize,
        "paperWidthMm":size.FCLbsPaperWidth,"paperHeightMm":size.FCLbsPaperHeight});
    let script = concat!(
        r#"
$ErrorActionPreference = 'Stop'
$cfg = $env:ADABAR_PAGE_QUERY | ConvertFrom-Json
Add-Type -AssemblyName System.Drawing
$doc = New-Object System.Drawing.Printing.PrintDocument
$doc.PrinterSettings.PrinterName = $cfg.printerName
if (!$doc.PrinterSettings.IsValid) { throw 'Printer unavailable' }
"#,
        include_str!("windows_page_settings.ps1"),
        r#"
$p = $doc.DefaultPageSettings.PrintableArea
# Some drivers expose portrait PrintableArea coordinates even with a landscape DEVMODE.
if ($landscape -and $p.Width -lt $p.Height) {
  $graphics = $doc.PrinterSettings.CreateMeasurementGraphics($doc.DefaultPageSettings)
  try {
    $graphics.PageUnit = [System.Drawing.GraphicsUnit]::Display
    $clip = $graphics.VisibleClipBounds
    if ($clip.Width -gt $clip.Height) {
      $p = [System.Drawing.RectangleF]::new($p.Y, $p.X, $clip.Width, $clip.Height)
    }
  } finally { $graphics.Dispose() }
}
@{left=$p.Left*0.254;top=$p.Top*0.254;right=$p.Right*0.254;bottom=$p.Bottom*0.254} | ConvertTo-Json -Compress
$doc.Dispose()
"#
    );
    let mut cmd = tokio::process::Command::new("powershell.exe");
    cmd.creation_flags(0x08000000)
        .kill_on_drop(true)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("ADABAR_PAGE_QUERY", cfg.to_string());
    let unavailable = || format!("อ่านพื้นที่พิมพ์ของ {printer} ไม่ได้ กรุณาตรวจเครื่องพิมพ์และถาดที่เลือก");
    let out = tokio::time::timeout(Duration::from_secs(15), cmd.output())
        .await
        .map_err(|_| unavailable())?
        .map_err(|_| unavailable())?;
    if !out.status.success() {
        return Err(unavailable());
    }
    let area: PrintableArea = serde_json::from_slice(&out.stdout).map_err(|_| unavailable())?;
    if area.right <= area.left || area.bottom <= area.top {
        return Err(unavailable());
    }
    Ok(area)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(width: f64, cols: usize, left: f64) -> TCNMLabelSize {
        serde_json::from_value(serde_json::json!({
            "FTLbsCode":"QA","FTLbsGroup":"","FTLbsName":"QA","FTLbsStaUse":"1",
            "FCLbsWidth":width,"FCLbsHeight":38.0,"FNLbsCol":cols,"FNLbsRow":7,
            "FCLbsMagLeft":left,"FCLbsMagTop":8.0,"FCLbsGapH":2.0,"FTLbsPaperSize":"A4"
        }))
        .unwrap()
    }

    #[test]
    fn samsung_a4_only_checks_occupied_slots_at_actual_size() {
        let area = PrintableArea {
            left: 4.445,
            top: 4.318,
            right: 205.6553,
            bottom: 292.65,
        };
        let mut large = sheet(100.0, 2, 8.0);
        let mut small = sheet(50.0, 4, 3.0);
        assert!(area.warning(&large, false, 1).is_none());
        assert!(area
            .warning(&large, false, 5)
            .unwrap()
            .contains("ขวา คอลัมน์ 2 เกิน 4.34"));
        assert!(area
            .warning(&small, false, 16)
            .unwrap()
            .contains("ขวา คอลัมน์ 4 เกิน 3.34"));
        for size in [&mut large, &mut small] {
            size.FCLbsMagLeft = Some(5.0);
            size.FCLbsGapH = Some(0.0);
            assert!(area.warning(size, false, 16).is_none());
            size.FCLbsOffsetX = Some(1.0);
            assert!(area.warning(size, false, 16).is_some());
            assert!(area.warning(size, true, 16).is_none());
            assert!(area.warning(size, false, 0).is_none());
        }
    }

    #[tokio::test]
    async fn roll_checks_the_configured_stock_instead_of_a4() {
        let mut size = sheet(32.0, 6, 3.0);
        size.FTLbsPaperSize = Some("ROLL".into());
        size.FCLbsPaperWidth = Some(104.0);
        size.FCLbsPaperHeight = Some(38.0);
        size.FCLbsHeight = 24.0;
        size.FCLbsOffsetX = Some(1.0);
        let area = read("Godex EZ520", "", "", &size).await.unwrap().unwrap();
        assert!(area.warning(&size, false, 3).is_none());
        assert!(area
            .warning(&size, false, 4)
            .unwrap()
            .contains("ขวา คอลัมน์ 4"));
        assert!(read("Raw", "", "127.0.0.1", &size).await.unwrap().is_none());
    }

    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "requires installed Samsung Windows printer driver"]
    async fn samsung_driver_reports_a4_clipping_without_printing() {
        let size = sheet(100.0, 2, 8.0);
        let area = read(
            "Samsung M262x 282x Series Class Driver",
            "Automatically Select",
            "",
            &size,
        )
        .await
        .unwrap()
        .unwrap();
        let warning = area.warning(&size, false, 5).unwrap();
        assert!(warning.contains("ขวา คอลัมน์ 2"), "{area:?} {warning}");
        eprintln!("{area:?} {warning}");
    }
}
