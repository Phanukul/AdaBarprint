#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMBranch {
    pub FTBchCode: String,
    pub FTBchName: Option<String>,
    pub FTBchStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPdt {
    pub FTPdtCode: String,
    pub FTPdtName: String,
    pub FTPdtNameOth: Option<String>,
    pub FTPdtSku: Option<String>,
    pub FTPdtBrand: Option<String>,
    pub FTPdtCgyName: Option<String>,
    pub FTPdtUnit: Option<String>,
    pub FTPdtDim: Option<String>,
    pub FTPdtStkCode: Option<String>,
    pub FTPdtSubDept: Option<String>,
    pub FTPdtRepleType: Option<String>,
    pub FTPdtPhasing: Option<String>,
    pub FTPdtRegNo: Option<String>,
    pub FTPdtStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPdtBar {
    pub FTPdbBarCode: String,
    pub FTPdtCode: String,
    pub FTPdbSymbology: String,
    pub FTPdbStaPri: String,
    pub FTPdbStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPdtPri {
    pub FTPdtCode: String,
    pub FTBchCode: String,
    pub FTPpcPriType: String,
    pub FCPpcPrice: f64,
    pub FCPpcUnitPrice: Option<f64>,
    pub FDPpcDStart: Option<String>,
    pub FDPpcDStop: Option<String>,
    pub FTPpcStaUse: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPmtHD {
    pub FTPmhCode: String,
    pub FTPmhDocNo: Option<String>,
    pub FTPmhName: Option<String>,
    pub FTPmhNameSlip: Option<String>,
    pub FTPmhRmk: Option<String>,
    pub FDPmhDStart: Option<String>,
    pub FDPmhDStop: Option<String>,
    pub FTPmhStaApv: String,
    pub FTPmhStaAll: String,
    pub FTPmhStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPmtDT {
    pub FTPmhCode: String,
    pub FNPmdSeq: i64,
    pub FTPdtCode: Option<String>,
    pub FTBarCode: Option<String>,
    pub FCPmdPricePmt: Option<f64>,
    pub FNPmdGetCond: Option<i64>,
    pub FCPmdGetValue: Option<f64>,
    pub FTPmdStaUse: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMLabelSize {
    pub FTLbsCode: String,
    pub FTLbsGroup: String,
    pub FTLbsName: String,
    pub FCLbsWidth: f64,
    pub FCLbsHeight: f64,
    pub FNLbsCol: i64,
    pub FNLbsRow: i64,
    pub FCLbsMagTop: Option<f64>,
    pub FCLbsMagLeft: Option<f64>,
    pub FCLbsMagBottom: Option<f64>,
    pub FCLbsMagRight: Option<f64>,
    pub FCLbsGapH: Option<f64>,
    pub FCLbsGapV: Option<f64>,
    pub FNLbsQtyPageNor: Option<i64>,
    pub FNLbsQtyPagePmt: Option<i64>,
    pub FTLbsTmplNor: Option<String>,
    pub FTLbsTmplPmt: Option<String>,
    pub FTLbsHhtCode: Option<String>,
    pub FTLbsStaUse: String,
    pub FTLbsPaperSize: Option<String>,
    pub FCLbsPaperWidth: Option<f64>,
    pub FCLbsPaperHeight: Option<f64>,
    pub FCLbsOffsetX: Option<f64>,
    pub FCLbsOffsetY: Option<f64>,
    pub FTPprCode: Option<String>,
    pub FTLbsStdTmplNor: Option<String>,
    pub FTLbsStdTmplPmt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNSSysOption {
    pub FTSopCode: String,
    pub FTSopApp: Option<String>,
    pub FTSopName: Option<String>,
    pub FTSopValue: Option<String>,
    pub FTSopDesc: Option<String>,
    pub FNSopSeq: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMLabelTmpl {
    pub FTLtmCode: String,
    pub FTLbsCode: String,
    pub FTLtmPriType: String,
    pub FTLtmName: String,
    pub FTLtmLayout: Option<String>,
    pub FTLtmStaUse: String,
    pub FTLtmMode: Option<String>,
    pub FTPprCode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPaperSize {
    pub FTPpsCode: String,
    pub FTPpsName: String,
    pub FCPpsWidth: f64,
    pub FCPpsHeight: f64,
    pub FTPpsWinKind: Option<String>,
    pub FTPpsStaCustom: String,
    pub FNPpsSeq: Option<i64>,
    pub FTPpsStaUse: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPreprint {
    pub FTPprCode: String,
    pub FTPprName: String,
    pub FTPprFileName: Option<String>,
    pub FTPpsCode: String,
    pub FCPprWidth: f64,
    pub FCPprHeight: f64,
    pub FNPprCol: i64,
    pub FNPprRow: i64,
    pub FCPprLabelW: Option<f64>,
    pub FCPprLabelH: Option<f64>,
    pub FCPprMagTop: Option<f64>,
    pub FCPprMagLeft: Option<f64>,
    pub FCPprMagBottom: Option<f64>,
    pub FCPprMagRight: Option<f64>,
    pub FCPprGapH: Option<f64>,
    pub FCPprGapV: Option<f64>,
    pub FNPprPageNo: Option<i64>,
    pub FTPprScope: String,
    pub FCPprOpacity: Option<f64>,
    pub FTPprSheetImg: Option<String>,
    pub FTPprCellImg: Option<String>,
    pub FTPprCropJson: Option<String>,
    pub FTPprStaUse: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMLabelField {
    pub FTLbfCode: String,
    pub FTLbfName: String,
    pub FTLbfType: String,
    pub FTLbfSample: Option<String>,
    pub FCLbfDefW: Option<f64>,
    pub FCLbfDefH: Option<f64>,
    pub FNLbfDefFont: Option<i64>,
    pub FTLbfDefWeight: Option<String>,
    pub FTLbfDefColor: Option<String>,
    pub FNLbfSeq: Option<i64>,
    pub FTLbfStaAdv: String,
    pub FTLbfStaUse: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPrinterServer {
    pub FTPsvCode: String,
    pub FTPsvName: Option<String>,
    pub FTPsvRmk: Option<String>,
    pub FTPsvStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMLabelPrinter {
    pub FTLbsCode: String,
    pub FTPsvCode: String,
    pub FTLbpPriType: String,
    pub FTLbpPrinter: Option<String>,
    pub FTLbpTray: Option<String>,
    pub FTLbpProtocol: String,
    pub FTLbpStaUse: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNMPdtLabel {
    pub FTPdtCode: String,
    pub FTBarCode: String,
    pub FTLbsCode: String,
    pub FNPdlQty: i64,
    pub FTPdlSource: String,
    pub FTPdlStaDef: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNTLabelJob {
    pub FTLjbJobID: String,
    pub FTLjbBchCode: String,
    pub FTLjbSource: String,
    pub FTLjbPriType: String,
    pub FDLjbFrom: Option<String>,
    pub FDLjbTo: Option<String>,
    pub FTLjbStaVfy: String,
    pub FTLjbLbsOvr: Option<String>,
    pub FTLjbStatus: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNTLabelJobDt {
    pub FTLjbJobID: String,
    pub FNLjdSeq: i64,
    pub FTLjdBarCode: Option<String>,
    pub FTLjdPdtCode: Option<String>,
    pub FTLjdName: Option<String>,
    pub FTLjdNameOth: Option<String>,
    pub FTLjdVolume: Option<String>,
    pub FTLjdSubDept: Option<String>,
    pub FTLjdReple: Option<String>,
    pub FDLjdPriDate: Option<String>,
    pub FTLjdPmtIntv: Option<String>,
    pub FTLjdPriStat: Option<String>,
    pub FTLjdSellUnit: Option<String>,
    pub FCLjdSetPrice: Option<f64>,
    pub FCLjdSalePrice: Option<f64>,
    pub FTLjdPhasing: Option<String>,
    pub FTLjdPerUnit: Option<String>,
    pub FTLjdCapFree: Option<String>,
    pub FTLjdCapName: Option<String>,
    pub FTLjdLabSize: Option<String>,
    pub FNLjdLabQty: i64,
    pub FNLjdQtyOrg: Option<i64>,
    pub FCLjdPmtCond: Option<f64>,
    pub FCLjdPmtVal: Option<f64>,
    pub FDLjdPmtFrom: Option<String>,
    pub FDLjdPmtTo: Option<String>,
    pub FNLjdBarSeq: Option<i64>,
    pub FNLjdPage: Option<i64>,
    pub FTLjdPriTyp: Option<String>,
    pub FTLjdTmpl: Option<String>,
    pub FTLjdStaSel: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TCNTLabelLog {
    pub FTLlgLogID: String,
    pub FTLjbJobID: String,
    pub FTLlgLbsCode: String,
    pub FTLlgPriTyp: String,
    pub FTLlgPsvCode: String,
    pub FTLlgPrinter: Option<String>,
    pub FNLlgQtyLabel: i64,
    pub FNLlgQtyPage: i64,
    pub FDLlgPriOn: Option<String>,
    pub FTLlgPriBy: Option<String>,
    pub FTLlgStatus: String,
    pub FDCreateOn: Option<String>,
    pub FTCreateBy: Option<String>,
    pub FDLastUpdOn: Option<String>,
    pub FTLastUpdBy: Option<String>,
}
