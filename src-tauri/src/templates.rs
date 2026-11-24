use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WhenCondition {
    pub path: String,
    pub eq: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateElement {
    #[serde(rename = "type")]
    pub el_type: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub preprinted_paper: Option<bool>,
    pub src: Option<String>,
    #[serde(rename="iconName")]
    pub icon_name: Option<String>,
    pub field: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub align: Option<String>,
    pub strike: Option<bool>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub fill: Option<String>,
    #[serde(rename = "lineWidth")]
    pub line_width: Option<f64>,
    pub symbology: Option<String>,
    #[serde(rename = "showText")]
    pub show_text: Option<bool>,
    pub format: Option<String>,
    pub when: Option<WhenCondition>,
    pub font: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LabelTemplate {
    pub code: String,
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub unit: String,
    pub orientation: Option<String>,
    pub elements: Vec<TemplateElement>,
}

pub fn get_templates() -> Vec<LabelTemplate> {
    let json = r##"[
  {
    "code": "SH-100x38-NOR",
    "name": "ป้ายราคาใหญ่ 10x3.8 (ปกติ)",
    "width": 100, "height": 38, "unit": "mm", "orientation": "landscape",
    "elements": [
      { "type": "price", "field": "price.value", "x": 1, "y": 6, "width": 34, "height": 20, "fontSize": 30, "align": "center", "suffix": " บาท" },
      { "type": "text", "field": "product.name", "x": 37, "y": 1, "width": 62, "height": 6, "fontSize": 11 },
      { "type": "text", "field": "product.name_oth", "x": 37, "y": 7, "width": 62, "height": 5, "fontSize": 8 },
      { "type": "text", "field": "product.dim", "x": 37, "y": 12, "width": 62, "height": 4, "fontSize": 8 },
      { "type": "text", "field": "tag.pri_per_unit", "x": 37, "y": 16, "width": 62, "height": 4, "fontSize": 6, "color": "#555555" },
      { "type": "barcode", "field": "product.barcode", "x": 37, "y": 20, "width": 46, "height": 13, "symbology": "EAN13", "showText": true },
      { "type": "text", "field": "tag.sub_dept", "x": 37, "y": 34, "width": 26, "height": 4, "fontSize": 6 },
      { "type": "text", "field": "tag.phasing", "x": 63, "y": 34, "width": 10, "height": 4, "fontSize": 6 },
      { "type": "text", "field": "tag.selling_unit", "x": 74, "y": 34, "width": 14, "height": 4, "fontSize": 6 },
      { "type": "date", "format": "%d/%m/%y", "x": 88, "y": 34, "width": 11, "height": 4, "fontSize": 6 }
    ]
  },
  {
    "code": "SH-100x38-PMT",
    "name": "ป้ายราคาใหญ่ 10x3.8 (โปรโมชัน)",
    "width": 100, "height": 38, "unit": "mm", "orientation": "landscape",
    "elements": [
      { "type": "rect", "x": 0, "y": 0, "width": 100, "height": 38, "fill": "#FFF3C4" },
      { "type": "price", "field": "price.old_value", "x": 1, "y": 0.5, "width": 20, "height": 5, "fontSize": 7, "strike": true, "prefix": "ปกติ ", "when": { "path": "promotion.is_discount", "eq": true } },
      { "type": "static", "text": "ลดราคา", "x": 1, "y": 30, "width": 14, "height": 5, "fontSize": 9, "color": "#CC0000", "when": { "path": "promotion.is_discount", "eq": true } },
      { "type": "text", "field": "promotion.text", "x": 30, "y": 0.5, "width": 40, "height": 7, "fontSize": 14, "color": "#CC0000", "when": { "path": "promotion.is_gift", "eq": true } },
      { "type": "price", "field": "price.value", "x": 1, "y": 6, "width": 34, "height": 20, "fontSize": 30, "align": "center", "suffix": " บาท" },
      { "type": "text", "field": "product.name", "x": 37, "y": 8, "width": 62, "height": 6, "fontSize": 11 },
      { "type": "text", "field": "product.dim", "x": 37, "y": 14, "width": 62, "height": 4, "fontSize": 8 },
      { "type": "barcode", "field": "product.barcode", "x": 37, "y": 18, "width": 46, "height": 12, "symbology": "EAN13" },
      { "type": "text", "field": "promotion.free_text", "x": 30, "y": 30, "width": 68, "height": 4, "fontSize": 6, "when": { "path": "promotion.is_gift", "eq": true } },
      { "type": "text", "field": "promotion.interval", "x": 37, "y": 34, "width": 50, "height": 4, "fontSize": 6 },
      { "type": "date", "format": "%d/%m/%y", "x": 88, "y": 34, "width": 11, "height": 4, "fontSize": 6 }
    ]
  },
  {
    "code": "BC-25x32-NOR",
    "name": "สติ๊กเกอร์บาร์โค้ด 25x32",
    "width": 25, "height": 32, "unit": "mm", "orientation": "portrait",
    "elements": [
      { "type": "text", "field": "product.name", "x": 1, "y": 1, "width": 23, "height": 8, "fontSize": 6 },
      { "type": "barcode", "field": "product.barcode", "x": 2, "y": 10, "width": 21, "height": 13, "symbology": "EAN13" },
      { "type": "text", "field": "product.code", "x": 1, "y": 24, "width": 23, "height": 4, "fontSize": 5 },
      { "type": "price", "field": "price.value", "x": 1, "y": 25, "width": 23, "height": 7, "fontSize": 12, "align": "center", "suffix": " บาท" }
    ]
  },
  {
    "code": "BC-25x32-NR",
    "name": "สติ๊กเกอร์ NOT FOR SALE",
    "width": 25, "height": 32, "unit": "mm", "orientation": "portrait",
    "elements": [
      { "type": "static", "text": "NOT FOR SALE", "x": 1, "y": 1, "width": 23, "height": 2.5, "fontSize": 6, "color": "#CC0000", "align": "center" },
      { "type": "static", "text": "NO REFUND", "x": 1, "y": 3.5, "width": 23, "height": 2.5, "fontSize": 6, "color": "#CC0000", "align": "center" },
      { "type": "text", "field": "product.name", "x": 1, "y": 6, "width": 23, "height": 8, "fontSize": 6 },
      { "type": "barcode", "field": "product.barcode", "x": 2, "y": 14, "width": 21, "height": 13, "symbology": "EAN13" },
      { "type": "text", "field": "product.code", "x": 1, "y": 27, "width": 23, "height": 4, "fontSize": 5 }
    ]
  },
  {
    "code": "BC-QR-30x30-NOR",
    "name": "QR Code สินค้า 30x30",
    "width": 30, "height": 30, "unit": "mm", "orientation": "portrait",
    "elements": [
      { "type": "qrcode", "field": "product.url", "x": 5, "y": 2, "width": 20, "height": 20 },
      { "type": "text", "field": "product.code", "x": 1, "y": 24, "width": 28, "height": 4, "fontSize": 6, "align": "center" }
    ]
  },
  {
    "code": "SH-50x38-NOR",
    "name": "ป้ายราคาเล็ก 5x3.8 (ปกติ)",
    "width": 50, "height": 38, "unit": "mm", "orientation": "landscape",
    "elements": [
      { "type": "price", "field": "price.value", "x": 1, "y": 5, "width": 20, "height": 14, "fontSize": 18, "align": "center", "suffix": " ฿" },
      { "type": "text", "field": "product.name", "x": 22, "y": 1, "width": 27, "height": 6, "fontSize": 7 },
      { "type": "text", "field": "product.dim", "x": 22, "y": 7, "width": 27, "height": 4, "fontSize": 6 },
      { "type": "barcode", "field": "product.barcode", "x": 22, "y": 11, "width": 25, "height": 12, "symbology": "EAN13" },
      { "type": "text", "field": "tag.sub_dept", "x": 22, "y": 33, "width": 15, "height": 4, "fontSize": 5 },
      { "type": "date", "format": "%d/%m/%y", "x": 38, "y": 33, "width": 11, "height": 4, "fontSize": 5 }
    ]
  },
  {
    "code": "SH-50x38-PMT",
    "name": "ป้ายราคาเล็ก 5x3.8 (โปรโมชัน)",
    "width": 50, "height": 38, "unit": "mm", "orientation": "landscape",
    "elements": [
      { "type": "rect", "x": 0, "y": 0, "width": 50, "height": 38, "fill": "#FFF3C4" },
      { "type": "price", "field": "price.old_value", "x": 1, "y": 0.5, "width": 14, "height": 4, "fontSize": 5, "strike": true, "when": { "path": "promotion.is_discount", "eq": true } },
      { "type": "static", "text": "ลดราคา", "x": 1, "y": 30, "width": 10, "height": 5, "fontSize": 7, "color": "#CC0000", "when": { "path": "promotion.is_discount", "eq": true } },
      { "type": "price", "field": "price.value", "x": 1, "y": 5, "width": 20, "height": 14, "fontSize": 18, "align": "center", "suffix": " ฿" },
      { "type": "text", "field": "product.name", "x": 22, "y": 6, "width": 27, "height": 6, "fontSize": 7 },
      { "type": "text", "field": "product.dim", "x": 22, "y": 12, "width": 27, "height": 4, "fontSize": 6 },
      { "type": "barcode", "field": "product.barcode", "x": 22, "y": 16, "width": 25, "height": 12, "symbology": "EAN13" },
      { "type": "text", "field": "promotion.interval", "x": 22, "y": 33, "width": 25, "height": 4, "fontSize": 5 }
    ]
  },
  {
    "code": "BC-32x24-NOR",
    "name": "Ticket Price 32x24",
    "width": 32, "height": 24, "unit": "mm", "orientation": "portrait",
    "elements": [
      { "type": "text", "field": "product.name", "x": 1, "y": 1, "width": 30, "height": 6, "fontSize": 6 },
      { "type": "barcode", "field": "product.barcode", "x": 2, "y": 7, "width": 28, "height": 11, "symbology": "EAN13" },
      { "type": "price", "field": "price.value", "x": 1, "y": 18, "width": 30, "height": 6, "fontSize": 12, "align": "center", "suffix": " บาท" }
    ]
  }
]"##;
    serde_json::from_str(json).expect("valid template JSON")
}
