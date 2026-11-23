//! Canonical StoreFront promotion rules. A shelf price is not a basket total.
//! Only an unconditional per-unit reward may become a numeric shelf price.
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Default)]
pub struct ProductScope {
    pub fields: HashMap<String, String>,
}

/// Use the same catalog dimensions as StoreFront's promotion adapter.
/// Missing columns remain unknown (not a false match).
pub async fn product_scope(pool: &SqlitePool, product: &str) -> Result<ProductScope, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM TCNMPdt WHERE FTPdtCode=?")
        .bind(product).fetch_optional(pool).await?;
    let mut scope = ProductScope::default();
    if let Some(row) = row {
        for (kind, column) in [("brand", "FTPbnCode"), ("group", "FTTcgCode"),
            ("type", "FTPtyCode"), ("model", "FTPmoCode"), ("department", "FTPgpChain")] {
            if let Ok(value) = row.try_get::<Option<String>, _>(column) {
                scope.fields.insert(kind.into(), value.unwrap_or_default().trim().into());
            }
        }
        if scope.fields.get("group").map(|s| s.is_empty()).unwrap_or(false) {
            if let Some(chain) = scope.fields.get("department").cloned() {
                scope.fields.insert("group".into(), chain);
            }
        }
    }
    Ok(scope)
}

pub fn reference_matches(reference: &Value, product: &str, barcode: &str, scope: &ProductScope) -> Result<bool, String> {
    let kind = reference["type"].as_str().unwrap_or("").trim().to_ascii_lowercase();
    let code = reference["code"].as_str().unwrap_or("").trim();
    let field = match kind.as_str() {
        "all" => return Ok(true),
        "sku" | "skc" => return Ok(!code.is_empty() && code == product),
        "barcode" | "bar" => return Ok(!code.is_empty() && code == barcode),
        "brand" | "brd" => "brand",
        "group" | "grp" => "group",
        "type" | "typ" => "type",
        "model" | "mdl" => "model",
        "department" | "dpt" => "department",
        _ => return Err(format!("ยังไม่รองรับขอบเขตโปรโมชั่นชนิด {kind} กรุณาตรวจเงื่อนไขหรือเลือกพิมพ์ราคาปกติ")),
    };
    let value = scope.fields.get(field).ok_or_else(|| format!("ข้อมูลสินค้าไม่มีมิติ {field} สำหรับตรวจขอบเขตโปรโมชั่น"))?;
    Ok(!code.is_empty() && code == value)
}

pub struct UnitReward { pub condition: i64, pub value: f64 }

fn nonempty(value: &Value) -> bool {
    !value.is_null() && value.as_array().map(|v| !v.is_empty()).unwrap_or(true)
}

/// Match exact product/barcode groups using the same include-minus-exclude
/// semantics as StoreFront. Unknown dimensions must not silently match nothing.
pub fn matches_group(group: &Value, product: &str, barcode: &str) -> Result<bool, String> {
    matches_group_with_scope(group, product, barcode, &ProductScope::default())
}

pub fn matches_group_with_scope(group: &Value, product: &str, barcode: &str, scope: &ProductScope) -> Result<bool, String> {
    if group["sourceType"].as_str() == Some("priceAdjustment") || nonempty(&group["priceAdjustmentSources"]) {
        return Err("โปรโมชั่นอ้างอิงเอกสารปรับราคา ต้องอ่านราคาจากเอกสารที่มีผล".into());
    }
    let include = group["include"].as_array().ok_or("โปรโมชั่นไม่มีรายการสินค้าในกลุ่ม")?;
    let mut included = false;
    for r in include { included |= reference_matches(r, product, barcode, scope)?; }
    if !included { return Ok(false); }
    if let Some(exclude) = group["exclude"].as_array() {
        for r in exclude { if reference_matches(r, product, barcode, scope)? { return Ok(false); } }
    }
    if included && (!group["minUnitPrice"].is_null() || !group["maxUnitPrice"].is_null()) {
        return Err("โปรโมชั่นมีเงื่อนไขราคาขั้นต่ำหรือสูงสุด ต้องตรวจเงื่อนไขก่อนสร้างป้าย".into());
    }
    Ok(included)
}

pub fn unit_reward(rule: &Value, product: &str, barcode: &str) -> Result<Option<UnitReward>, String> {
    unit_reward_with_scope(rule, product, barcode, &ProductScope::default())
}

pub fn unit_reward_with_scope(rule: &Value, product: &str, barcode: &str, scope: &ProductScope) -> Result<Option<UnitReward>, String> {
    let status = rule["status"].as_str().unwrap_or("").to_ascii_lowercase();
    if ["closed", "inactive", "cancelled", "canceled", "draft"].contains(&status.as_str()) { return Ok(None); }
    let groups = rule["groups"].as_object().ok_or("โปรโมชั่นไม่มีข้อมูลกลุ่มสินค้า")?;
    let mut matched = Vec::new();
    for (key, group) in groups { if matches_group_with_scope(group, product, barcode, scope)? { matched.push(key.as_str()); } }
    if matched.is_empty() { return Ok(None); }
    if nonempty(&rule["who"]) || nonempty(&rule["limits"]) || nonempty(&rule["when"]["calendars"])
        || !matches!(rule["audienceType"].as_str(), None | Some("all")) {
        return Err("โปรโมชั่นนี้มีเงื่อนไขสิทธิ์ เวลา หรือจำนวนครั้ง ต้องแสดงเงื่อนไขบนป้ายก่อนใช้ราคาพิเศษ".into());
    }
    let rewards = rule["get"].as_array().ok_or("โปรโมชั่นไม่มีรายละเอียดสิทธิ์ที่ได้รับ")?;
    if rewards.len() != 1 { return Err("โปรโมชั่นมีหลายสิทธิ์ ต้องแสดงรายละเอียดให้ครบก่อนพิมพ์".into()); }
    let reward = &rewards[0];
    let target = reward["target"].as_str().unwrap_or("@bill");
    if !matched.contains(&target) || target.starts_with('@') {
        return Err("โปรโมชั่นคิดส่วนลดระดับบิลหรือชุดสินค้า ไม่สามารถใช้เป็นราคาต่อชิ้นได้".into());
    }
    // A single-unit buy clause can repeat independently for each purchased unit.
    if let Some(buy) = rule["buy"].as_array() {
        if buy.len() > 1 { return Err("โปรโมชั่นต้องซื้อสินค้าร่วมกันหลายกลุ่ม ต้องแสดงเงื่อนไขก่อนพิมพ์".into()); }
        for clause in buy {
            if clause["group"].as_str() != Some(target)
                || !matches!(clause["basis"].as_str(), None | Some("qty"))
                || clause["min"].as_f64().unwrap_or(1.0) != 1.0
                || !clause["max"].is_null() || !clause["every"].is_null()
                || nonempty(&clause["tiers"]) || nonempty(&clause["everySteps"]) {
                return Err("โปรโมชั่นมีเงื่อนไขจำนวนหรือยอดซื้อ ต้องแสดงเงื่อนไขบนป้าย ไม่ใช่ราคาต่อชิ้น".into());
            }
        }
    }
    if !reward["qty"].is_null() || !reward["pick"].is_null() {
        return Err("โปรโมชั่นจำกัดชิ้นที่ได้รับส่วนลด ต้องแสดงเงื่อนไขก่อนพิมพ์".into());
    }
    let condition = match reward["type"].as_str().unwrap_or("").to_ascii_lowercase().as_str() {
        "discountpercent" | "dp" => 2,
        "newprice" | "np" => 3,
        // DA is a per-round total, not necessarily a per-unit reduction.
        _ => return Err("โปรโมชั่นนี้ไม่ใช่ราคาพิเศษต่อชิ้น ต้องแสดงรายละเอียดสิทธิ์บนป้าย".into()),
    };
    let value = reward["value"].as_f64().ok_or("โปรโมชั่นไม่มีมูลค่าสิทธิ์ที่ถูกต้อง")?;
    if !value.is_finite() || value < 0.0 || (condition == 2 && value > 100.0) {
        return Err("มูลค่าสิทธิ์โปรโมชั่นไม่ถูกต้อง".into());
    }
    Ok(Some(UnitReward { condition, value }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn rule() -> Value { json!({"groups":{"A":{"include":[{"type":"sku","code":"P1"}]}},"buy":[{"group":"A","basis":"qty","min":1}],"get":[{"type":"NP","target":"A","value":79}]}) }
    #[test]
    fn catalog_dimensions_match_and_exclusions_win() {
        let mut scope = ProductScope::default();
        for (kind, alias) in [("brand","BRD"),("group","GRP"),("type","TYP"),("model","MDL"),("department","DPT")] {
            scope.fields.insert(kind.into(), "A1".into());
            let mut r = rule();
            r["groups"]["A"]["include"] = json!([{"type":alias,"code":"A1"}]);
            assert_eq!(unit_reward_with_scope(&r,"P1","B1",&scope).unwrap().unwrap().value,79.0);
            r["groups"]["A"]["exclude"] = json!([{"type":"barcode","code":"B1"}]);
            assert!(unit_reward_with_scope(&r,"P1","B1",&scope).unwrap().is_none());
            r["groups"]["A"]["include"][0]["code"] = json!("OTHER");
            assert!(unit_reward_with_scope(&r,"P2","B2",&scope).unwrap().is_none());
        }
    }
    #[test]
    fn unrelated_conditional_promotion_does_not_block_product() {
        let scope = ProductScope { fields: [("group".into(),"OTHER".into())].into_iter().collect() };
        let mut r=rule();
        r["groups"]["A"]["include"]=json!([{"type":"GRP","code":"G1"}]);
        r["buy"][0]["min"]=json!(3);
        assert!(unit_reward_with_scope(&r,"P1","B1",&scope).unwrap().is_none());
        assert!(unit_reward_with_scope(&r,"P1","B1",&ProductScope::default()).is_err());
    }
    #[test] fn exact_product_price() {
        assert_eq!(unit_reward(&rule(),"P1","B1").unwrap().unwrap().value,79.0);
        assert!(unit_reward(&rule(),"P2","B2").unwrap().is_none());
    }
    #[test] fn quantity_and_bill_rewards_are_not_unit_prices() {
        let mut r=rule();r["buy"][0]["min"]=json!(3);
        assert!(unit_reward(&r,"P1","B1").is_err());
        let mut r=rule();r["get"][0]["target"]=json!("@bill");
        assert!(unit_reward(&r,"P1","B1").is_err());
    }
    #[test] fn excluded_product_does_not_receive_reward() {
        let mut r=rule();r["groups"]["A"]["exclude"]=json!([{"type":"barcode","code":"B1"}]);
        assert!(unit_reward(&r,"P1","B1").unwrap().is_none());
    }
}
