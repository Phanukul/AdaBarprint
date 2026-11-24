use serde_json::{json, Value};
use sqlx::SqlitePool;

// Coordinates are fractions of the label, font sizes are fractions of its height.
// Store the result in millimetres and points, like every document saved by Design.
struct Canvas {
    width: f64,
    height: f64,
    elements: Vec<Value>,
}
impl Canvas {
    fn layer(&mut self, kind: &str, name: &str, bounds: [f64; 4], props: Value) {
        let [x, y, w, h] = bounds;
        let mut el = json!({"id": format!("layer-{}", self.elements.len()+1), "label":name, "type":kind,
            "x":x*self.width,"y":y*self.height,"width":w*self.width,"height":h*self.height});
        el.as_object_mut()
            .unwrap()
            .extend(props.as_object().unwrap().clone());
        self.elements.push(el);
    }
    fn text(&mut self, name: &str, field: &str, bounds: [f64; 4], size: f64, color: &str) {
        let props = json!({"field":field,"fontSize":size*self.height*72.0/25.4,
            "align":"center","fontWeight":"bold","color":color});
        self.layer("text", name, bounds, props);
    }
    fn caption(&mut self, text: &str, bounds: [f64; 4], size: f64, color: &str) {
        self.layer(
            "static",
            text,
            bounds,
            json!({"text":text,"fontSize":size*self.height*72.0/25.4,
            "align":"center","fontWeight":"bold","color":color}),
        );
    }
    fn rect(&mut self, name: &str, bounds: [f64; 4], color: &str, radius: f64) {
        self.layer(
            "rect",
            name,
            bounds,
            json!({"fill":color,"borderRadius":radius,"borderWidth":0}),
        );
    }
    fn barcode(&mut self, bounds: [f64; 4]) {
        self.layer(
            "barcode",
            "บาร์โค้ด",
            bounds,
            json!({"field":"product.barcode","showText":true,"fontSize":6}),
        );
    }
    fn old_price(&mut self, bounds: [f64; 4], size: f64) {
        self.layer("price","ราคาก่อนลด",bounds,json!({"field":"price.old_value","prefix":"ราคาปกติ ",
            "fontSize":size*self.height*72.0/25.4,"strike":true,"align":"center","when":{"path":"promotion.is_discount","eq":true}}));
    }
}

fn standard_layout(code: &str, name: &str, width: f64, height: f64, promo: bool) -> Value {
    let mut c = Canvas {
        width,
        height,
        elements: vec![],
    };
    let green = "#3b6e38";
    let red = "#b91c1c";
    let ink = "#111111";
    let mut bg = "#ffffff";
    let mut border = "#cbd5e1";
    let mut stroke = 0.25;
    let period = if promo {
        "promotion.interval"
    } else {
        "tag.print_date"
    };
    match code {
        "UFM-PRO-A5" | "UFM-SALE-8" | "UFM-NOR-16" => {
            border = if promo { red } else { green };
            stroke = if code == "UFM-PRO-A5" { 2.5 } else { 0.3 };
            c.rect(
                "แถบหัวป้าย",
                [0., 0., 0.975, 0.10],
                if promo { "#f5c518" } else { green },
                0.,
            );
            c.rect("พื้นชื่อสาขา", [0.02, 0.015, 0.17, 0.065], "#c9302c", 5.);
            c.text(
                "ชื่อสาขา",
                "store.name",
                [0.02, 0.015, 0.17, 0.065],
                0.029,
                "#ffffff",
            );
            c.rect(
                "พื้นวันที่",
                [0.22, 0.015, 0.53, 0.065],
                if promo { "#ca8a04" } else { "#166534" },
                1.0,
            );
            c.rect(
                "พื้นชนิดราคา",
                [0.79, 0.015, 0.16, 0.065],
                if promo { "#991b1b" } else { "#ffffff" },
                1.0,
            );
            c.text(
                "วันที่",
                period,
                [0.20, 0.01, 0.57, 0.078],
                0.028,
                if promo { ink } else { "#ffffff" },
            );
            c.caption(
                if promo {
                    if code == "UFM-PRO-A5" {
                        "安い!"
                    } else {
                        "SALE"
                    }
                } else {
                    "ราคาปกติ"
                },
                [0.78, 0.01, 0.18, 0.078],
                0.031,
                if promo { "#ffffff" } else { green },
            );
            c.text(
                "ชื่อสินค้า",
                "product.name",
                [0.02, 0.11, 0.94, 0.09],
                0.043,
                ink,
            );
            c.text(
                "ชื่อรอง",
                "product.name_oth",
                [0.02, 0.2, 0.94, 0.065],
                0.027,
                ink,
            );
            if code == "UFM-NOR-16" {
                c.text(
                    "ราคาขาย",
                    "price.value",
                    [0.03, 0.36, 0.93, 0.31],
                    0.142,
                    ink,
                );
                c.barcode([0.04, 0.72, 0.69, 0.21]);
                c.caption("บาท", [0.74, 0.75, 0.21, 0.14], 0.05, ink);
            } else {
                c.layer(
                    "barcode",
                    "บาร์โค้ด",
                    [0.035, 0.53, 0.30, 0.12],
                    json!({"field":"product.barcode","showText":false}),
                );
                c.rect(
                    "พื้นเลขบาร์โค้ด",
                    [0.04, 0.745, 0.29, 0.06],
                    if promo { red } else { green },
                    4.,
                );
                c.layer("text","เลขบาร์โค้ด",[0.04,0.745,0.29,0.06],json!({"field":"product.barcode","prefix":"BARCODE: ","align":"center","fontWeight":"bold","fontSize":height*0.023*72.0/25.4,"color":"#ffffff"}));
                c.layer(
                    "image",
                    "รูปสินค้า",
                    [0.035, 0.44, 0.30, 0.29],
                    json!({"field":"product.image","objectFit":"contain","when":{"path":"product.has_image","eq":true}}),
                );
                if promo {
                    c.old_price([0.48, 0.34, 0.47, 0.09], 0.035);
                }
                c.text(
                    "ราคาขาย",
                    "price.value",
                    [0.35, 0.43, 0.60, 0.32],
                    0.15,
                    if promo { red } else { ink },
                );
                c.caption(
                    "฿/Pcs.",
                    [0.73, 0.75, 0.22, 0.11],
                    0.046,
                    if promo { red } else { ink },
                );
            }
        }
        "UFM-TIMESALE" => {
            c.rect(
                "พื้นโปรโมชั่น",
                [0., 0., 0.39, 1.],
                if promo { "#facc15" } else { green },
                0.,
            );
            c.text(
                "สาขา",
                "store.name",
                [0.01, 0.03, 0.37, 0.12],
                0.065,
                "#ffffff",
            );
            c.caption(
                if promo {
                    "Time Sale!"
                } else {
                    "ราคาปกติ"
                },
                [0.01, 0.25, 0.37, 0.26],
                0.14,
                if promo { red } else { "#ffffff" },
            );
            if promo {
                c.text(
                    "ส่วนลด",
                    "promotion.discount_percent",
                    [0.01, 0.14, 0.37, 0.15],
                    0.095,
                    red,
                );
            }
            c.text("ช่วงราคา", period, [0.01, 0.57, 0.37, 0.14], 0.04, ink);
            c.barcode([0.035, 0.76, 0.32, 0.20]);
            c.text(
                "ชื่อสินค้า",
                "product.name",
                [0.41, 0.03, 0.56, 0.16],
                0.075,
                ink,
            );
            if promo {
                c.old_price([0.44, 0.22, 0.50, 0.12], 0.055);
            }
            c.text(
                "ราคาขาย",
                "price.value",
                [0.40, 0.35, 0.57, 0.49],
                0.27,
                if promo { red } else { ink },
            );
            c.caption("฿/Pcs.", [0.77, 0.83, 0.20, 0.13], 0.07, ink);
        }
        "ADA-200x142" | "ADA-200x95" | "ADA-200x95-SP" | "ADA-100x142" => {
            if promo {
                bg = "#f2d263";
            }
            c.rect(
                "พื้นหัวป้าย",
                [0.01, 0.01, 0.27, 0.23],
                if promo { red } else { green },
                8.,
            );
            c.caption(
                if !promo {
                    "ราคาปกติ"
                } else if code.ends_with("-SP") {
                    "สินค้าพิเศษ"
                } else {
                    "ลดกระหน่ำ"
                },
                [0.015, 0.045, 0.26, 0.15],
                0.044,
                "#ffffff",
            );
            if promo {
                c.text(
                    "โปรโมชั่น",
                    "promotion.text",
                    [0.30, 0.02, 0.67, 0.12],
                    0.046,
                    ink,
                );
                if code == "ADA-200x142" {
                    c.text(
                        "ข้อความเพิ่มเติม",
                        "promotion.free_text",
                        [0.30, 0.14, 0.67, 0.08],
                        0.032,
                        ink,
                    );
                }
                c.old_price([0.02, 0.66, 0.30, 0.13], 0.046);
            }
            c.text(
                "ราคาขาย",
                "price.value",
                [0.21, 0.20, 0.70, 0.35],
                0.19,
                ink,
            );
            c.caption("บาท", [0.75, 0.55, 0.20, 0.09], 0.047, ink);
            c.text(
                "ชื่อสินค้า",
                "product.name",
                [0.02, 0.79, 0.55, 0.10],
                0.045,
                ink,
            );
            c.text(
                "ชื่อรอง",
                "product.name_oth",
                [0.02, 0.89, 0.55, 0.08],
                0.031,
                ink,
            );
            c.text("ช่วงราคา", period, [0.58, 0.67, 0.40, 0.10], 0.031, ink);
            c.barcode([0.60, 0.79, 0.36, 0.17]);
        }
        _ => {
            if promo {
                bg = "#fff3c4";
                c.old_price([0.02, 0.02, 0.39, 0.15], 0.076);
            }
            c.text(
                "ราคาขาย",
                "price.value",
                [0.01, 0.19, 0.40, 0.40],
                0.13,
                ink,
            );
            c.caption("บาท", [0.03, 0.59, 0.36, 0.17], 0.072, ink);
            c.text(
                "ชื่อสินค้า",
                "product.name",
                [0.42, 0.02, 0.56, 0.19],
                0.077,
                ink,
            );
            c.text(
                "ชื่อรอง",
                "product.name_oth",
                [0.42, 0.21, 0.56, 0.14],
                0.061,
                ink,
            );
            c.barcode([0.44, 0.38, 0.51, 0.37]);
            c.text(
                "หน่วย",
                "tag.selling_unit",
                [0.43, 0.77, 0.23, 0.16],
                0.061,
                ink,
            );
            c.text("วันที่", period, [0.67, 0.77, 0.31, 0.16], 0.053, ink);
        }
    }
    json!({"customDesign":true,"designMode":"full","lbsCode":code,"name":name,"width":width,"height":height,
        "priType":if promo {"2"}else{"1"},"backgroundColor":bg,"borderColor":border,"borderWidth":stroke,"borderRadius":0,"elements":c.elements})
}

pub async fn install(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let sizes:Vec<(String,String,f64,f64)>=sqlx::query_as("SELECT FTLbsCode,coalesce(FTLbsName,FTLbsCode),FCLbsWidth,FCLbsHeight FROM TCNMLabelSize s WHERE FTLbsCode IN ('UFM-PRO-A5','UFM-SALE-8','UFM-NOR-16','UFM-TIMESALE','ADA-100x38','ADA-50x38','ADA-200x142','ADA-200x95','ADA-200x95-SP','ADA-100x142') OR NOT EXISTS (SELECT 1 FROM TCNMLabelTmpl t WHERE t.FTLbsCode=s.FTLbsCode)").fetch_all(&mut *tx).await?;
    for (code, name, w, h) in sizes {
        for (price, suffix) in [("1", "NOR"), ("2", "PMT")] {
            let id = format!("{code}-STD-{suffix}");
            let layout = standard_layout(&code, &name, w, h, price == "2");
            sqlx::query("INSERT OR IGNORE INTO TCNMLabelTmpl(FTLtmCode,FTLbsCode,FTLtmPriType,FTLtmName,FTLtmLayout,FTLtmStaUse,FTLtmMode,FTCreateBy) VALUES(?,?,?,?,?,'1','full','SYSTEM')")
                .bind(&id).bind(&code).bind(price).bind(&name).bind(layout.to_string()).execute(&mut *tx).await?;
            let col = if price == "2" {
                "FTLbsTmplPmt"
            } else {
                "FTLbsTmplNor"
            };
            sqlx::query(&format!(
                "UPDATE TCNMLabelSize SET {col}=? WHERE FTLbsCode=? AND coalesce({col},'')=''"
            ))
            .bind(&id)
            .bind(&code)
            .execute(&mut *tx)
            .await?;
        }
    }
    // Existing scheduled profiles keep their printer, quantities and event settings.
    // Only replace a legacy selector when its independent standard template exists.
    let rules: Vec<(String, String)> =
        sqlx::query_as("SELECT id,config FROM ABPrintRule WHERE instr(config,'legacy:')>0")
            .fetch_all(&mut *tx)
            .await?;
    for (id, raw) in rules {
        let mut rule: Value =
            serde_json::from_str(&raw).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let suffix = if rule["source"] == "PROMO_START" {
            "PMT"
        } else {
            "NOR"
        };
        let mut changed = false;
        let paths = std::iter::once(String::new()).chain(
            (0..rule["targets"].as_array().map_or(0, Vec::len)).map(|i| format!("/targets/{i}")),
        );
        for path in paths {
            let target = rule.pointer_mut(&path).unwrap();
            if let Some(size) = target["size"].as_str() {
                if target["template"] == format!("legacy:{size}") {
                    let code = format!("{size}-STD-{suffix}");
                    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM TCNMLabelTmpl WHERE FTLbsCode=? AND FTLtmCode=? AND FTLtmStaUse='1')")
                        .bind(size).bind(&code).fetch_one(&mut *tx).await?;
                    if exists {
                        target["template"] = json!(code);
                        changed = true;
                    }
                }
            }
        }
        if changed {
            sqlx::query("UPDATE ABPrintRule SET config=? WHERE id=?")
                .bind(rule.to_string())
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await
}
