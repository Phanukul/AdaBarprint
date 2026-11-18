/// Barcode validation and check-digit calculation engine (ported from src/lib/barcode.ts)

fn calc_check(body: &str, weights: &[u32]) -> u32 {
    let total: u32 = body
        .chars()
        .zip(weights.iter())
        .filter_map(|(c, w)| c.to_digit(10).map(|d| d * w))
        .sum();
    (10 - (total % 10)) % 10
}

pub fn calc_ean13(body12: &str) -> String {
    calc_check(body12, &[1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3]).to_string()
}

pub fn calc_ean8(body7: &str) -> String {
    calc_check(body7, &[3, 1, 3, 1, 3, 1, 3]).to_string()
}

pub fn calc_upca(body11: &str) -> String {
    calc_check(body11, &[3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3]).to_string()
}

fn upce_expand(code8: &str) -> String {
    let bytes: Vec<char> = code8.chars().collect();
    if bytes.len() < 8 {
        return code8.to_string();
    }
    let ns = bytes[0];
    let s: String = bytes[1..7].iter().collect();
    let chk = bytes[7];
    let s5 = bytes[6];
    match s5 {
        '0' | '1' | '2' => format!("{}{}{}{}{}{}", ns, &s[0..2], s5, "0000", &s[2..5], chk),
        '3' => format!("{}{}{}{}{}{}", ns, &s[0..3], "00000", &s[3..5], "", chk),
        '4' => format!("{}{}{}{}{}", ns, &s[0..4], "00000", &s[4..5], chk),
        _ => format!("{}{}{}{}{}", ns, &s[0..5], "0000", s5, chk),
    }
}

pub fn validate_barcode(code: &str, symbology: &str) -> Result<(), String> {
    let code = code.trim();
    match symbology {
        "EAN13" => {
            if code.len() != 13 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err("EAN-13 ต้องมี 13 หลัก".to_string());
            }
            let expected = calc_ean13(&code[0..12]);
            if &code[12..13] != expected {
                return Err(format!("Check digit ผิด (ที่ถูกคือ {})", expected));
            }
            Ok(())
        }
        "EAN8" => {
            if code.len() != 8 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err("EAN-8 ต้องมี 8 หลัก".to_string());
            }
            let expected = calc_ean8(&code[0..7]);
            if &code[7..8] != expected {
                return Err("Check digit EAN-8 ผิด".to_string());
            }
            Ok(())
        }
        "UPCA" => {
            if code.len() != 12 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err("UPC-A ต้องมี 12 หลัก".to_string());
            }
            let expected = calc_upca(&code[0..11]);
            if &code[11..12] != expected {
                return Err("Check digit UPC-A ผิด".to_string());
            }
            Ok(())
        }
        "UPCE" => {
            if code.len() != 8
                || !(code.starts_with('0') || code.starts_with('1'))
                || !code.chars().all(|c| c.is_ascii_digit())
            {
                return Err("UPC-E ต้อง 8 หลัก ขึ้นต้น 0/1".to_string());
            }
            validate_barcode(&upce_expand(code), "UPCA")
        }
        "ITF14" => {
            if code.len() != 14 || !code.chars().all(|c| c.is_ascii_digit()) {
                return Err("ITF-14 ต้องมี 14 หลัก".to_string());
            }
            Ok(())
        }
        "CODE39" => {
            let valid = !code.is_empty()
                && code.chars().all(|c| {
                    c.is_ascii_uppercase() || c.is_ascii_digit() || "-. $/+%".contains(c)
                });
            if !valid {
                return Err("ตัวอักษร Code39 ไม่ถูกต้อง".to_string());
            }
            Ok(())
        }
        "CODE128" | "GS1128" | "QRCODE" | "DATAMATRIX" => {
            if code.is_empty() {
                return Err("ข้อมูลว่าง".to_string());
            }
            Ok(())
        }
        other => Err(format!("ไม่รู้จัก symbology: {}", other)),
    }
}
