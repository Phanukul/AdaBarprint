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

/// Code128 pattern table (values 0..=106, standard 11-module bars/spaces, stop=13 modules).
const CODE128_PATTERNS: [&str; 107] = [
    "11011001100","11001101100","11001100110","10010011000","10010001100","10001001100",
    "10011001000","10011000100","10001100100","11001001000","11001000100","11000100100",
    "10110011100","10011011100","10011001110","10111001100","10011101100","10011100110",
    "11001110010","11001011100","11001001110","11011100100","11001110100","11101101110",
    "11101001100","11100101100","11100100110","11101100100","11100110100","11100110010",
    "11011011000","11011000110","11000110110","10100011000","10001011000","10001000110",
    "10110001000","10001101000","10001100010","11010001000","11000101000","11000100010",
    "10110111000","10110001110","10001101110","10111011000","10111000110","10001110110",
    "11101110110","11010001110","11000101110","11011101000","11011100010","11011101110",
    "11101011000","11101000110","11100010110","11101101000","11101100010","11100011010",
    "11101111010","11001000010","11110001010","10100110000","10100001100","10010110000",
    "10010000110","10000101100","10000100110","10110010000","10110000100","10011010000",
    "10011000010","10000110100","10000110010","11000010010","11001010000","11110111010",
    "11000010100","10001111010","10100111100","10010111100","10010011110","10111100100",
    "10011110100","10011110010","11110100100","11110010100","11110010010","11011011110",
    "11011110110","11110110110","10101111000","10100011110","10001011110","10111101000",
    "10111100010","11110101000","11110100010","10111011110","10111101110","11101011110",
    "11110101110","11010000100","11010010000","11010011100","1100011101011",
];

fn encode_ean13_modules(digits: &str) -> String {
    const L: [&str; 10] = ["0001101","0011001","0010011","0111101","0100011","0110001","0101111","0111011","0110111","0001011"];
    const G: [&str; 10] = ["0100111","0110011","0011011","0100001","0011101","0111001","0000101","0010001","0001001","0010111"];
    const R: [&str; 10] = ["1110010","1100110","1101100","1000010","1011100","1001110","1010000","1000100","1001000","1110100"];
    const PARITY: [&str; 10] = ["LLLLLL","LLGLGG","LLGGLG","LLGGGL","LGLLGG","LGGLLG","LGGGLL","LGLGLG","LGLGGL","LGGLGL"];

    let d: Vec<usize> = digits.chars().filter_map(|c| c.to_digit(10).map(|v| v as usize)).collect();
    if d.len() != 13 {
        return String::new();
    }
    let mut out = String::with_capacity(95);
    out.push_str("101");
    let par = PARITY[d[0]];
    for (i, ch) in par.chars().enumerate() {
        let dig = d[i + 1];
        out.push_str(if ch == 'G' { G[dig] } else { L[dig] });
    }
    out.push_str("01010");
    for &dig in &d[7..13] {
        out.push_str(R[dig]);
    }
    out.push_str("101");
    out
}

/// Encode barcode string to a scannable '1'/'0' bar pattern (EAN-13 if 13 digits valid check, otherwise Code128B).
pub fn encode_barcode_modules(code: &str, symbology: &str) -> String {
    let trimmed = code;
    if trimmed.is_empty() || !trimmed.bytes().all(|b| (32..=126).contains(&b)) {
        return String::new();
    }
    let sym = symbology.to_uppercase();
    if sym == "EAN13"
        && trimmed.len() == 13
        && trimmed.chars().all(|c| c.is_ascii_digit())
        && validate_barcode(trimmed, "EAN13").is_ok()
    {
        return encode_ean13_modules(trimmed);
    }

    // Fallback to Code 128B (supports all printable ASCII 32..=126)
    let mut out = String::from(CODE128_PATTERNS[104]); // Start B
    let mut checksum: usize = 104;
    let mut pos: usize = 1;
    for ch in trimmed.chars() {
        let ascii = ch as u32;
        if (32..=126).contains(&ascii) {
            let val = (ascii - 32) as usize;
            out.push_str(CODE128_PATTERNS[val]);
            checksum += val * pos;
            pos += 1;
        }
    }
    out.push_str(CODE128_PATTERNS[checksum % 103]);
    out.push_str(CODE128_PATTERNS[106]); // Stop
    out
}
