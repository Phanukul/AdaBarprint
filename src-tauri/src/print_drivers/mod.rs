use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::templates::TemplateElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrinterBrand {
    Zebra,
    Honeywell,
    Tsc,
    Argox,
    Godex,
    Bixolon,
    Samsung,
    Windows,
}

impl PrinterBrand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "ZEBRA" => Some(Self::Zebra),
            "HONEYWELL" => Some(Self::Honeywell),
            "TSC" => Some(Self::Tsc),
            "ARGOX" => Some(Self::Argox),
            "GODEX" => Some(Self::Godex),
            "BIXOLON" => Some(Self::Bixolon),
            "SAMSUNG" => Some(Self::Samsung),
            "WINDOWS" => Some(Self::Windows),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zebra => "ZEBRA",
            Self::Honeywell => "HONEYWELL",
            Self::Tsc => "TSC",
            Self::Argox => "ARGOX",
            Self::Godex => "GODEX",
            Self::Bixolon => "BIXOLON",
            Self::Samsung => "SAMSUNG",
            Self::Windows => "WINDOWS",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Zebra => "Zebra",
            Self::Honeywell => "Honeywell",
            Self::Tsc => "TSC",
            Self::Argox => "ARGOX",
            Self::Godex => "GODEX",
            Self::Bixolon => "BIXOLON",
            Self::Samsung => "Samsung (A4)",
            Self::Windows => "Windows Printer",
        }
    }

    pub fn language_label(&self) -> &'static str {
        match self {
            Self::Zebra => "ZPL II",
            Self::Honeywell => "Fingerprint/DP",
            Self::Tsc => "TSPL",
            Self::Argox => "PPLZ (ZPL-compat)",
            Self::Godex => "EZPL",
            Self::Bixolon => "BXL Label Script",
            Self::Samsung | Self::Windows => "A4 / Windows Print",
        }
    }

    pub fn file_ext(&self) -> &'static str {
        match self {
            Self::Zebra => "zpl",
            Self::Honeywell => "dp",
            Self::Tsc => "tspl",
            Self::Argox => "pplz",
            Self::Godex => "ezpl",
            Self::Bixolon => "bxl",
            Self::Samsung | Self::Windows => "html",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DriverOptions {
    pub dpi: u32,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub gap_mm: f64,
    pub speed: u32,
    pub darkness: u32,
    pub copies: u32,
}

impl Default for DriverOptions {
    fn default() -> Self {
        Self {
            dpi: 203,
            label_width_mm: 100.0,
            label_height_mm: 38.0,
            gap_mm: 2.0,
            speed: 3,
            darkness: 10,
            copies: 1,
        }
    }
}

pub fn mm_to_dots(mm: f64, dpi: u32) -> i64 {
    ((mm / 25.4) * (dpi as f64)).round() as i64
}

fn dig<'a>(ctx: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = ctx;
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur)
}

fn dig_str(ctx: &Value, path: &str) -> String {
    match dig(ctx, path) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn when_condition(el: &TemplateElement, ctx: &Value) -> bool {
    let Some(when) = &el.when else {
        return true;
    };
    match dig(ctx, &when.path) {
        Some(Value::Bool(b)) => *b == when.eq,
        _ => false,
    }
}

fn format_price(val: Option<&Value>, prefix: &str, suffix: &str) -> String {
    let num = match val {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.parse::<f64>().ok(),
        _ => None,
    };
    let Some(n) = num else {
        return String::new();
    };
    format!("{}{:.2}{}", prefix, n, suffix)
}

fn get_text_value(el: &TemplateElement, ctx: &Value) -> String {
    let prefix = el.prefix.as_deref().unwrap_or("");
    let suffix = el.suffix.as_deref().unwrap_or("");
    match el.el_type.as_str() {
        "price" => {
            let val = el.field.as_deref().and_then(|f| dig(ctx, f));
            format_price(val, prefix, suffix)
        }
        "static" => {
            let raw = el.text.as_deref().unwrap_or("");
            raw.to_string()
        }
        "date" => Local::now().format("%d/%m/%y").to_string(),
        "time" => Local::now().format("%H:%M").to_string(),
        _ => {
            if let Some(f) = &el.field {
                dig_str(ctx, f)
            } else {
                String::new()
            }
        }
    }
}

fn barcode_kind<'a>(el: &'a TemplateElement, data: &str) -> &'a str {
    let kind = el.symbology.as_deref().unwrap_or("EAN13");
    match kind {
        "EAN13" | "EAN8" | "UPCA" | "CODE39" if crate::barcode::validate_barcode(data, kind).is_ok()
            && data == data.trim() => kind,
        _ => "CODE128",
    }
}

fn module_dots(el: &TemplateElement, kind: &str, data: &str, dpi: u32) -> i64 {
    let modules = match kind {
        "EAN13" | "UPCA" => 95,
        "EAN8" => 67,
        "CODE39" => 16 * (data.len() + 2),
        _ => 11 * (data.len() + 2) + 13,
    };
    (mm_to_dots(el.width.unwrap_or(25.0), dpi) / (modules as i64 + 22)).min(3)
}

// ─── GODEX EZPL ──────────────────────────────────────────────────────────────

fn generate_ezpl_label(ctx: &Value, elements: &[TemplateElement], opts: &DriverOptions) -> String {
    let mut lines = vec![
        format!("^Q{},{}", opts.label_height_mm.round() as i64, opts.gap_mm.round() as i64),
        format!("^W{}", opts.label_width_mm.round() as i64),
        format!("^H{}", opts.darkness.min(19)),
        format!("^S{}", opts.speed.clamp(1, 7)),
        "^ON".to_string(),
        format!("^P{}", opts.copies),
        "^L".to_string(),
    ];

    for el in elements {
        if !when_condition(el, ctx) {
            continue;
        }
        let x = mm_to_dots(el.x.unwrap_or(0.0), opts.dpi);
        let y = mm_to_dots(el.y.unwrap_or(0.0), opts.dpi);
        match el.el_type.as_str() {
            "text" | "price" | "static" | "date" | "time" => {
                let text = get_text_value(el, ctx);
                if text.is_empty() {
                    continue;
                }
                let font_mm = el.font_size.unwrap_or(8.0) * 0.352778;
                let fh = mm_to_dots(font_mm * 1.4, opts.dpi).max(12);
                let font = if fh >= 28 { "D" } else if fh >= 18 { "C" } else { "B" };
                lines.push(format!("A{font},{x},{y},1,1,0,0,{text}"));
            }
            "barcode" => {
                let data = dig_str(ctx,el.field.as_deref().unwrap_or("product.barcode"));
                if data.is_empty() { continue; }
                let kind = barcode_kind(el, &data);
                let symbol = match kind { "EAN13"=>"E", "EAN8"=>"B", "UPCA"=>"H", "CODE39"=>"A3", _=>"Q" };
                let dots = module_dots(el,kind,&data,opts.dpi);
                let height = mm_to_dots(el.height.unwrap_or(13.0),opts.dpi);
                let show = u8::from(el.show_text.unwrap_or(true));
                lines.push(format!("B{symbol},{},{y},{dots},{},{height},0,{show},{data}",x+11*dots,2*dots));
            }
            "qrcode" => {
                let data = el.field.as_deref().map(|f| dig_str(ctx, f)).unwrap_or_default();
                if data.is_empty() {
                    continue;
                }
                let size = mm_to_dots(el.width.unwrap_or(20.0), opts.dpi);
                let side = qrcode::QrCode::new(data.as_bytes()).map(|q| q.width() as i64).unwrap_or(21);
                let dots = (size / (side + 8)).clamp(1, 40);
                lines.push(format!("W{},{},5,2,M,8,{dots},{},0\r\n{data}", x + 4*dots, y + 4*dots, data.len()));
            }
            "rect" => {
                let w = mm_to_dots(el.width.unwrap_or(100.0), opts.dpi);
                let h = mm_to_dots(el.height.unwrap_or(38.0), opts.dpi);
                lines.push(format!("R{x},{y},{},{},1,1", x+w, y+h));
            }
            _ => {}
        }
    }

    lines.push("E".to_string());
    lines.join("\r\n")
}

// ─── ZEBRA ZPL II & ARGOX PPLZ ───────────────────────────────────────────────

fn generate_zpl_like_label(
    ctx: &Value,
    elements: &[TemplateElement],
    opts: &DriverOptions,
    is_argox: bool,
) -> String {
    let w_dots = mm_to_dots(opts.label_width_mm, opts.dpi);
    let h_dots = mm_to_dots(opts.label_height_mm, opts.dpi);
    let dark = if is_argox { opts.darkness.min(15) } else { opts.darkness.min(30) };

    let mut lines = vec![
        "^XA".to_string(),
        "^CI28".to_string(),
        format!("^PW{}", w_dots),
        format!("^LL{}", h_dots),
        format!("^PR{}", opts.speed),
        format!("~SD{}", dark),
        format!("^PQ{}", opts.copies),
    ];

    for el in elements {
        if !when_condition(el, ctx) {
            continue;
        }
        let x = mm_to_dots(el.x.unwrap_or(0.0), opts.dpi);
        let y = mm_to_dots(el.y.unwrap_or(0.0), opts.dpi);
        match el.el_type.as_str() {
            "text" | "price" | "static" | "date" | "time" => {
                let text = get_text_value(el, ctx);
                if text.is_empty() {
                    continue;
                }
                let fh = mm_to_dots(el.font_size.unwrap_or(4.0) * 0.6, opts.dpi).max(16);
                let fw = (fh as f64 * 0.8).round() as i64;
                lines.push(format!("^FO{},{}^A0N,{},{}^FD{}^FS", x, y, fh, fw, text));
            }
            "barcode" => {
                let data = dig_str(ctx,el.field.as_deref().unwrap_or("product.barcode"));
                if data.is_empty() {
                    continue;
                }
                let h = mm_to_dots(el.height.unwrap_or(13.0), opts.dpi).max(30);
                let show = if el.show_text.unwrap_or(true) { "Y" } else { "N" };
                let sym = barcode_kind(el, &data);
                let dots = module_dots(el,sym,&data,opts.dpi);
                let cmd = match sym {
                    "EAN13" => format!("^BEN,{},{},N", h, show),
                    "EAN8" => format!("^B8N,{},{},N", h, show),
                    "UPCA" => format!("^BUN,{},{},N,Y", h, show),
                    "CODE39" => format!("^B3N,N,{},{},N", h, show),
                    _ => format!("^BCN,{},{},N,N", h, show),
                };
                let field = if matches!(sym,"EAN13"|"EAN8"|"UPCA") { &data[..data.len()-1] } else { &data };
                lines.push(format!("^FO{},{y}^BY{dots}{cmd}^FD{field}^FS", x + 11*dots));
            }
            "qrcode" => {
                let data = el.field.as_deref().map(|f| dig_str(ctx, f)).unwrap_or_default();
                if data.is_empty() {
                    continue;
                }
                lines.push(format!("^FO{},{}^BQN,2,5^FDMA,{}^FS", x, y, data));
            }
            "rect" => {
                let w = mm_to_dots(el.width.unwrap_or(100.0), opts.dpi);
                let h = mm_to_dots(el.height.unwrap_or(38.0), opts.dpi);
                lines.push(format!("^FO{},{}^GB{},{},2^FS", x, y, w, h));
            }
            _ => {}
        }
    }

    lines.push("^XZ".to_string());
    lines.join("\r\n")
}

// ─── TSC TSPL & BIXOLON BXL ──────────────────────────────────────────────────

fn generate_tspl_like_label(
    ctx: &Value,
    elements: &[TemplateElement],
    opts: &DriverOptions,
) -> String {
    let mut lines = vec![
        format!("SIZE {} mm, {} mm", opts.label_width_mm, opts.label_height_mm),
        format!("GAP {} mm, 0 mm", opts.gap_mm),
        format!("SPEED {}", opts.speed.min(6)),
        format!("DENSITY {}", opts.darkness.min(15)),
        "DIRECTION 1".to_string(),
        "CODEPAGE UTF-8".to_string(),
        "CLS".to_string(),
    ];

    for el in elements {
        if !when_condition(el, ctx) {
            continue;
        }
        let x = mm_to_dots(el.x.unwrap_or(0.0), opts.dpi);
        let y = mm_to_dots(el.y.unwrap_or(0.0), opts.dpi);
        match el.el_type.as_str() {
            "text" | "price" | "static" | "date" | "time" => {
                let text = get_text_value(el, ctx).replace('"', "'");
                if text.is_empty() {
                    continue;
                }
                let mul = ((el.font_size.unwrap_or(8.0) / 8.0).round() as i64).clamp(1, 4);
                lines.push(format!("TEXT {},{},\"3\",0,{},{},\"{}\"", x, y, mul, mul, text));
            }
            "barcode" => {
                let data = dig_str(ctx,el.field.as_deref().unwrap_or("product.barcode"));
                if data.is_empty() {
                    continue;
                }
                let h = mm_to_dots(el.height.unwrap_or(13.0), opts.dpi).max(30);
                let show = if el.show_text.unwrap_or(true) { 1 } else { 0 };
                let sym = barcode_kind(el, &data);
                let dots = module_dots(el,sym,&data,opts.dpi);
                let btype = match sym {
                    "EAN13" => "EAN13",
                    "EAN8" => "EAN8",
                    "UPCA" => "UPCA",
                    "CODE39" => "39",
                    _ => "128",
                };
                lines.push(format!("BARCODE {},{},\"{}\",{},{},0,{dots},{dots},\"{}\"", x + 11*dots, y, btype, h, show, data));
            }
            "qrcode" => {
                let data = el.field.as_deref().map(|f| dig_str(ctx, f)).unwrap_or_default();
                if data.is_empty() {
                    continue;
                }
                lines.push(format!("QRCODE {},{},M,5,A,0,\"{}\"", x, y, data));
            }
            "rect" => {
                let w = mm_to_dots(el.width.unwrap_or(100.0), opts.dpi);
                let h = mm_to_dots(el.height.unwrap_or(38.0), opts.dpi);
                lines.push(format!("BOX {},{},{},{},2", x, y, x + w, y + h));
            }
            _ => {}
        }
    }

    lines.push(format!("PRINT 1,{}", opts.copies));
    lines.join("\r\n")
}

// ─── HONEYWELL DP (Direct Protocol / Fingerprint) ─────────────────────────────

fn generate_dp_label(ctx: &Value, elements: &[TemplateElement], opts: &DriverOptions) -> String {
    let mut lines = vec![
        "INPUT OFF".to_string(),
        "NASC 8".to_string(),
        "CLL".to_string(),
    ];

    for el in elements {
        if !when_condition(el, ctx) {
            continue;
        }
        let x = mm_to_dots(el.x.unwrap_or(0.0), opts.dpi);
        let y = mm_to_dots(el.y.unwrap_or(0.0), opts.dpi);
        match el.el_type.as_str() {
            "text" | "price" | "static" | "date" | "time" => {
                let text = get_text_value(el, ctx).replace('"', "'");
                if text.is_empty() {
                    continue;
                }
                let pt = (el.font_size.unwrap_or(8.0) as i64).clamp(6, 36);
                lines.push(format!("PP {},{}", x, y));
                lines.push(format!("FT \"Swiss 721 BT\",{}", pt));
                lines.push(format!("PT \"{}\"", text));
            }
            "barcode" => {
                let data = dig_str(ctx,el.field.as_deref().unwrap_or("product.barcode"));
                if data.is_empty() {
                    continue;
                }
                let h = mm_to_dots(el.height.unwrap_or(13.0), opts.dpi).max(30);
                let kind = barcode_kind(el, &data);
                let dots = module_dots(el, kind, &data, opts.dpi);
                lines.push(format!("PP {},{}", x + 11*dots, y));
                lines.push(format!("BT \"{kind}\""));
                lines.push(format!("BM {dots}"));
                lines.push(format!("BH {}", h));
                lines.push(format!("PB \"{}\"", data));
            }
            "qrcode" => {
                let data = el.field.as_deref().map(|f| dig_str(ctx, f)).unwrap_or_default();
                if data.is_empty() {
                    continue;
                }
                lines.push(format!("PP {},{}", x, y));
                lines.push("BT \"QRCODE\"".to_string());
                lines.push(format!("PB \"{}\"", data));
            }
            _ => {}
        }
    }

    lines.push(format!("PF {}", opts.copies));
    lines.join("\r\n")
}

// ─── Unified Public API ──────────────────────────────────────────────────────

pub fn generate_commands(
    brand: PrinterBrand,
    labels: &[(Value, u32)],
    elements: &[TemplateElement],
    opts: &DriverOptions,
) -> Result<String, String> {
    let mut parts = Vec::new();
    for (ctx, qty) in labels {
        for el in elements.iter().filter(|el| el.el_type == "barcode" && when_condition(el,ctx)) {
            let data = dig_str(ctx,el.field.as_deref().unwrap_or("product.barcode"));
            if data.is_empty() || !data.bytes().all(|b| (32..=126).contains(&b)) {
                return Err("ข้อมูลบาร์โค้ดว่างหรือเครื่องพิมพ์ LAN เข้ารหัสไม่ได้".into());
            }
            if module_dots(el, barcode_kind(el, &data), &data, opts.dpi) < 1 {
                return Err(format!("พื้นที่บาร์โค้ดแคบเกินไปสำหรับ {data} กรุณาขยายช่องบาร์โค้ด"));
            }
            let reserved = match brand {
                PrinterBrand::Zebra | PrinterBrand::Argox => data.contains(['^','~','>']),
                PrinterBrand::Tsc | PrinterBrand::Bixolon | PrinterBrand::Honeywell => data.contains('"'),
                _ => false,
            };
            if reserved { return Err(format!("รหัส {data} มีอักขระพิเศษสำหรับคำสั่งเครื่องพิมพ์ ใช้ไดรเวอร์ Windows")); }
        }
        for _ in 0..(*qty).max(1) {
            let cmd = match brand {
                PrinterBrand::Godex => generate_ezpl_label(ctx, elements, opts),
                PrinterBrand::Zebra => generate_zpl_like_label(ctx, elements, opts, false),
                PrinterBrand::Argox => generate_zpl_like_label(ctx, elements, opts, true),
                PrinterBrand::Tsc | PrinterBrand::Bixolon => generate_tspl_like_label(ctx, elements, opts),
                PrinterBrand::Honeywell => generate_dp_label(ctx, elements, opts),
                PrinterBrand::Samsung | PrinterBrand::Windows => format!("A4_PRINT:{}", ctx.get("product").and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("")),
            };
            parts.push(cmd);
        }
    }
    Ok(parts.join("\r\n"))
}

pub fn generate_test_pattern(brand: PrinterBrand, dpi: u32, width_mm: f64, height_mm: f64) -> String {
    let w_dots = mm_to_dots(width_mm, dpi);
    let h_dots = mm_to_dots(height_mm, dpi);
    match brand {
        PrinterBrand::Godex => format!(
            "^Q{},2\r\n^W{}\r\n^H10\r\n^S2\r\n^P1\r\n^L\r\nAC,20,20,1,1,0,0,AdaPos PrintBar TEST\r\nBE,30,70,2,4,80,0,1,8850001000019\r\nE",
            height_mm.round() as i64, width_mm.round() as i64
        ),
        PrinterBrand::Zebra | PrinterBrand::Argox => format!(
            "^XA\r\n^PW{}\r\n^LL{}\r\n^FO20,20^A0N,30,25^FDAdaPos 6.0 PrintBar TEST^FS\r\n^FO20,70^BY2^BEN,80,Y,N^FD885000100001^FS\r\n^XZ",
            w_dots, h_dots
        ),
        PrinterBrand::Tsc | PrinterBrand::Bixolon => format!(
            "SIZE {} mm, {} mm\r\nGAP 2 mm, 0 mm\r\nCLS\r\nTEXT 20,20,\"3\",0,1,1,\"AdaPos 6.0 PrintBar TEST\"\r\nBARCODE 20,70,\"EAN13\",80,1,0,2,2,\"8850001000019\"\r\nPRINT 1,1",
            width_mm, height_mm
        ),
        PrinterBrand::Honeywell => {
            "INPUT OFF\r\nCLL\r\nPP 20,20\r\nFT \"Swiss 721 BT\",12\r\nPT \"AdaPos 6.0 PrintBar TEST\"\r\nPP 20,70\r\nBT \"EAN13\"\r\nBH 80\r\nPB \"8850001000019\"\r\nPF 1".to_string()
        }
        PrinterBrand::Samsung | PrinterBrand::Windows => {
            "Windows Print Mode (System Printer Driver)".to_string()
        }
    }
}

pub async fn tcp_test(ip: &str, port: u16) -> (bool, String, u128) {
    let start = Instant::now();
    let addr = format!("{}:{}", ip, port);
    match timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => (true, "เชื่อมต่อสำเร็จ".to_string(), start.elapsed().as_millis()),
        Ok(Err(e)) => (false, format!("เชื่อมต่อไม่สำเร็จ: {}", e), start.elapsed().as_millis()),
        Err(_) => (false, "หมดเวลาเชื่อมต่อ (5 วินาที)".to_string(), start.elapsed().as_millis()),
    }
}

pub async fn tcp_send(ip: &str, port: u16, data: &str) -> (bool, String, usize) {
    let addr = format!("{}:{}", ip, port);
    match timeout(Duration::from_secs(10), TcpStream::connect(&addr)).await {
        Ok(Ok(mut stream)) => {
            let bytes = data.as_bytes();
            match timeout(Duration::from_secs(10), stream.write_all(bytes)).await {
                Ok(Ok(())) => {
                    let _ = stream.flush().await;
                    (
                        true,
                        format!("ส่งไปยัง {}:{} สำเร็จ ({} bytes)", ip, port, bytes.len()),
                        bytes.len(),
                    )
                }
                Ok(Err(e)) => (false, format!("ส่งข้อมูลไม่สำเร็จ: {}", e), 0),
                Err(_) => (false, format!("หมดเวลาส่งข้อมูล ({}:{})", ip, port), 0),
            }
        }
        Ok(Err(e)) => (false, format!("ผิดพลาด: {}", e), 0),
        Err(_) => (false, format!("หมดเวลาเชื่อมต่อ ({}:{}) 10 วินาที", ip, port), 0),
    }
}

#[cfg(test)]
mod barcode_output_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mixed_product_codes_use_a_supported_command_and_preserve_the_value() {
        let el: TemplateElement = serde_json::from_value(json!({
            "type":"barcode", "field":"product.barcode", "symbology":"EAN13", "width":28
        })).unwrap();
        for (brand, command) in [(PrinterBrand::Godex, "BQ,"), (PrinterBrand::Zebra,"^BCN,"),
            (PrinterBrand::Argox,"^BCN,"), (PrinterBrand::Tsc,"\"128\""), (PrinterBrand::Bixolon,"\"128\""),
            (PrinterBrand::Honeywell,"BT \"CODE128\"")] {
            for value in ["QA26112301B02","8850001000014"] {
                let result = generate_commands(brand, &[(json!({"product":{"barcode":value}}),1)],
                    std::slice::from_ref(&el), &DriverOptions::default()).unwrap();
                assert!(result.contains(command) && result.contains(value), "{result}");
            }
            assert!(generate_commands(brand, &[(json!({"product":{"barcode":""}}),1)],
                std::slice::from_ref(&el), &DriverOptions::default()).is_err());
        }
    }
}
