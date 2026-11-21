-- Schema only; generated from StoreFront DB + ensureAccdSyncSchema. No customer rows.
CREATE TABLE IF NOT EXISTS [TCNMAddressTmp_L] (
	[FNLngID]	integer NOT NULL,
	[FTAddGrpType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTAddRefCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNAddSeqNo]	integer NOT NULL,
	[FTAddRefNo]	varchar(20) COLLATE NOCASE,
	[FTAddName]	varchar(200) COLLATE NOCASE,
	[FTAddTaxNo]	varchar(20) COLLATE NOCASE,
	[FTAddRmk]	varchar(200) COLLATE NOCASE,
	[FTAddCountry]	varchar(100) COLLATE NOCASE,
	[FTAddVersion]	varchar(1) COLLATE NOCASE,
	[FTAddV1No]	varchar(30) COLLATE NOCASE,
	[FTAddV1Soi]	varchar(30) COLLATE NOCASE,
	[FTAddV1Village]	varchar(70) COLLATE NOCASE,
	[FTAddV1Road]	varchar(30) COLLATE NOCASE,
	[FTAddV1SubDist]	varchar(30) COLLATE NOCASE,
	[FTAddV1DstCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PvnCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PostCode]	varchar(5) COLLATE NOCASE,
	[FTAddTel]	varchar(50) COLLATE NOCASE,
	[FTAddFax]	varchar(50) COLLATE NOCASE,
	[FTAddV2Desc1]	varchar(255) COLLATE NOCASE,
	[FTAddV2Desc2]	varchar(255) COLLATE NOCASE,
	[FTAddWebsite]	varchar(200) COLLATE NOCASE,
	[FTAddLongitude]	varchar(50) COLLATE NOCASE,
	[FTAddLatitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMAdMsg] (
	[FTAdvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTAdvType]	varchar(1) COLLATE NOCASE,
	[FNAdvSeqNo]	integer,
	[FTAdvStaUse]	varchar(1) COLLATE NOCASE,
	[FDAdvStart]	datetime,
	[FDAdvStop]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTAdvCode])

);
CREATE TABLE IF NOT EXISTS [TCNMAdMsg_L] (
	[FTAdvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAdvMsg]	varchar(250) COLLATE NOCASE,
	[FTAdvRmk]	varchar(250) COLLATE NOCASE,
	[FTAdvName]	varchar(250) COLLATE NOCASE,
    PRIMARY KEY ([FTAdvCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMAdMsgTmp] (
	[FTAdvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTAdvType]	varchar(1) COLLATE NOCASE,
	[FNAdvSeqNo]	integer,
	[FTAdvStaUse]	varchar(1) COLLATE NOCASE,
	[FDAdvStart]	datetime,
	[FDAdvStop]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMAdMsgTmp_L] (
	[FTAdvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAdvMsg]	varchar(250) COLLATE NOCASE,
	[FTAdvRmk]	varchar(250) COLLATE NOCASE,
	[FTAdvName]	varchar(250) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMAgency_L] (
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAgnName]	varchar(200) COLLATE NOCASE,
	[FTAgnRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTAgnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMAgencyGrp] (
	[FTAggCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTAggCode])

);
CREATE TABLE IF NOT EXISTS [TCNMAgencyGrp_L] (
	[FTAggCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAggName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTAggCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMAgencySpc] (
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTCmpVatInOrEx]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FTAgnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMAgencyType] (
	[FTAtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTAtyCode])

);
CREATE TABLE IF NOT EXISTS [TCNMAgencyType_L] (
	[FTAtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAtyName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTAtyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMAppModule] (
	[FTAppCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTAppModule]	varchar(2) NOT NULL COLLATE NOCASE,
	[FNAppSeqNo]	integer NOT NULL,
	[FTAppSignType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTAppCode], [FTAppModule], [FNAppSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNMArea] (
	[FTAreCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTAreCode])

);
CREATE TABLE IF NOT EXISTS [TCNMArea_L] (
	[FTAreCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAreName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTAreCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMAutoFmt] (
	[FTSatTblName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatFedCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatStaDocType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTSatColType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNSatColSeq]	integer,
	[FTSatStaReset]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTSatTblName], [FTSatFedCode], [FTSatStaDocType], [FTSatColType])

);
CREATE TABLE IF NOT EXISTS [TCNMBranch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchType]	varchar(1) COLLATE NOCASE,
	[FTBchPriority]	varchar(1) COLLATE NOCASE,
	[FTBchRegNo]	varchar(30) COLLATE NOCASE,
	[FTBchRefID]	varchar(30) COLLATE NOCASE,
	[FDBchStart]	datetime,
	[FDBchStop]	datetime,
	[FDBchSaleStart]	datetime,
	[FDBchSaleStop]	datetime,
	[FTBchStaHQ]	varchar(1) COLLATE NOCASE,
	[FTBchStaActive]	varchar(1) COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNBchDefLang]	integer,
	[FTBchUriSrvMQ]	varchar(200) COLLATE NOCASE,
	[FTBchUriSrvSG]	varchar(200) COLLATE NOCASE,
	[FTMerCode]	varchar(10) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode])

);
CREATE TABLE IF NOT EXISTS [TCNMBranch_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBchName]	varchar(100) COLLATE NOCASE,
	[FTBchRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMBranchTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchType]	varchar(1) COLLATE NOCASE,
	[FTBchPriority]	varchar(1) COLLATE NOCASE,
	[FTBchRegNo]	varchar(30) COLLATE NOCASE,
	[FTBchRefID]	varchar(30) COLLATE NOCASE,
	[FDBchStart]	datetime,
	[FDBchStop]	datetime,
	[FDBchSaleStart]	datetime,
	[FDBchSaleStop]	datetime,
	[FTBchStaHQ]	varchar(1) COLLATE NOCASE,
	[FTBchStaActive]	varchar(1) COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNBchDefLang]	integer,
	[FTBchUriSrvMQ]	varchar(200) COLLATE NOCASE,
	[FTBchUriSrvSG]	varchar(200) COLLATE NOCASE,
	[FTMerCode]	varchar(10) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMBranchTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBchName]	varchar(100) COLLATE NOCASE,
	[FTBchRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMComp] (
	[FTCmpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCmpTel]	varchar(50) COLLATE NOCASE,
	[FTCmpFax]	varchar(50) COLLATE NOCASE,
	[FTBchcode]	varchar(5) COLLATE NOCASE,
	[FTCmpWhsInOrEx]	varchar(1) COLLATE NOCASE,
	[FTCmpRetInOrEx]	varchar(1) COLLATE NOCASE,
	[FTCmpEmail]	varchar(50) COLLATE NOCASE,
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTVatCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCmpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMComp_L] (
	[FTCmpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCmpName]	varchar(200) COLLATE NOCASE,
	[FTCmpShop]	varchar(200) COLLATE NOCASE,
	[FTCmpDirector]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTCmpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCompTmp] (
	[FTCmpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCmpTel]	varchar(50) COLLATE NOCASE,
	[FTCmpFax]	varchar(50) COLLATE NOCASE,
	[FTBchcode]	varchar(5) COLLATE NOCASE,
	[FTCmpWhsInOrEx]	varchar(1) COLLATE NOCASE,
	[FTCmpRetInOrEx]	varchar(1) COLLATE NOCASE,
	[FTCmpEmail]	varchar(50) COLLATE NOCASE,
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTVatCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCompTmp_L] (
	[FTCmpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCmpName]	varchar(200) COLLATE NOCASE,
	[FTCmpShop]	varchar(200) COLLATE NOCASE,
	[FTCmpDirector]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCourieLogin] (
	[FTCryCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManCardID]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDManPwdStart]	datetime NOT NULL,
	[FDManPwdExpired]	datetime,
	[FTManLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTManLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTManRmk]	varchar(50) COLLATE NOCASE,
	[FTManStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTCryCode], [FTManCardID], [FTManLogType], [FDManPwdStart])

);
CREATE TABLE IF NOT EXISTS [TCNMCourieMan] (
	[FTCryCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManCardID]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManTel]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTManCardEmp]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManSex]	varchar(1) COLLATE NOCASE,
	[FDManDob]	datetime,
	[FTManStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTCryCode], [FTManCardID])

);
CREATE TABLE IF NOT EXISTS [TCNMCourieMan_L] (
	[FTCryCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTManCardID]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTManName]	varchar(200) COLLATE NOCASE,
	[FTCryRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTCryCode], [FTManCardID], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCourier] (
	[FTCryCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCryCrTerm]	integer,
	[FCCryCrLimit]	real,
	[FTCryCardID]	varchar(20) COLLATE NOCASE,
	[FTCryTaxNo]	varchar(20) COLLATE NOCASE,
	[FTCryTel]	varchar(50) COLLATE NOCASE,
	[FTCryFax]	varchar(50) COLLATE NOCASE,
	[FTCryEmail]	varchar(100) COLLATE NOCASE,
	[FTCrySex]	varchar(1) COLLATE NOCASE,
	[FDCryDob]	datetime,
	[FTCgpCode]	varchar(5) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTCryBusiness]	varchar(1) COLLATE NOCASE,
	[FTCryBchHQ]	varchar(1) COLLATE NOCASE,
	[FTCryBchCode]	varchar(50) COLLATE NOCASE,
	[FTCryDelimeterQR]	varchar(3) COLLATE NOCASE,
	[FTCryStaActive]	varchar(1) COLLATE NOCASE,
	[FTCryLoginType]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCryCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCourier_L] (
	[FTCryCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCryName]	varchar(200) COLLATE NOCASE,
	[FTCryNameOth]	varchar(200) COLLATE NOCASE,
	[FTCryRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTCryCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCourierGrp] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCourierGrp_L] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCgpName]	varchar(100) COLLATE NOCASE,
	[FTCgpRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCourierType] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCourierType_L] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCtyName]	varchar(100) COLLATE NOCASE,
	[FTCtyRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCrdLogin] (
	[FTCrdCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCrdLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDCrdPwdStart]	datetime NOT NULL,
	[FDCrdPwdExpired]	datetime,
	[FTCrdLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCrdLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCrdRmk]	varchar(50) COLLATE NOCASE,
	[FTCrdStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FTCrdLogType], [FDCrdPwdStart])

);
CREATE TABLE IF NOT EXISTS [TCNMCst] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTCstCardID]	varchar(20) COLLATE NOCASE,
	[FTCstTaxNo]	varchar(20) COLLATE NOCASE,
	[FTCstTel]	varchar(50) COLLATE NOCASE,
	[FTCstFax]	varchar(50) COLLATE NOCASE,
	[FTCstEmail]	varchar(100) COLLATE NOCASE,
	[FTCstSex]	varchar(1) COLLATE NOCASE,
	[FDCstDob]	datetime,
	[FTCgpCode]	varchar(5) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTClvCode]	varchar(5) COLLATE NOCASE,
	[FTPplCodeRet]	varchar(20) COLLATE NOCASE,
	[FTPplCodeWhs]	varchar(20) COLLATE NOCASE,
	[FTPplCodenNet]	varchar(20) COLLATE NOCASE,
	[FTPmgCode]	varchar(20) COLLATE NOCASE,
	[FTOcpCode]	varchar(5) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTCstDiscWhs]	varchar(50) COLLATE NOCASE,
	[FTCstDiscRet]	varchar(50) COLLATE NOCASE,
	[FTCstBusiness]	varchar(1) COLLATE NOCASE,
	[FTCstBchHQ]	varchar(1) COLLATE NOCASE,
	[FTCstBchCode]	varchar(50) COLLATE NOCASE,
	[FTCstStaActive]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwPosCalSo]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTCstBizRole TEXT,
    PRIMARY KEY ([FTCstCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCst_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCstName]	varchar(200) COLLATE NOCASE,
	[FTCstNameOth]	varchar(200) COLLATE NOCASE,
	[FTCstRmk]	varchar(200) COLLATE NOCASE, FTAgnCode TEXT, FTCstPayRmk TEXT, FTCstBillRmk TEXT, FTCstViaRmk TEXT,
    PRIMARY KEY ([FTCstCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCstAddress_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAddGrpType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNAddSeqNo]	integer NOT NULL,
	[FTAddRefNo]	varchar(20) COLLATE NOCASE,
	[FTAddName]	varchar(200) COLLATE NOCASE,
	[FTAddRmk]	varchar(200) COLLATE NOCASE,
	[FTAddCountry]	varchar(100) COLLATE NOCASE,
	[FTAreCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTAddVersion]	varchar(1) COLLATE NOCASE,
	[FTAddV1No]	varchar(30) COLLATE NOCASE,
	[FTAddV1Soi]	varchar(30) COLLATE NOCASE,
	[FTAddV1Village]	varchar(70) COLLATE NOCASE,
	[FTAddV1Road]	varchar(30) COLLATE NOCASE,
	[FTAddV1SubDist]	varchar(30) COLLATE NOCASE,
	[FTAddV1DstCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PvnCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PostCode]	varchar(5) COLLATE NOCASE,
	[FTAddTel]	varchar(50) COLLATE NOCASE,
	[FTAddFax]	varchar(50) COLLATE NOCASE,
	[FTAddV2Desc1]	varchar(255) COLLATE NOCASE,
	[FTAddV2Desc2]	varchar(255) COLLATE NOCASE,
	[FTAddWebsite]	varchar(200) COLLATE NOCASE,
	[FTAddLongitude]	varchar(50) COLLATE NOCASE,
	[FTAddLatitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTAgnCode TEXT, FTAddAttn TEXT, FTAddContactTo TEXT, FTAddContactTel TEXT, FTAddV1Moo TEXT, FTAddV1Building TEXT, FTAddV1Floor TEXT, FTAddV1Room TEXT, FTAddV1DstName TEXT, FTAddV1PvnName TEXT, FTAddV1CtrName TEXT, FTAddV2Desc3 TEXT, FTAddFullText TEXT,
    PRIMARY KEY ([FTCstCode], [FNLngID], [FTAddGrpType], [FNAddSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNMCstAddressTmp_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAddGrpType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNAddSeqNo]	integer NOT NULL,
	[FTAddRefNo]	varchar(20) COLLATE NOCASE,
	[FTAddName]	varchar(200) COLLATE NOCASE,
	[FTAddRmk]	varchar(200) COLLATE NOCASE,
	[FTAddCountry]	varchar(100) COLLATE NOCASE,
	[FTAreCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTAddVersion]	varchar(1) COLLATE NOCASE,
	[FTAddV1No]	varchar(30) COLLATE NOCASE,
	[FTAddV1Soi]	varchar(30) COLLATE NOCASE,
	[FTAddV1Village]	varchar(70) COLLATE NOCASE,
	[FTAddV1Road]	varchar(30) COLLATE NOCASE,
	[FTAddV1SubDist]	varchar(30) COLLATE NOCASE,
	[FTAddV1DstCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PvnCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PostCode]	varchar(5) COLLATE NOCASE,
	[FTAddTel]	varchar(50) COLLATE NOCASE,
	[FTAddFax]	varchar(50) COLLATE NOCASE,
	[FTAddV2Desc1]	varchar(255) COLLATE NOCASE,
	[FTAddV2Desc2]	varchar(255) COLLATE NOCASE,
	[FTAddWebsite]	varchar(200) COLLATE NOCASE,
	[FTAddLongitude]	varchar(50) COLLATE NOCASE,
	[FTAddLatitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstCard] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDCstApply]	datetime,
	[FTCstCrdNo]	varchar(30) COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDCstCrdIssue]	datetime,
	[FDCstCrdExpire]	datetime,
	[FTCstStaAge]	varchar(1) COLLATE NOCASE, FTCgpCode TEXT, FTAgnCode TEXT, FTCstUserRegis TEXT,
    PRIMARY KEY ([FTCstCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstCardTmp] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDCstApply]	datetime,
	[FTCstCrdNo]	varchar(30) COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDCstCrdIssue]	datetime,
	[FDCstCrdExpire]	datetime,
	[FTCstStaAge]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstContact_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FNCtrSeq]	integer NOT NULL,
	[FTCtrRefBch]	varchar(5) COLLATE NOCASE,
	[FTCtrName]	varchar(200) COLLATE NOCASE,
	[FTCtrFax]	varchar(50) COLLATE NOCASE,
	[FTCtrTel]	varchar(50) COLLATE NOCASE,
	[FTCtrEmail]	varchar(50) COLLATE NOCASE,
	[FTCtrRmk]	varchar(100) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTAgnCode TEXT,
    PRIMARY KEY ([FTCstCode], [FNLngID], [FNCtrSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMCstContactTmp_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FNCtrSeq]	integer NOT NULL,
	[FTCtrRefBch]	varchar(5) COLLATE NOCASE,
	[FTCtrName]	varchar(200) COLLATE NOCASE,
	[FTCtrFax]	varchar(50) COLLATE NOCASE,
	[FTCtrTel]	varchar(50) COLLATE NOCASE,
	[FTCtrEmail]	varchar(50) COLLATE NOCASE,
	[FTCtrRmk]	varchar(100) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstCredit] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCstCrTerm]	integer,
	[FCCstCrLimit]	real,
	[FTCstStaAlwOrdSun]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdMon]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdTue]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdWed]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdThu]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdFri]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdSat]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwPosCalSo]	varchar(1) COLLATE NOCASE,
	[FDCstLastCta]	datetime,
	[FDCstLastPay]	datetime,
	[FTCstPayRmk]	varchar(100) COLLATE NOCASE,
	[FTCstBillRmk]	varchar(100) COLLATE NOCASE,
	[FNCstViaTime]	integer,
	[FTCstViaRmk]	varchar(100) COLLATE NOCASE,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
	[FTCstTspPaid]	varchar(1) COLLATE NOCASE,
	[FTCstStaApv]	varchar(1) COLLATE NOCASE, FTAgnCode TEXT, FTCcpCode TEXT, FTBchCode TEXT, FTPahDocNo TEXT,
    PRIMARY KEY ([FTCstCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstCreditTmp] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCstCrTerm]	integer,
	[FCCstCrLimit]	real,
	[FTCstStaAlwOrdSun]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdMon]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdTue]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdWed]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdThu]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdFri]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwOrdSat]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwPosCalSo]	varchar(1) COLLATE NOCASE,
	[FDCstLastCta]	datetime,
	[FDCstLastPay]	datetime,
	[FTCstPayRmk]	varchar(100) COLLATE NOCASE,
	[FTCstBillRmk]	varchar(100) COLLATE NOCASE,
	[FNCstViaTime]	integer,
	[FTCstViaRmk]	varchar(100) COLLATE NOCASE,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
	[FTCstTspPaid]	varchar(1) COLLATE NOCASE,
	[FTCstStaApv]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstGrp] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstGrp_L] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCgpName]	varchar(100) COLLATE NOCASE,
	[FTCgpRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCstGrpTmp] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstGrpTmp_L] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCgpName]	varchar(100) COLLATE NOCASE,
	[FTCgpRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstLev] (
	[FTClvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTClvCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstLev_L] (
	[FTClvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTClvName]	varchar(100) COLLATE NOCASE,
	[FTClvRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTClvCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCstLevTmp] (
	[FTClvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstLevTmp_L] (
	[FTClvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTClvName]	varchar(100) COLLATE NOCASE,
	[FTClvRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstLogin] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCstLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDCstPwdStart]	datetime NOT NULL,
	[FDCstPwdExpired]	datetime,
	[FTCstLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCstLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCstRmk]	varchar(50) COLLATE NOCASE,
	[FTCstStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTCstCode], [FTCstLogType], [FDCstPwdStart])

);
CREATE TABLE IF NOT EXISTS [TCNMCstOcp] (
	[FTOcpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTOcpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstOcp_L] (
	[FTOcpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTOcpName]	varchar(50) COLLATE NOCASE,
	[FTOcpRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTOcpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCstRFID_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCstID]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCrfName]	varchar(200) COLLATE NOCASE, FTAgnCode TEXT,
    PRIMARY KEY ([FTCstCode], [FTCstID], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMCstRFIDTmp_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCstID]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCrfName]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstTmp] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTCstCardID]	varchar(20) COLLATE NOCASE,
	[FTCstTaxNo]	varchar(20) COLLATE NOCASE,
	[FTCstTel]	varchar(50) COLLATE NOCASE,
	[FTCstFax]	varchar(50) COLLATE NOCASE,
	[FTCstEmail]	varchar(100) COLLATE NOCASE,
	[FTCstSex]	varchar(1) COLLATE NOCASE,
	[FDCstDob]	datetime,
	[FTCgpCode]	varchar(5) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTClvCode]	varchar(5) COLLATE NOCASE,
	[FTPplCodeRet]	varchar(20) COLLATE NOCASE,
	[FTPplCodeWhs]	varchar(20) COLLATE NOCASE,
	[FTPplCodenNet]	varchar(20) COLLATE NOCASE,
	[FTPmgCode]	varchar(20) COLLATE NOCASE,
	[FTOcpCode]	varchar(5) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTCstDiscWhs]	varchar(50) COLLATE NOCASE,
	[FTCstDiscRet]	varchar(50) COLLATE NOCASE,
	[FTCstBusiness]	varchar(1) COLLATE NOCASE,
	[FTCstBchHQ]	varchar(1) COLLATE NOCASE,
	[FTCstBchCode]	varchar(50) COLLATE NOCASE,
	[FTCstStaActive]	varchar(1) COLLATE NOCASE,
	[FTCstStaAlwPosCalSo]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstTmp_L] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCstName]	varchar(200) COLLATE NOCASE,
	[FTCstNameOth]	varchar(200) COLLATE NOCASE,
	[FTCstRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMCstType] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode])

);
CREATE TABLE IF NOT EXISTS [TCNMCstType_L] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCtyName]	varchar(100) COLLATE NOCASE,
	[FTCtyRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMDistrict] (
	[FTDstCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTDstPost]	varchar(10) COLLATE NOCASE,
	[FTPvnCode]	varchar(5) COLLATE NOCASE,
	[FTDstLatitude]	varchar(50) COLLATE NOCASE,
	[FTDstLongitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTDstCode])

);
CREATE TABLE IF NOT EXISTS [TCNMDistrict_L] (
	[FTDstCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTDstName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTDstCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMDocApvRole] (
	[FNDarID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTDarTable]	varchar(50) COLLATE NOCASE,
	[FTDarRefType]	varchar(50) COLLATE NOCASE,
	[FNDarApvSeq]	integer,
	[FTDarUsrRole]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMEventDT] (
	[FTEvhCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNEvdSeqNo]	integer NOT NULL,
	[FTEvdType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTEvdTStart]	varchar(8) COLLATE NOCASE,
	[FDEvdDStart]	datetime,
	[FTEvdTFinish]	varchar(8) COLLATE NOCASE,
	[FDEvdDFinish]	datetime,
	[FTEvdStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTEvhCode], [FNEvdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNMEventDT_L] (
	[FTEvhCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNEvdSeqNo]	integer NOT NULL,
	[FNLngID]	integer NOT NULL,
	[FTEvdName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTEvhCode], [FNEvdSeqNo], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMEventHD] (
	[FTEvhCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTEvhStaActive]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTEvhCode])

);
CREATE TABLE IF NOT EXISTS [TCNMEventHD_L] (
	[FTEvhCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTEvhName]	varchar(100) COLLATE NOCASE,
	[FTEvhRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTEvhCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMImgObj] (
	[FNImgID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMImgObjTmp] (
	[FNImgID]	integer NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMImgPdt] (
	[FNImgID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMImgPdtTmp] (
	[FNImgID]	integer NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMImgPerson] (
	[FNImgID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMImgPersonTmp] (
	[FNImgID]	integer NOT NULL,
	[FTImgRefID]	varchar(30) COLLATE NOCASE,
	[FNImgSeq]	integer,
	[FTImgTable]	varchar(50) COLLATE NOCASE,
	[FTImgKey]	varchar(20) COLLATE NOCASE,
	[FTImgObj]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMLimitRole] (
	[FTLhdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLimSeq]	integer NOT NULL,
	[FCLimMin]	real,
	[FCLimMax]	real,
	[FTLimStaWarn]	varchar(1) COLLATE NOCASE,
	[FTLimMsgSpc]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTLhdCode], [FTRolCode], [FNLimSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMMediaObj] (
	[FNMedID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTMedRefID]	varchar(30) COLLATE NOCASE,
	[FNMedSeq]	integer,
	[FNMedType]	integer,
	[FTMedFileType]	varchar(3) COLLATE NOCASE,
	[FTMedTable]	varchar(50) COLLATE NOCASE,
	[FTMedKey]	varchar(20) COLLATE NOCASE,
	[FTMedPath]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMMediaObjTmp] (
	[FNMedID]	integer NOT NULL,
	[FTMedRefID]	varchar(30) COLLATE NOCASE,
	[FNMedSeq]	integer,
	[FNMedType]	integer,
	[FTMedFileType]	varchar(3) COLLATE NOCASE,
	[FTMedTable]	varchar(50) COLLATE NOCASE,
	[FTMedKey]	varchar(20) COLLATE NOCASE,
	[FTMedPath]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMMerchant] (
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMerEmail]	varchar(50) COLLATE NOCASE,
	[FTMerTel]	varchar(50) COLLATE NOCASE,
	[FTMerFax]	varchar(50) COLLATE NOCASE,
	[FTMerMo]	varchar(50) COLLATE NOCASE,
	[FTMerStaActive]	varchar(1) COLLATE NOCASE,
	[FTMerRefCode]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTMerCode])

);
CREATE TABLE IF NOT EXISTS [TCNMMerchant_L] (
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTMerName]	varchar(200) COLLATE NOCASE,
	[FTMerRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTMerCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMMerchantTmp] (
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMerEmail]	varchar(50) COLLATE NOCASE,
	[FTMerTel]	varchar(50) COLLATE NOCASE,
	[FTMerFax]	varchar(50) COLLATE NOCASE,
	[FTMerMo]	varchar(50) COLLATE NOCASE,
	[FTMerStaActive]	varchar(1) COLLATE NOCASE,
	[FTMerRefCode]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMMerchantTmp_L] (
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTMerName]	varchar(200) COLLATE NOCASE,
	[FTMerRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMMerPdtGrp] (
	[FTMgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTMgpCode], [FTMerCode])

);
CREATE TABLE IF NOT EXISTS [TCNMMerPdtGrp_L] (
	[FTMgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTMgpName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTMgpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdt_L] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTPdtNameOth]	varchar(100) COLLATE NOCASE,
	[FTPdtNameABB]	varchar(50) COLLATE NOCASE,
	[FTPdtRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtAge] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtAge]	real,
	[FCPdtMaxDegree]	real,
	[FCPdtMinDegree]	real,
	[FTPdtIngredients]	varchar(200) COLLATE NOCASE,
	[FTPdtHowToUse]	varchar(200) COLLATE NOCASE,
	[FTPdtWarning]	varchar(200) COLLATE NOCASE,
	[FDPdtMfg]	datetime,
	[FDPdtExp]	datetime,
	[FTPdtPerVolumn]	varchar(100) COLLATE NOCASE,
	[FCPdtPerCalories]	real,
	[FCPdtCookTime]	real,
	[FCPdtCookHeat]	real,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtBarTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBarCode]	varchar(25) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBarStaUse]	varchar(1) COLLATE NOCASE,
	[FTBarStaAlwSale]	varchar(1) COLLATE NOCASE,
	[FTBarStaByGen]	varchar(1) COLLATE NOCASE,
	[FTPlcCode]	varchar(5) COLLATE NOCASE,
	[FNPldSeq]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtBrand] (
	[FTPbnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPbnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtBrand_L] (
	[FTPbnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPbnName]	varchar(50) COLLATE NOCASE,
	[FTPbnRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPbnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtBrandTmp] (
	[FTPbnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtBrandTmp_L] (
	[FTPbnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPbnName]	varchar(50) COLLATE NOCASE,
	[FTPbnRmk]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtColor] (
	[FTClrCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTClrRefValue]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTClrCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtColor_L] (
	[FTClrCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTClrName]	varchar(50) COLLATE NOCASE,
	[FTClrRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTClrCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtColorTmp] (
	[FTClrCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTClrRefValue]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtColorTmp_L] (
	[FTClrCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTClrName]	varchar(50) COLLATE NOCASE,
	[FTClrRmk]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostAvg] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FCPdtCostLast]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostAvg4AGG] (
	[FTAggCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtCostLast]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTAggCode], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostAvg4BCH] (
	[FTXchBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtCostLast]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTXchBchTo], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostAvg4ZNE] (
	[FTXchZneTo]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtCostLast]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTXchZneTo], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostAvgTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FCPdtCostLast]	real,
	[FDLastUpdOn]	datetime

);
CREATE TABLE IF NOT EXISTS [TCNMPdtCostFIFO] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtDocRef]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPdtAffect]	datetime NOT NULL,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtCostLast]	real,
	[FCPdtDocQtyAll]	real,
	[FCPdtQtyAllUse]	real,
	[FCPdtCostAmt]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTPdtCode], [FTPdtDocRef])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtDrug] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCPdgAge]	real,
	[FDPdgCreate]	datetime,
	[FDPdgExpired]	datetime,
	[FTPdgHowtoUse]	varchar(200) COLLATE NOCASE,
	[FTPdgActIngredient]	varchar(200) COLLATE NOCASE,
	[FTPdgProperties]	varchar(200) COLLATE NOCASE,
	[FTPdgCtd]	varchar(200) COLLATE NOCASE,
	[FTPdgWarn]	varchar(200) COLLATE NOCASE,
	[FTPdgStopUse]	varchar(200) COLLATE NOCASE,
	[FCPdgDoseSchedule]	real,
	[FCPdgMaxIntake]	real,
	[FTPdgBrandName]	varchar(100) COLLATE NOCASE,
	[FTPdgGenericName]	varchar(100) COLLATE NOCASE,
	[FTPdgCategory]	varchar(50) COLLATE NOCASE,
	[FTPdgType]	varchar(50) COLLATE NOCASE,
	[FTPdgRegNo]	varchar(20) COLLATE NOCASE,
	[FTPdgStorage]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPdgForm]	varchar(50) COLLATE NOCASE,
	[FTPdgCtrlRole]	varchar(10) COLLATE NOCASE,
	[FTPdgManufacturer]	varchar(50) COLLATE NOCASE,
	[FTPdgVolume]	varchar(50) COLLATE NOCASE,
	[FTPdgStaType]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF0DCS] (
	[FTDcsCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTDepCode]	varchar(10) COLLATE NOCASE,
	[FTClsCode]	varchar(10) COLLATE NOCASE,
	[FTSclCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTDcsCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF0DCS_L] (
	[FTDcsCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTDcsName]	varchar(150) COLLATE NOCASE,
	[FTDcsRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTDcsCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF1Depart] (
	[FTDepCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTDepCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF1Depart_L] (
	[FTDepCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTDepName]	varchar(50) COLLATE NOCASE,
	[FTDepRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTDepCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF2Class] (
	[FTClsCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTClsCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF2Class_L] (
	[FTClsCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTClsName]	varchar(50) COLLATE NOCASE,
	[FTClsRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTClsCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF3SubClass] (
	[FTSclCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSclCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtF3SubClass_L] (
	[FTSclCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSclName]	varchar(50) COLLATE NOCASE,
	[FTSclRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTSclCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtFCostAvg] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPszCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTClrCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCPdtCostEx]	real,
	[FCPdtCostIn]	real,
	[FCPdtCostLast]	real,
	[FCPdtQtyBal]	real,
	[FCPdtCostAmt]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTPdtCode], [FTPszCode], [FTClrCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtFSeason] (
	[FTPgpCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FNPgpLevel]	integer NOT NULL,
	[FTPgpParent]	varchar(3) COLLATE NOCASE,
	[FTPgpChain]	varchar(30) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPgpCode], [FNPgpLevel])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtFSeason_L] (
	[FTPgpCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FNPgpLevel]	integer NOT NULL,
	[FNLngID]	integer NOT NULL,
	[FTPgpName]	varchar(100) COLLATE NOCASE,
	[FTPgpRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTPgpCode], [FNPgpLevel], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtGrp] (
	[FTPgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPgpChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNPgpLevel]	integer NOT NULL,
	[FTPgpParent]	varchar(5) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPgpCode], [FTPgpChain])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtGrp_L] (
	[FTPgpChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPgpName]	varchar(100) COLLATE NOCASE,
	[FTPgpChainName]	varchar(255) COLLATE NOCASE,
	[FTPgpRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTPgpChain], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtGrpTmp] (
	[FTPgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPgpChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNPgpLevel]	integer NOT NULL,
	[FTPgpParent]	varchar(5) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtGrpTmp_L] (
	[FTPgpChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPgpName]	varchar(100) COLLATE NOCASE,
	[FTPgpChainName]	varchar(255) COLLATE NOCASE,
	[FTPgpRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLoc] (
	[FTPlcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTZneChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTPigCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPlcCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLoc_L] (
	[FTPlcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPlcName]	varchar(100) COLLATE NOCASE,
	[FTPlcRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTPlcCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLocPickGrp] (
	[FTPigCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPigCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLocPickGrp_L] (
	[FTPigCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPigName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTPigCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLocZone] (
	[FTZneChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNZneLevel]	integer NOT NULL,
	[FTZneParent]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTZneChain])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtLocZone_L] (
	[FTZneCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTZneName]	varchar(100) COLLATE NOCASE,
	[FTZneRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTZneCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtModel] (
	[FTPmoCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPmoCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtModel_L] (
	[FTPmoCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmoName]	varchar(50) COLLATE NOCASE,
	[FTPmoRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPmoCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtModelTmp] (
	[FTPmoCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtModelTmp_L] (
	[FTPmoCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmoName]	varchar(50) COLLATE NOCASE,
	[FTPmoRmk]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPackSize] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCPdtUnitFact]	real,
	[FCPdtPriceRET]	real,
	[FCPdtPriceWHS]	real,
	[FCPdtPriceNET]	real,
	[FTPdtGrade]	varchar(50) COLLATE NOCASE,
	[FCPdtWeight]	real,
	[FTClrCode]	varchar(5) COLLATE NOCASE,
	[FTPszCode]	varchar(5) COLLATE NOCASE,
	[FTPdtUnitDim]	varchar(50) COLLATE NOCASE,
	[FTPdtPkgDim]	varchar(50) COLLATE NOCASE,
	[FTPdtStaAlwPick]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwPoHQ]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwBuy]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwSale]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTPdtStaAlwRet VARCHAR(1),
    PRIMARY KEY ([FTPdtCode], [FTPunCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPackSizeTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCPdtUnitFact]	real,
	[FCPdtPriceRET]	real,
	[FCPdtPriceWHS]	real,
	[FCPdtPriceNET]	real,
	[FTPdtGrade]	varchar(50) COLLATE NOCASE,
	[FCPdtWeight]	real,
	[FTClrCode]	varchar(5) COLLATE NOCASE,
	[FTPszCode]	varchar(5) COLLATE NOCASE,
	[FTPdtUnitDim]	varchar(50) COLLATE NOCASE,
	[FTPdtPkgDim]	varchar(50) COLLATE NOCASE,
	[FTPdtStaAlwPick]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwPoHQ]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwBuy]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwSale]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPmtGrp] (
	[FTPmgCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPmgCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPmtGrp_L] (
	[FTPmgCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmgName]	varchar(200) COLLATE NOCASE,
	[FTPmgRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTPmgCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPriListTmp] (
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtPriListTmp_L] (
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPplName]	varchar(200) COLLATE NOCASE,
	[FTPplRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSize] (
	[FTPszCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPszCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSize_L] (
	[FTPszCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPszName]	varchar(50) COLLATE NOCASE,
	[FTPszRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPszCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSizeTmp] (
	[FTPszCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSizeTmp_L] (
	[FTPszCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPszName]	varchar(50) COLLATE NOCASE,
	[FTPszRmk]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSpcBch] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FTMerCode]	varchar(10) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTMgpCode]	varchar(5) COLLATE NOCASE,
	[FCPdtMin]	real,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTPdtRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSpcBchTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FTMerCode]	varchar(10) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTMgpCode]	varchar(5) COLLATE NOCASE,
	[FCPdtMin]	real,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTPdtRmk]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtSpcWah] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCSpwQtyMin]	real,
	[FCSpwQtyMax]	real,
	[FTSpwRmk]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FTBchCode], [FTWahCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtStkControl]	varchar(1) COLLATE NOCASE,
	[FTPdtGrpControl]	varchar(1) COLLATE NOCASE,
	[FTPdtForSystem]	varchar(30) COLLATE NOCASE,
	[FCPdtQtyOrdBuy]	real,
	[FCPdtCostDef]	real,
	[FCPdtCostOth]	real,
	[FCPdtCostStd]	real,
	[FCPdtMin]	real,
	[FCPdtMax]	real,
	[FTPdtPoint]	varchar(1) COLLATE NOCASE,
	[FCPdtPointTime]	real,
	[FTPdtType]	varchar(1) COLLATE NOCASE,
	[FTPdtSaleType]	varchar(1) COLLATE NOCASE,
	[FTPdtSetOrSN]	varchar(1) COLLATE NOCASE,
	[FTPdtStaSetPri]	varchar(1) COLLATE NOCASE,
	[FTPdtStaSetShwDT]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwDis]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwReturn]	varchar(1) COLLATE NOCASE,
	[FTPdtStaVatBuy]	varchar(1) COLLATE NOCASE,
	[FTPdtStaVat]	varchar(1) COLLATE NOCASE,
	[FTPdtStaActive]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwReCalOpt]	varchar(1) COLLATE NOCASE,
	[FTPdtStaCsm]	varchar(1) COLLATE NOCASE,
	[FTTcgCode]	varchar(5) COLLATE NOCASE,
	[FTPgpChain]	varchar(30) COLLATE NOCASE,
	[FTPtyCode]	varchar(5) COLLATE NOCASE,
	[FTPbnCode]	varchar(5) COLLATE NOCASE,
	[FTPmoCode]	varchar(5) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FTEvhCode]	varchar(5) COLLATE NOCASE,
	[FDPdtSaleStart]	datetime,
	[FDPdtSaleStop]	datetime,
	[FTPdtStaSetPrcStk]	varchar(1) COLLATE NOCASE,
	[FTPdtCtrlRole]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTmp_L] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTPdtNameOth]	varchar(100) COLLATE NOCASE,
	[FTPdtNameABB]	varchar(50) COLLATE NOCASE,
	[FTPdtRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTouchGrp] (
	[FTTcgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTTcgStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTTcgCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTouchGrp_L] (
	[FTTcgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTTcgName]	varchar(100) COLLATE NOCASE,
	[FTTcgRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTTcgCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTouchGrpTmp] (
	[FTTcgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTTcgStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTouchGrpTmp_L] (
	[FTTcgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTTcgName]	varchar(100) COLLATE NOCASE,
	[FTTcgRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtType] (
	[FTPtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPtyCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtType_L] (
	[FTPtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPtyName]	varchar(100) COLLATE NOCASE,
	[FTPtyRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTPtyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTypeTmp] (
	[FTPtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtTypeTmp_L] (
	[FTPtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPtyName]	varchar(100) COLLATE NOCASE,
	[FTPtyRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtUnit] (
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPunCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtUnit_L] (
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPunName]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTPunCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPdtUnitTmp] (
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPdtUnitTmp_L] (
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPunName]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPos_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPosName]	varchar(100) COLLATE NOCASE,
	[FTPosNameOth]	varchar(100) COLLATE NOCASE,
	[FTPosRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPosCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPosAds] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNPsdSeq]	integer NOT NULL,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTPsdPosition]	varchar(2) COLLATE NOCASE,
	[FTAdvCode]	varchar(5) COLLATE NOCASE,
	[FNPsdWide]	integer,
	[FNPsdHigh]	integer,
	[FDPsdStart]	datetime,
	[FDPsdStop]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTShpCode], [FTPosCode], [FNPsdSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMPosAdsTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNPsdSeq]	integer NOT NULL,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTPsdPosition]	varchar(2) COLLATE NOCASE,
	[FTAdvCode]	varchar(5) COLLATE NOCASE,
	[FNPsdWide]	integer,
	[FNPsdHigh]	integer,
	[FDPsdStart]	datetime,
	[FDPsdStop]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPosHW] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPhwCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTShwCode]	varchar(10) COLLATE NOCASE,
	[FTPhwCodeRef]	varchar(30) COLLATE NOCASE,
	[FNPhwSeq]	integer NOT NULL,
	[FTPhwName]	varchar(100) COLLATE NOCASE,
	[FTPhwConnType]	varchar(255) COLLATE NOCASE,
	[FTPhwConnRef]	varchar(255) COLLATE NOCASE,
	[FTPhwCustom]	varchar(255) COLLATE NOCASE,
	[FNPhwTimeOut]	integer,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPosCode], [FTPhwCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPosHWTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPhwCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTShwCode]	varchar(10) COLLATE NOCASE,
	[FTPhwCodeRef]	varchar(30) COLLATE NOCASE,
	[FNPhwSeq]	integer NOT NULL,
	[FTPhwName]	varchar(100) COLLATE NOCASE,
	[FTPhwConnType]	varchar(255) COLLATE NOCASE,
	[FTPhwConnRef]	varchar(255) COLLATE NOCASE,
	[FTPhwCustom]	varchar(255) COLLATE NOCASE,
	[FNPhwTimeOut]	integer,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPosLastNo] (
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNPosDocType]	integer NOT NULL,
	[FTPosComName]	varchar(50) COLLATE NOCASE,
	[FNPosLastNo]	integer,
	[FDPosLastSale]	datetime,
    PRIMARY KEY ([FTPosCode], [FNPosDocType])

);
CREATE TABLE IF NOT EXISTS [TCNMPosLastNoTmp] (
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNPosDocType]	integer NOT NULL,
	[FTPosComName]	varchar(50) COLLATE NOCASE,
	[FNPosLastNo]	integer,
	[FDPosLastSale]	datetime

);
CREATE TABLE IF NOT EXISTS [TCNMPosTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosType]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTPosRegNo]	varchar(20) COLLATE NOCASE,
	[FTSmgCode]	varchar(5) COLLATE NOCASE,
	[FTPosStaRorW]	varchar(1) COLLATE NOCASE,
	[FTPosStaPrnEJ]	varchar(1) COLLATE NOCASE,
	[FTPosStaVatSend]	varchar(1) COLLATE NOCASE,
	[FTPosStaUse]	varchar(1) COLLATE NOCASE,
	[FTPosStaShift]	varchar(1) COLLATE NOCASE,
	[FTPosStaDate]	varchar(1) COLLATE NOCASE,
	[FTPosStaSumScan]	varchar(1) COLLATE NOCASE,
	[FTPosStaSumPrn]	varchar(1) COLLATE NOCASE,
	[FTPrgRegToken]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPosTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPosName]	varchar(100) COLLATE NOCASE,
	[FTPosNameOth]	varchar(100) COLLATE NOCASE,
	[FTPosRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPrinter] (
	[FTPrnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FTPrnSrcType]	varchar(1) COLLATE NOCASE,
	[FTSppCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTPrnDriver]	varchar(200) COLLATE NOCASE,
	[FTPrnType]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPrnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMPrinter_L] (
	[FTPrnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPrnName]	varchar(100) COLLATE NOCASE,
	[FTPrnRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTPrnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMPrinterTmp] (
	[FTPrnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FTPrnSrcType]	varchar(1) COLLATE NOCASE,
	[FTSppCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTPrnDriver]	varchar(200) COLLATE NOCASE,
	[FTPrnType]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMPrinterTmp_L] (
	[FTPrnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPrnName]	varchar(100) COLLATE NOCASE,
	[FTPrnRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMProvince] (
	[FTPvnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(5) COLLATE NOCASE,
	[FTPvnLatitude]	varchar(50) COLLATE NOCASE,
	[FTPvnLongitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPvnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMProvince_L] (
	[FTPvnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPvnName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTPvnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMProvinceTmp] (
	[FTPvnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(5) COLLATE NOCASE,
	[FTPvnLatitude]	varchar(50) COLLATE NOCASE,
	[FTPvnLongitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMProvinceTmp_L] (
	[FTPvnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPvnName]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMRsn] (
	[FTRsnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTRsgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRsnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMRsn_L] (
	[FTRsnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRsnName]	varchar(200) COLLATE NOCASE,
	[FTRsnRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTRsnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMRsnTmp] (
	[FTRsnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTRsgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMRsnTmp_L] (
	[FTRsnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRsnName]	varchar(200) COLLATE NOCASE,
	[FTRsnRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMScale] (
	[FTSclCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTSslCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTSclConType]	varchar(1) COLLATE NOCASE,
	[FTSclStaPrinter]	varchar(1) COLLATE NOCASE,
	[FTSclStaPrinDiver]	varchar(150) COLLATE NOCASE,
	[FTSclLanIP]	varchar(15) COLLATE NOCASE,
	[FNSclLanPort]	integer,
	[FNSclComPort]	integer,
	[FNSclTimer]	integer,
	[FTSclUnit]	varchar(20) COLLATE NOCASE,
	[FTSclComUsed]	varchar(50) COLLATE NOCASE,
	[FTPosCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSclCode], [FTSslCode])

);
CREATE TABLE IF NOT EXISTS [TCNMShipVia] (
	[FTViaCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTViaCode])

);
CREATE TABLE IF NOT EXISTS [TCNMShipVia_L] (
	[FTViaCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTViaName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTViaCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMShop] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTShpStaShwPrice]	varchar(1) COLLATE NOCASE,
	[FTShpType]	varchar(1) COLLATE NOCASE,
	[FTShpRegNo]	varchar(30) COLLATE NOCASE,
	[FTShpRefID]	varchar(30) COLLATE NOCASE,
	[FDShpStart]	datetime,
	[FDShpStop]	datetime,
	[FDShpSaleStart]	datetime,
	[FDShpSaleStop]	datetime,
	[FTShpStaActive]	varchar(1) COLLATE NOCASE,
	[FTShpStaClose]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTShpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMShop_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTShpName]	varchar(100) COLLATE NOCASE,
	[FTShpRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTShpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMShopGP] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDSgpStart]	datetime NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FNSgpSeq]	integer NOT NULL,
	[FCSgpPerAvg]	real,
	[FCSgpPerSun]	real,
	[FCSgpPerMon]	real,
	[FCSgpPerTue]	real,
	[FCSgpPerWed]	real,
	[FCSgpPerThu]	real,
	[FCSgpPerFri]	real,
	[FCSgpPerSat]	real,
    PRIMARY KEY ([FTBchCode], [FTShpCode], [FDSgpStart], [FNSgpSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMShopTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTShpStaShwPrice]	varchar(1) COLLATE NOCASE,
	[FTShpType]	varchar(1) COLLATE NOCASE,
	[FTShpRegNo]	varchar(30) COLLATE NOCASE,
	[FTShpRefID]	varchar(30) COLLATE NOCASE,
	[FDShpStart]	datetime,
	[FDShpStop]	datetime,
	[FDShpSaleStart]	datetime,
	[FDShpSaleStop]	datetime,
	[FTShpStaActive]	varchar(1) COLLATE NOCASE,
	[FTShpStaClose]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMShopTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTShpName]	varchar(100) COLLATE NOCASE,
	[FTShpRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMShpWah] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTShpCode], [FTWahCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSlipMsgDT_L] (
	[FTSmgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSmgType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNSmgSeq]	integer NOT NULL,
	[FTSmgName]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTSmgCode], [FNLngID], [FTSmgType], [FNSmgSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMSlipMsgDTTmp_L] (
	[FTSmgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSmgType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNSmgSeq]	integer NOT NULL,
	[FTSmgName]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMSlipMsgHD_L] (
	[FTSmgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTSmgTitle]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSmgCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSlipMsgHDTmp_L] (
	[FTSmgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTSmgTitle]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMSpl] (
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FTSplTel]	varchar(50) COLLATE NOCASE,
	[FTSplFax]	varchar(50) COLLATE NOCASE,
	[FTSplEmail]	varchar(100) COLLATE NOCASE,
	[FTSplSex]	varchar(1) COLLATE NOCASE,
	[FDSplDob]	datetime,
	[FTSgpCode]	varchar(5) COLLATE NOCASE,
	[FTStyCode]	varchar(5) COLLATE NOCASE,
	[FTSlvCode]	varchar(5) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FTSplStaVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTSplDiscBillRet]	varchar(50) COLLATE NOCASE,
	[FTSplDiscBillWhs]	varchar(50) COLLATE NOCASE,
	[FTSplDiscBillNet]	varchar(50) COLLATE NOCASE,
	[FTSplBusiness]	varchar(1) COLLATE NOCASE,
	[FTSplStaBchOrHQ]	varchar(1) COLLATE NOCASE,
	[FTSplBchCode]	varchar(50) COLLATE NOCASE,
	[FTSplStaActive]	varchar(1) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSplCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSpl_L] (
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSplName]	varchar(200) COLLATE NOCASE,
	[FTSplNameOth]	varchar(200) COLLATE NOCASE,
	[FTSplPayRmk]	varchar(100) COLLATE NOCASE,
	[FTSplBillRmk]	varchar(100) COLLATE NOCASE,
	[FTSplViaRmk]	varchar(100) COLLATE NOCASE,
	[FTSplRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTSplCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSplAddress_L] (
	[FTSplCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTAddGrpType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FNAddSeqNo]	integer NOT NULL,
	[FTAddRefNo]	varchar(20) COLLATE NOCASE,
	[FTAddName]	varchar(200) COLLATE NOCASE,
	[FTAddTaxNo]	varchar(20) COLLATE NOCASE,
	[FTAddRmk]	varchar(200) COLLATE NOCASE,
	[FTAddCountry]	varchar(100) COLLATE NOCASE,
	[FTAreCode]	varchar(5) COLLATE NOCASE,
	[FTZneCode]	varchar(30) COLLATE NOCASE,
	[FTAddVersion]	varchar(1) COLLATE NOCASE,
	[FTAddV1No]	varchar(30) COLLATE NOCASE,
	[FTAddV1Soi]	varchar(30) COLLATE NOCASE,
	[FTAddV1Village]	varchar(70) COLLATE NOCASE,
	[FTAddV1Road]	varchar(30) COLLATE NOCASE,
	[FTAddV1SubDist]	varchar(30) COLLATE NOCASE,
	[FTAddV1DstCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PvnCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PostCode]	varchar(5) COLLATE NOCASE,
	[FTAddTel]	varchar(50) COLLATE NOCASE,
	[FTAddFax]	varchar(50) COLLATE NOCASE,
	[FTAddV2Desc1]	varchar(255) COLLATE NOCASE,
	[FTAddV2Desc2]	varchar(255) COLLATE NOCASE,
	[FTAddWebsite]	varchar(200) COLLATE NOCASE,
	[FTAddLongitude]	varchar(50) COLLATE NOCASE,
	[FTAddLatitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTAgnCode TEXT, FTAddAttn TEXT, FTAddContactTo TEXT, FTAddContactTel TEXT, FTAddV1Moo TEXT, FTAddV1Building TEXT, FTAddV1Floor TEXT, FTAddV1Room TEXT, FTAddV1DstName TEXT, FTAddV1PvnName TEXT, FTAddV1CtrName TEXT, FTAddV2Desc3 TEXT, FTAddFullText TEXT,
    PRIMARY KEY ([FTSplCode], [FNLngID], [FTAddGrpType], [FNAddSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNMSplCard] (
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDSplApply]	datetime,
	[FTSplRefExCrdNo]	varchar(30) COLLATE NOCASE,
	[FDSplCrdIssue]	datetime,
	[FDSplCrdExpire]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSplCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSplContact_L] (
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FNCtrSeq]	integer NOT NULL,
	[FTCtrName]	varchar(200) COLLATE NOCASE,
	[FTCtrFax]	varchar(50) COLLATE NOCASE,
	[FTCtrTel]	varchar(50) COLLATE NOCASE,
	[FTCtrEmail]	varchar(50) COLLATE NOCASE,
	[FTCtrRmk]	varchar(100) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSplCode], [FNLngID], [FNCtrSeq])

);
CREATE TABLE IF NOT EXISTS [TCNMSplCredit] (
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNSplCrTerm]	integer,
	[FCSplCrLimit]	real,
	[FTSplDayCta]	varchar(20) COLLATE NOCASE,
	[FDSplLastCta]	datetime,
	[FDSplLastPay]	datetime,
	[FNSplLimitRow]	integer,
	[FCSplLeadTime]	real,
	[FTSplViaRmk]	varchar(100) COLLATE NOCASE,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
	[FTSplTspPaid]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSplCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSplGrp] (
	[FTSgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSgpCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSplGrp_L] (
	[FTSgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSgpName]	varchar(100) COLLATE NOCASE,
	[FTSgpRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTSgpCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSplLev] (
	[FTSlvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSlvCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSplLev_L] (
	[FTSlvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSlvName]	varchar(100) COLLATE NOCASE,
	[FTSlvRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTSlvCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSplType] (
	[FTStyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTStyCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSplType_L] (
	[FTStyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTStyName]	varchar(100) COLLATE NOCASE,
	[FTStyRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTStyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSpn] (
	[FTSpnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSpnTel]	varchar(50) COLLATE NOCASE,
	[FCSpnSleAmt]	real,
	[FTSpnEmail]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSpnCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSpn_L] (
	[FTSpnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSpnName]	varchar(100) COLLATE NOCASE,
	[FTSpnRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTSpnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSubDistrict] (
	[FTSudCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTDstCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTSudLatitude]	varchar(50) COLLATE NOCASE,
	[FTSudLongitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSudCode])

);
CREATE TABLE IF NOT EXISTS [TCNMSubDistrict_L] (
	[FTSudCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSudName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTSudCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMSubDistrictTmp] (
	[FTSudCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTDstCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTSudLatitude]	varchar(50) COLLATE NOCASE,
	[FTSudLongitude]	varchar(50) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMSubDistrictTmp_L] (
	[FTSudCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSudName]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMTaxAddress_L] (
	[FTAddTaxNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FNAddSeqNo]	integer NOT NULL,
	[FTCstCode]	varchar(20) COLLATE NOCASE,
	[FTAddName]	varchar(200) COLLATE NOCASE,
	[FTAddRmk]	varchar(200) COLLATE NOCASE,
	[FTAddCountry]	varchar(100) COLLATE NOCASE,
	[FTAreCode]	varchar(5) COLLATE NOCASE,
	[FTZneCode]	varchar(30) COLLATE NOCASE,
	[FTAddVersion]	varchar(1) COLLATE NOCASE,
	[FTAddV1No]	varchar(30) COLLATE NOCASE,
	[FTAddV1Soi]	varchar(30) COLLATE NOCASE,
	[FTAddV1Village]	varchar(70) COLLATE NOCASE,
	[FTAddV1Road]	varchar(30) COLLATE NOCASE,
	[FTAddV1SubDist]	varchar(30) COLLATE NOCASE,
	[FTAddV1DstCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PvnCode]	varchar(5) COLLATE NOCASE,
	[FTAddV1PostCode]	varchar(5) COLLATE NOCASE,
	[FTAddV2Desc1]	varchar(255) COLLATE NOCASE,
	[FTAddV2Desc2]	varchar(255) COLLATE NOCASE,
	[FTAddWebsite]	varchar(200) COLLATE NOCASE,
	[FTAddLongitude]	varchar(50) COLLATE NOCASE,
	[FTAddLatitude]	varchar(50) COLLATE NOCASE,
	[FTAddStaBusiness]	varchar(1) COLLATE NOCASE,
	[FTAddStaHQ]	varchar(1) COLLATE NOCASE,
	[FTAddStaBchCode]	varchar(5) COLLATE NOCASE,
	[FTAddTel]	varchar(50) COLLATE NOCASE,
	[FTAddFax]	varchar(50) COLLATE NOCASE,
	[FTAddRefNo]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTVatStaOffline TEXT, FTAgnCode TEXT,
    PRIMARY KEY ([FTAddTaxNo], [FNLngID], [FNAddSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNMTxnAPI] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTApiTxnType]	varchar(1) COLLATE NOCASE,
	[FTApiPrcType]	varchar(1) COLLATE NOCASE,
	[FTApiGrpPrc]	varchar(5) COLLATE NOCASE,
	[FNApiGrpSeq]	integer,
	[FTApiFmtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiURL]	varchar(255) COLLATE NOCASE,
	[FTApiLoginUsr]	varchar(50) COLLATE NOCASE,
	[FTApiLoginPwd]	varchar(255) COLLATE NOCASE,
	[FTApiToken]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTApiCode])

);
CREATE TABLE IF NOT EXISTS [TCNMTxnAPI_L] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTApiName]	varchar(50) COLLATE NOCASE,
	[FTApiRmk]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTApiCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMTxnAPITmp] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTApiTxnType]	varchar(1) COLLATE NOCASE,
	[FTApiPrcType]	varchar(1) COLLATE NOCASE,
	[FTApiGrpPrc]	varchar(5) COLLATE NOCASE,
	[FNApiGrpSeq]	integer,
	[FTApiFmtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiURL]	varchar(255) COLLATE NOCASE,
	[FTApiLoginUsr]	varchar(50) COLLATE NOCASE,
	[FTApiLoginPwd]	varchar(255) COLLATE NOCASE,
	[FTApiToken]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMTxnAPITmp_L] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTApiName]	varchar(50) COLLATE NOCASE,
	[FTApiRmk]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMTxnSpcAPI] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTCmpCode]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiFmtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiURL]	varchar(255) COLLATE NOCASE,
	[FTSpaUsrCode]	varchar(50) COLLATE NOCASE,
	[FTSpaUsrPwd]	varchar(100) COLLATE NOCASE,
	[FTSpaApiKey]	varchar(100) COLLATE NOCASE,
	[FTSpaRmk]	varchar(100) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTApiCode], [FTCmpCode], [FTAgnCode], [FTBchCode], [FTMerCode], [FTShpCode], [FTPosCode])

);
CREATE TABLE IF NOT EXISTS [TCNMTxnSpcAPITmp] (
	[FTApiCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTCmpCode]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiFmtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTApiURL]	varchar(255) COLLATE NOCASE,
	[FTSpaUsrCode]	varchar(50) COLLATE NOCASE,
	[FTSpaUsrPwd]	varchar(100) COLLATE NOCASE,
	[FTSpaApiKey]	varchar(100) COLLATE NOCASE,
	[FTSpaRmk]	varchar(100) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUrlObj] (
	[FNSocID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTSocRefID]	varchar(30) COLLATE NOCASE,
	[FNSocSeq]	integer,
	[FTSocTable]	varchar(50) COLLATE NOCASE,
	[FTSocKey]	varchar(20) COLLATE NOCASE,
	[FTSocURL]	varchar(255) COLLATE NOCASE,
	[FTSocLogo]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUser] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTUsrTel]	varchar(50) COLLATE NOCASE,
	[FTUsrEmail]	varchar(100) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTUsrCode])

);
CREATE TABLE IF NOT EXISTS [TCNMUser_L] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTUsrName]	varchar(100) COLLATE NOCASE,
	[FTUsrRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTUsrCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMUserTmp] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTUsrTel]	varchar(50) COLLATE NOCASE,
	[FTUsrEmail]	varchar(100) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUserTmp_L] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTUsrName]	varchar(100) COLLATE NOCASE,
	[FTUsrRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUsrActRole] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode], [FTUsrCode])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrActRoleTmp] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUsrDepart] (
	[FTDptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTDptCode])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrDepart_L] (
	[FTDptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTDptName]	varchar(100) COLLATE NOCASE,
	[FTDptRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTDptCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrDepartTmp] (
	[FTDptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUsrDepartTmp_L] (
	[FTDptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTDptName]	varchar(100) COLLATE NOCASE,
	[FTDptRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUsrLogin] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTUsrLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDUsrPwdStart]	datetime NOT NULL,
	[FDUsrPwdExpired]	datetime,
	[FTUsrLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUsrLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUsrRmk]	varchar(50) COLLATE NOCASE,
	[FTUsrStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTUsrCode], [FTUsrLogType], [FDUsrPwdStart])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrLoginTmp] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTUsrLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDUsrPwdStart]	datetime NOT NULL,
	[FDUsrPwdExpired]	datetime,
	[FTUsrLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUsrLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUsrRmk]	varchar(50) COLLATE NOCASE,
	[FTUsrStaActive]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMUsrRole] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRolLevel]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrRole_L] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRolName]	varchar(100) COLLATE NOCASE,
	[FTRolRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMUsrRoleSpc] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode], [FTAgnCode], [FTBchCode])

);
CREATE TABLE IF NOT EXISTS [TCNMVatRate] (
	[FTVatCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDVatStart]	datetime NOT NULL,
	[FCVatRate]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTVatCode], [FDVatStart])

);
CREATE TABLE IF NOT EXISTS [TCNMVatRateTmp] (
	[FTVatCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDVatStart]	datetime NOT NULL,
	[FCVatRate]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMWaHouse] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahStaType]	varchar(1) COLLATE NOCASE,
	[FTWahRefCode]	varchar(50) COLLATE NOCASE,
	[FTWahStaChkStk]	varchar(1) COLLATE NOCASE,
	[FTWahStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTWahStaAlwCntStk VARCHAR(1), FTWahStaAlwCostAmt VARCHAR(1),
    PRIMARY KEY ([FTWahCode], [FTBchCode])

);
CREATE TABLE IF NOT EXISTS [TCNMWaHouse_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTWahName]	varchar(100) COLLATE NOCASE,
	[FTWahRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTWahCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMWahouseTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahStaType]	varchar(1) COLLATE NOCASE,
	[FTWahRefCode]	varchar(50) COLLATE NOCASE,
	[FTWahStaChkStk]	varchar(1) COLLATE NOCASE,
	[FTWahStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMWahouseTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTWahName]	varchar(100) COLLATE NOCASE,
	[FTWahRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNMZone] (
	[FTZneChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTZneCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNZneLevel]	integer NOT NULL,
	[FTZneParent]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAreCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTZneChain])

);
CREATE TABLE IF NOT EXISTS [TCNMZone_L] (
	[FTZneChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTZneName]	varchar(100) COLLATE NOCASE,
	[FTZneCode]	varchar(255) COLLATE NOCASE,
	[FTZneChainName]	varchar(255) COLLATE NOCASE,
	[FTZneRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTZneChain], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNMZoneObj] (
	[FNZneID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTZneRefCode]	varchar(20) COLLATE NOCASE,
	[FTZneTable]	varchar(50) COLLATE NOCASE,
	[FTZneKey]	varchar(20) COLLATE NOCASE,
	[FNZneSeq]	integer,
	[FTZneChain]	varchar(30) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNSChannel] (
	[FTChnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAppCode]	varchar(5) COLLATE NOCASE,
	[FTChnGroup]	varchar(255) COLLATE NOCASE,
	[FNChnSeq]	integer,
	[FTChnStaUse]	varchar(1) COLLATE NOCASE,
	[FTChnAppDef]	varchar(1) COLLATE NOCASE,
	[FTChnRefCode]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTChnCode])

);
CREATE TABLE IF NOT EXISTS [TCNSChannel_L] (
	[FTChnCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTChnName]	varchar(100) COLLATE NOCASE,
	[FTPdtRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTChnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNSLimitDT] (
	[FTLhdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTGdtCallByName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTLdtRefCode]	varchar(20) COLLATE NOCASE,
	[FNLdtSign]	integer,
	[FTLhdStaUse]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTLhdCode], [FTGdtCallByName])

);
CREATE TABLE IF NOT EXISTS [TCNSLimitHD] (
	[FTLhdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTLhdCondType]	varchar(1) COLLATE NOCASE,
	[FTLhdStaAlwSeq]	varchar(1) COLLATE NOCASE,
	[FTLhdStaUse]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTLhdCode])

);
CREATE TABLE IF NOT EXISTS [TCNSLimitHD_L] (
	[FTLhdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTLhdName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTLhdCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNSMenu] (
	[FTLicCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTLicType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTLicURLType]	varchar(1) COLLATE NOCASE,
	[FTLicMnuGrp]	varchar(255) COLLATE NOCASE,
	[FTLicMnuName]	varchar(255) COLLATE NOCASE,
	[FTLicMnuNameOth]	varchar(255) COLLATE NOCASE,
	[FTLicMnuRoute]	varchar(255) COLLATE NOCASE,
	[FTLicStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTAutoHisMas] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAhmTblName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTAhmFedCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTAhmFmtAll]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTAhmFmtChar]	varchar(50) COLLATE NOCASE,
	[FTAhmFmtNum]	varchar(50) COLLATE NOCASE,
	[FNAhmLastNum]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTAhmTblName], [FTAhmFedCode], [FTAhmFmtAll])

);
CREATE TABLE IF NOT EXISTS [TCNTAutoHisTxn] (
	[FTAhmTblName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTAhmFedCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatStaDocType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTAhmFmtAll]	varchar(100) NOT NULL COLLATE NOCASE,
	[FNAhmFedSize]	integer,
	[FTAhmFmtChar]	varchar(50) COLLATE NOCASE,
	[FTAhmStaBch]	varchar(1) COLLATE NOCASE,
	[FTAhmFmtYear]	varchar(50) COLLATE NOCASE,
	[FTAhmFmtMonth]	varchar(50) COLLATE NOCASE,
	[FTAhmFmtDay]	varchar(50) COLLATE NOCASE,
	[FNAhmLastNum]	integer,
	[FTAhmStaReset]	varchar(1) COLLATE NOCASE,
	[FTAhmLastRun]	varchar(50) COLLATE NOCASE,
	[FTSatUsrNum]	varchar(50) COLLATE NOCASE,
	[FTAhmFmtReset]	varchar(100) COLLATE NOCASE,
	[FTAhmFmtPst]	varchar(100) COLLATE NOCASE,
	[FTSatStaAlwSep]	varchar(1) COLLATE NOCASE,
	[FNAhmNumSize]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTAhmTblName], [FTAhmFedCode], [FTSatStaDocType], [FTAhmFmtAll])

);
CREATE TABLE IF NOT EXISTS [TCNTCstPoint] (
	[FTBchCode]	varchar(6) NOT NULL COLLATE NOCASE,
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPntRefDoc]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPntDate]	datetime,
	[FCPntOptBuyAmt]	real,
	[FCPntOptGetAmt]	real,
	[FNPntB4Bill]	integer,
	[FCPntBillAmt]	real,
	[FNPntBillQty]	integer,
	[FTPntExpired]	varchar(1) COLLATE NOCASE,
	[FTPntStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTPntCardType]	varchar(1) COLLATE NOCASE,
	[FTCptJDate]	varchar(6) COLLATE NOCASE,
	[FTCptTime]	varchar(8) COLLATE NOCASE,
	[FDPntSplStart]	datetime,
	[FDPntSplExpired]	datetime,
	[FTPntStaExpired]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCstCode], [FTPntRefDoc], [FTSplCode])

);
CREATE TABLE IF NOT EXISTS [TCNTCstPointTmp] (
	[FTBchCode]	varchar(6) NOT NULL COLLATE NOCASE,
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPntRefDoc]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTSplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPntDate]	datetime,
	[FCPntOptBuyAmt]	real,
	[FCPntOptGetAmt]	real,
	[FNPntB4Bill]	integer,
	[FCPntBillAmt]	real,
	[FNPntBillQty]	integer,
	[FTPntExpired]	varchar(1) COLLATE NOCASE,
	[FTPntStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTPntCardType]	varchar(1) COLLATE NOCASE,
	[FTCptJDate]	varchar(6) COLLATE NOCASE,
	[FTCptTime]	varchar(8) COLLATE NOCASE,
	[FDPntSplStart]	datetime,
	[FDPntSplExpired]	datetime,
	[FTPntStaExpired]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTDocDTDisTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTSessionID]	varchar(255) NOT NULL COLLATE NOCASE,
	[FDXtdDateIns]	datetime NOT NULL,
	[FNXtdStaDis]	integer,
	[FTXtdDisChgType]	varchar(10) COLLATE NOCASE,
	[FCXtdNet]	real,
	[FCXtdValue]	real,
	[FDLastUpdOn]	datetime,
	[FDCreateOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTXtdDisChgTxt]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo], [FTSessionID], [FDXtdDateIns])

);
CREATE TABLE IF NOT EXISTS [TCNTDocDTTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) COLLATE NOCASE,
	[FNXtdSeqNo]	integer,
	[FTXthDocKey]	varchar(20) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(255) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FTXtdBchRef]	varchar(5) COLLATE NOCASE,
	[FTXtdDocNoRef]	varchar(20) COLLATE NOCASE,
	[FCXtdPriceRet]	real,
	[FCXtdPriceWhs]	real,
	[FCXtdPriceNet]	real,
	[FTXtdShpTo]	varchar(5) COLLATE NOCASE,
	[FTXtdBchTo]	varchar(5) COLLATE NOCASE,
	[FTSrnCode]	varchar(50) COLLATE NOCASE,
	[FTXtdSaleType]	varchar(1) COLLATE NOCASE,
	[FCXtdSalePrice]	real,
	[FCXtdAmtB4DisChg]	real,
	[FTXtdDisChgTxt]	varchar(50) COLLATE NOCASE,
	[FCXtdDis]	real,
	[FCXtdChg]	real,
	[FCXtdNetAfHD]	real,
	[FCXtdWhtAmt]	real,
	[FTXtdWhtCode]	varchar(5) COLLATE NOCASE,
	[FCXtdWhtRate]	real,
	[FCXtdQtyLef]	real,
	[FCXtdQtyRfn]	real,
	[FTXtdStaAlwDis]	varchar(1) COLLATE NOCASE,
	[FTPdtName]	varchar(50) COLLATE NOCASE,
	[FCPdtUnitFact]	real,
	[FTPgpChain]	varchar(50) COLLATE NOCASE,
	[FNAjdLayRow]	float,
	[FNAjdLayCol]	float,
	[FCAjdWahB4Adj]	real,
	[FCAjdSaleB4AdjC1]	real,
	[FDAjdDateTimeC1]	datetime,
	[FCAjdUnitQtyC1]	real,
	[FCAjdQtyAllC1]	real,
	[FCAjdSaleB4AdjC2]	real,
	[FDAjdDateTimeC2]	datetime,
	[FCAjdUnitQtyC2]	real,
	[FCAjdQtyAllC2]	real,
	[FCAjdUnitQty]	real,
	[FDAjdDateTime]	datetime,
	[FCAjdQtyAll]	real,
	[FCAjdQtyAllDiff]	real,
	[FTAjdPlcCode]	varchar(5) COLLATE NOCASE,
	[FTSessionID]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FDCreateOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FNLayRowForTWXVD]	integer,
	[FNLayColForTWXVD]	integer,
	[FCLayColQtyMaxForTWXVD]	real,
	[FCStkQty]	real,
	[FCMaxTransferForTWXVD]	real,
	[FCUserInPutTransferForTWXVD]	real,
	[FTMerCodeForADJPL]	varchar(5) COLLATE NOCASE,
	[FTShpCodeForADJPL]	varchar(5) COLLATE NOCASE,
	[FTPzeCodeForADJPL]	varchar(5) COLLATE NOCASE,
	[FTRthCodeForADJPL]	varchar(5) COLLATE NOCASE,
	[FTSizNameForADJPL]	varchar(40) COLLATE NOCASE,
	[FTBchCodeForADJPL]	varchar(5) COLLATE NOCASE,
	[FNLayRowForADJSTKVD]	integer,
	[FNLayColForADJSTKVD]	integer,
	[FCLayColQtyMaxForADJSTKVD]	real,
	[FCUserInPutForADJSTKVD]	real,
	[FCDateTimeInputForADJSTKVD]	datetime,
	[FNCabSeqForTWXVD]	integer,
	[FTCabNameForTWXVD]	varchar(255) COLLATE NOCASE,
	[FTXthWhFrmForTWXVD]	varchar(5) COLLATE NOCASE,
	[FTXthWhToForTWXVD]	varchar(5) COLLATE NOCASE,
	[FTBddTypeForDeposit]	varchar(255) COLLATE NOCASE,
	[FTBddRefNoForDeposit]	varchar(20) COLLATE NOCASE,
	[FDBddRefDateForDeposit]	datetime,
	[FCBddRefAmtForDeposit]	real,
	[FTBddRefBnkNameForDeposit]	varchar(255) COLLATE NOCASE,
	[FTTmpRemark]	varchar(255) COLLATE NOCASE,
	[FTTmpStatus]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTDocHDDisTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDXtdDateIns]	datetime NOT NULL,
	[FTXtdDisChgTxt]	varchar(50) COLLATE NOCASE,
	[FTXtdDisChgType]	varchar(10) COLLATE NOCASE,
	[FCXtdTotalAfDisChg]	real,
	[FCXtdTotalB4DisChg]	real,
	[FCXtdDisChg]	real,
	[FCXtdAmt]	real,
	[FTSessionID]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FDCreateOn]	datetime,
	[FTLastUpdBy]	varchar(255) COLLATE NOCASE,
	[FTCreateBy]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FDXtdDateIns])

);
CREATE TABLE IF NOT EXISTS [TCNTMemAmtActive] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMemCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCTxnBuyTotal]	real,
	[FDTxnBuyLast]	datetime,
    PRIMARY KEY ([FTCgpCode], [FTMemCode])

);
CREATE TABLE IF NOT EXISTS [TCNTMemPntActive] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMemCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCTxnPntQty]	real,
	[FCTxnPnt2ExpYear]	real,
	[FDTxnPntLast]	datetime,
    PRIMARY KEY ([FTCgpCode], [FTMemCode])

);
CREATE TABLE IF NOT EXISTS [TCNTMemPntSplActive] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMemCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTTxnRefSpl]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDTxnPntStart]	datetime NOT NULL,
	[FCTxnPntQty]	real,
	[FDTxnPntLast]	datetime,
    PRIMARY KEY ([FTCgpCode], [FTMemCode], [FTTxnRefSpl])

);
CREATE TABLE IF NOT EXISTS [TCNTMemTxnRedeem] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMemCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTRedRefDoc]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTRedRefInt]	varchar(30) COLLATE NOCASE,
	[FTRedRefSpl]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDRedRefDate]	datetime,
	[FCRedPntB4Bill]	real,
	[FCRedPntBillQty]	real,
	[FTRedPntStaClosed]	varchar(1) COLLATE NOCASE,
	[FDRedPntStart]	datetime,
	[FDRedPntExpired]	datetime,
	[FTRedPntDocType]	varchar(1) COLLATE NOCASE,
	[FTRedStaSend]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode], [FTMemCode], [FTRedRefDoc])

);
CREATE TABLE IF NOT EXISTS [TCNTMemTxnSale] (
	[FTCgpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMemCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTTxnRefDoc]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTTxnRefInt]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTTxnRefSpl]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTTxnPntDocType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDTxnRefDate]	datetime,
	[FCTxnRefGrand]	real,
	[FCTxnPntOptBuyAmt]	real,
	[FCTxnPntOptGetQty]	real,
	[FCTxnPntB4Bill]	real,
	[FDTxnPntStart]	datetime,
	[FDTxnPntExpired]	datetime,
	[FCTxnPntBillQty]	real,
	[FCTxnPntUsed]	real,
	[FCTxnPntExpired]	real,
	[FTTxnPntStaClosed]	varchar(1) COLLATE NOCASE,
	[FTTxnStaSend]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCgpCode], [FTMemCode], [FTTxnRefDoc], [FTTxnRefInt], [FTTxnRefSpl])

);
CREATE TABLE IF NOT EXISTS [TCNTMenuFavorite] (
	[FTMfvID]	integer NOT NULL,
	[FTMnuCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuRoute]	varchar(250) COLLATE NOCASE,
	[FTMnuImgPath]	varchar(250) COLLATE NOCASE,
	[FTMfvOwner]	varchar(250) COLLATE NOCASE,
	[FTMfvName]	varchar(250) COLLATE NOCASE,
    PRIMARY KEY ([FTMfvID], [FTMnuCode])

);
CREATE TABLE IF NOT EXISTS [TCNTmpPrcStkCard] (
	[FTComName]	varchar(50) COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkDocNo]	varchar(20) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FCStkQty]	real,
	[FCStkSetPrice]	real,
	[FCStkCostIn]	real,
	[FCStkCostEx]	real,
	[FCStkVatable]	real,
	[FTStkStaSent]	varchar(1) COLLATE NOCASE,
	[FDDateUpd]	datetime,
	[FTTimeUpd]	varchar(10) COLLATE NOCASE,
	[FTWhoUpd]	varchar(50) COLLATE NOCASE,
	[FDDateIns]	datetime,
	[FTTimeIns]	varchar(10) COLLATE NOCASE,
	[FTWhoIns]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTMsgQueue] (
	[FNMsgID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTMsgQName]	varchar(50) COLLATE NOCASE,
	[FTMsgQData]	varchar(255) COLLATE NOCASE,
	[FTMsgStaActive]	varchar(1) COLLATE NOCASE,
	[FTMsgStaPrc]	integer,
	[FTMsgRemark]	varchar(30) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTMsgRemind] (
	[FNMsgID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTMsgStaRead]	varchar(1) COLLATE NOCASE,
	[FNMsgSeq]	integer,
	[FNMsgType]	integer,
	[FTMsgGroup]	varchar(50) COLLATE NOCASE,
	[FTMsgDocRef]	varchar(30) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTMsgRemind_L] (
	[FNMsgID]	integer NOT NULL,
	[FNLngID]	integer NOT NULL,
	[FTMsgData]	varchar(255) COLLATE NOCASE,
	[FTMsgRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FNMsgID], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjCostDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXchDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXcdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FCXcdFactor]	real,
	[FTPgpChain]	varchar(30) COLLATE NOCASE,
	[FTClrCode]	varchar(5) COLLATE NOCASE,
	[FTPszCode]	varchar(5) COLLATE NOCASE,
	[FTDcsCode]	varchar(30) COLLATE NOCASE,
	[FTXcdBarScan]	varchar(25) COLLATE NOCASE,
	[FCXcdCostOld]	real,
	[FCXcdCostNew]	real,
	[FCXcdDiff]	real,
	[FCXcdRefDecQty]	real,
	[FCXcdRefDecQtyAll]	real,
	[FCXcdRefDecAmt]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXchDocNo], [FNXcdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjCostHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXchDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXchDocType]	integer,
	[FDXchDocDate]	datetime,
	[FTXchDocTime]	varchar(8) COLLATE NOCASE,
	[FDXchAffect]	datetime,
	[FTAggCode]	varchar(5) COLLATE NOCASE,
	[FTXchZneTo]	varchar(30) COLLATE NOCASE,
	[FTXchBchTo]	varchar(5) COLLATE NOCASE,
	[FTXchDocType]	varchar(1) COLLATE NOCASE,
	[FTXchRefInt]	varchar(20) COLLATE NOCASE,
	[FDXchRefIntDate]	datetime,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTXchApvCode]	varchar(20) COLLATE NOCASE,
	[FCXchTotal]	real,
	[FTXchStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXchStaApv]	varchar(1) COLLATE NOCASE,
	[FTXchStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FNXchStaDocAct]	integer,
	[FTXchRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXchDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjPriBch_] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXpbBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXphDocNo], [FTXpbBchTo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjPriRT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXpdSeq]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPzeCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTRthCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXpdShpTo]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXphDocNo], [FNXpdSeq])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjStkDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAjhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNAjdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtName]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FTAjdBarcode]	varchar(25) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FCPdtUnitFact]	real,
	[FTPgpChain]	varchar(30) COLLATE NOCASE,
	[FTAjdPlcCode]	varchar(5) COLLATE NOCASE,
	[FNCabSeq]	integer,
	[FNAjdLayRow]	integer NOT NULL,
	[FNAjdLayCol]	integer NOT NULL,
	[FCAjdWahB4Adj]	real,
	[FCAjdSaleB4AdjC1]	real,
	[FDAjdDateTimeC1]	datetime,
	[FCAjdUnitQtyC1]	real,
	[FCAjdQtyAllC1]	real,
	[FCAjdSaleB4AdjC2]	real,
	[FDAjdDateTimeC2]	datetime,
	[FCAjdUnitQtyC2]	real,
	[FCAjdQtyAllC2]	real,
	[FCAjdUnitQty]	real,
	[FDAjdDateTime]	datetime,
	[FCAjdQtyAll]	real,
	[FCAjdQtyAllDiff]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTAjhDocNo], [FNAjdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtAdjStkHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAjhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNAjhDocType]	integer,
	[FTAjhDocType]	varchar(1) COLLATE NOCASE,
	[FDAjhDocDate]	datetime,
	[FTAjhBchTo]	varchar(5) COLLATE NOCASE,
	[FTAjhMerchantTo]	varchar(10) COLLATE NOCASE,
	[FTAjhShopTo]	varchar(5) COLLATE NOCASE,
	[FTAjhPosTo]	varchar(5) COLLATE NOCASE,
	[FTAjhWhTo]	varchar(5) COLLATE NOCASE,
	[FTAjhPlcCode]	varchar(5) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FTAjhRmk]	varchar(200) COLLATE NOCASE,
	[FNAjhDocPrint]	integer,
	[FTAjhApvSeqChk]	varchar(1) COLLATE NOCASE,
	[FTAjhApvCode]	varchar(20) COLLATE NOCASE,
	[FTAjhStaApv]	varchar(1) COLLATE NOCASE,
	[FTAjhStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTAjhStaDoc]	varchar(1) COLLATE NOCASE,
	[FNAjhStaDocAct]	integer,
	[FTAjhStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FTAjhDocRef]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTAjhDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtIntDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXthDocType]	integer NOT NULL,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTXthWahTo]	varchar(255) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FCXtdQty]	real,
	[FCXtdQtyRcv]	real,
	[FCXtdQtyAll]	real,
	[FTXtdRvtRef]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtIntDTBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FNXthDocType]	integer NOT NULL,
	[FTXthBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthWahTo]	varchar(255) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FCXtdQty]	real,
	[FCXtdQtyRcv]	real,
	[FCXtdQtyAll]	real,
	[FTXtdRvtRef]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtLocSeq] (
	[FTPlcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBarCode]	varchar(25) NOT NULL COLLATE NOCASE,
	[FNPldSeq]	integer,
	[FTPdtCapacity]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPlcCode], [FTBarCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtCB] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPbySeq]	integer NOT NULL,
	[FTPmdGrpName]	varchar(50) COLLATE NOCASE,
	[FTPbyStaCalSum]	varchar(1) COLLATE NOCASE,
	[FTPbyStaBuyCond]	varchar(1) COLLATE NOCASE,
	[FTPbyStaPdtDT]	varchar(1) COLLATE NOCASE,
	[FCPbyPerAvgDis]	real,
	[FCPbyMinSetPri]	real,
	[FCPbyMinValue]	real,
	[FCPbyMaxValue]	real,
	[FTPbyMinTime]	varchar(10) COLLATE NOCASE,
	[FTPbyMaxTime]	varchar(10) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNPbySeq])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtCBTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPbySeq]	integer NOT NULL,
	[FTPmdGrpName]	varchar(50) COLLATE NOCASE,
	[FTPbyStaCalSum]	varchar(1) COLLATE NOCASE,
	[FTPbyStaBuyCond]	varchar(1) COLLATE NOCASE,
	[FTPbyStaPdtDT]	varchar(1) COLLATE NOCASE,
	[FCPbyPerAvgDis]	real,
	[FCPbyMinSetPri]	real,
	[FCPbyMinValue]	real,
	[FCPbyMaxValue]	real,
	[FTPbyMinTime]	varchar(10) COLLATE NOCASE,
	[FTPbyMaxTime]	varchar(10) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtCGTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPgtSeq]	integer NOT NULL,
	[FTPmdGrpName]	varchar(50) COLLATE NOCASE,
	[FTPgtStaGetEffect]	varchar(1) COLLATE NOCASE,
	[FTPgtStaGetType]	varchar(1) COLLATE NOCASE,
	[FTPgtStaGetPdt]	varchar(1) COLLATE NOCASE,
	[FTRolCode]	varchar(5) COLLATE NOCASE,
	[FCPgtGetvalue]	real,
	[FTPplCode]	varchar(20) COLLATE NOCASE,
	[FCPgtGetQty]	real,
	[FCPgtPerAvgDis]	real,
	[FTPgtStaPoint]	varchar(1) COLLATE NOCASE,
	[FTPgtStaPntCalType]	varchar(1) COLLATE NOCASE,
	[FTPgtStaPdtDT]	varchar(1) COLLATE NOCASE,
	[FNPgtPntGet]	integer,
	[FNPgtPntBuy]	integer,
	[FTPgtStaCoupon]	varchar(1) COLLATE NOCASE,
	[FTPgtCpnText]	varchar(50) COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmdSeq]	integer NOT NULL,
	[FTPmdStaType]	varchar(1) COLLATE NOCASE,
	[FTPmdGrpName]	varchar(50) COLLATE NOCASE,
	[FTPmdRefCode]	varchar(20) COLLATE NOCASE,
	[FTPmdSubRef]	varchar(5) COLLATE NOCASE,
	[FTPmdBarCode]	varchar(25) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNPmdSeq])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtDTTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmdSeq]	integer NOT NULL,
	[FTPmdStaType]	varchar(1) COLLATE NOCASE,
	[FTPmdGrpName]	varchar(50) COLLATE NOCASE,
	[FTPmdRefCode]	varchar(20) COLLATE NOCASE,
	[FTPmdSubRef]	varchar(5) COLLATE NOCASE,
	[FTPmdBarCode]	varchar(25) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPmhDStart]	datetime,
	[FDPmhDStop]	datetime,
	[FDPmhTStart]	datetime,
	[FDPmhTStop]	datetime,
	[FTPmhStaLimitCst]	varchar(1) COLLATE NOCASE,
	[FTPmhStaClosed]	varchar(1) COLLATE NOCASE,
	[FTPmhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhStaApv]	varchar(1) COLLATE NOCASE,
	[FTPmhStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FNPmhStaDocAct]	integer,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTPmhUsrApv]	varchar(20) COLLATE NOCASE,
	[FTPmhStaOnTopPmt]	varchar(1) COLLATE NOCASE,
	[FTPmhStaAlwCalPntStd]	varchar(1) COLLATE NOCASE,
	[FTPmhStaRcvFree]	varchar(1) COLLATE NOCASE,
	[FTPmhStaLimitGet]	varchar(1) COLLATE NOCASE,
	[FTPmhStaLimitTime]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGetPdt]	varchar(1) COLLATE NOCASE,
	[FTPmhRefAccCode]	varchar(20) COLLATE NOCASE,
	[FTPmhStaPdtExc]	varchar(1) COLLATE NOCASE,
	[FTRolCode]	varchar(5) COLLATE NOCASE,
	[FNPmhLimitQty]	integer,
	[FTPmhStaChkLimit]	varchar(1) COLLATE NOCASE,
	[FTPmhStaChkCst]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGrpPriority]	varchar(1) COLLATE NOCASE,
	[FTPmhStaChkQuota]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGetPri]	varchar(1) COLLATE NOCASE,
	[FTPmhStaOnTopDis]	varchar(1) COLLATE NOCASE,
	[FTPmhStaSpcGrpDis]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHD_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmhName]	varchar(200) COLLATE NOCASE,
	[FTPmhNameSlip]	varchar(25) COLLATE NOCASE,
	[FTPmhRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHD_LTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmhName]	varchar(200) COLLATE NOCASE,
	[FTPmhNameSlip]	varchar(25) COLLATE NOCASE,
	[FTPmhRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPmhShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE, FTPmhAgnTo VARCHAR(5),
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FTPmhBchTo], [FTPmhMerTo], [FTPmhShpTo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDBchTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPmhShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDCst] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSpmStaLimitCst]	varchar(1) COLLATE NOCASE,
	[FNSpmMemAgeLT]	varchar(255) COLLATE NOCASE,
	[FTSpmStaChkCstDOB]	varchar(1) COLLATE NOCASE,
	[FNPmhCstDobPrev]	integer,
	[FNPmhCstDobNext]	integer,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDCstPri] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FTPplCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDCstPriTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDCstTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSpmStaLimitCst]	varchar(1) COLLATE NOCASE,
	[FNSpmMemAgeLT]	varchar(255) COLLATE NOCASE,
	[FTSpmStaChkCstDOB]	varchar(1) COLLATE NOCASE,
	[FNPmhCstDobPrev]	integer,
	[FNPmhCstDobNext]	integer

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPmtHDTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPmhDStart]	datetime,
	[FDPmhDStop]	datetime,
	[FDPmhTStart]	datetime,
	[FDPmhTStop]	datetime,
	[FTPmhStaLimitCst]	varchar(1) COLLATE NOCASE,
	[FTPmhStaClosed]	varchar(1) COLLATE NOCASE,
	[FTPmhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhStaApv]	varchar(1) COLLATE NOCASE,
	[FTPmhStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FNPmhStaDocAct]	integer,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTPmhUsrApv]	varchar(20) COLLATE NOCASE,
	[FTPmhStaOnTopPmt]	varchar(1) COLLATE NOCASE,
	[FTPmhStaAlwCalPntStd]	varchar(1) COLLATE NOCASE,
	[FTPmhStaRcvFree]	varchar(1) COLLATE NOCASE,
	[FTPmhStaLimitGet]	varchar(1) COLLATE NOCASE,
	[FTPmhStaLimitTime]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGetPdt]	varchar(1) COLLATE NOCASE,
	[FTPmhRefAccCode]	varchar(20) COLLATE NOCASE,
	[FTPmhStaPdtExc]	varchar(1) COLLATE NOCASE,
	[FTRolCode]	varchar(5) COLLATE NOCASE,
	[FNPmhLimitQty]	integer,
	[FTPmhStaChkLimit]	varchar(1) COLLATE NOCASE,
	[FTPmhStaChkCst]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGrpPriority]	varchar(1) COLLATE NOCASE,
	[FTPmhStaChkQuota]	varchar(1) COLLATE NOCASE,
	[FTPmhStaGetPri]	varchar(1) COLLATE NOCASE,
	[FTPmhStaOnTopDis]	varchar(1) COLLATE NOCASE,
	[FTPmhStaSpcGrpDis]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPrice4CST] (
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDPghDStart]	datetime NOT NULL,
	[FTPghTStart]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDPghDStop]	datetime,
	[FTPghTStop]	varchar(10) COLLATE NOCASE,
	[FTPghDocNo]	varchar(20) COLLATE NOCASE,
	[FTPghDocType]	varchar(1) COLLATE NOCASE,
	[FTPghStaAdj]	varchar(1) COLLATE NOCASE,
	[FCPgdPriceRet]	real,
	[FCPgdPriceWhs]	real,
	[FCPgdPriceNet]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPplCode], [FTPdtCode], [FTPunCode], [FDPghDStart], [FTPghTStart])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtPrice4PDTTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDPghDStart]	datetime NOT NULL,
	[FTPghTStart]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPghDStop]	datetime,
	[FTPghTStop]	varchar(10) COLLATE NOCASE,
	[FTPghDocNo]	varchar(20) COLLATE NOCASE,
	[FTPghDocType]	varchar(1) COLLATE NOCASE,
	[FTPghStaAdj]	varchar(1) COLLATE NOCASE,
	[FCPgdPriceRet]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtReqTnfDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FTXtdSaleType]	varchar(1) COLLATE NOCASE,
	[FCXtdSalePrice]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdDisChgAvi]	real,
	[FTXtdDisChgTxt]	varchar(50) COLLATE NOCASE,
	[FCXtdDis]	real,
	[FCXtdChg]	real,
	[FCXtdNet]	real,
	[FCXtdNetAfHD]	real,
	[FCXtdNetEx]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdWhtAmt]	real,
	[FTXtdWhtCode]	varchar(5) COLLATE NOCASE,
	[FCXtdWhtRate]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPdt]	varchar(1) COLLATE NOCASE,
	[FCXtdQtyLef]	real,
	[FCXtdQtyRfn]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXtdStaAlwDis]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtReqTnfHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTXthTnfType]	varchar(1) COLLATE NOCASE,
	[FNXthDocType]	integer,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTXthCshOrCrd]	varchar(1) COLLATE NOCASE,
	[FTXthOther]	varchar(100) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthBchFrm]	varchar(5) COLLATE NOCASE,
	[FTXthBchTo]	varchar(5) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTSplCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FTRteCode]	varchar(5) COLLATE NOCASE,
	[FCXthRteFac]	real,
	[FCXthTotal]	real,
	[FCXtVatNoDisChg]	real,
	[FCXthNoVatNoDisChg]	real,
	[FCXthVatDisChgAvi]	real,
	[FCXthNoVatDisChgAvi]	real,
	[FTXthDisChgTxt]	varchar(50) COLLATE NOCASE,
	[FCXthDis]	real,
	[FCXthChg]	real,
	[FCXthRefAEAmt]	real,
	[FCXthVatAfDisChg]	real,
	[FCXthNoVatAfDisChg]	real,
	[FCXthAfDisChgAE]	real,
	[FTXthWpCode]	varchar(5) COLLATE NOCASE,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FCXthGrandB4Wht]	real,
	[FCXthWpTax]	real,
	[FCXthGrand]	real,
	[FCXthRnd]	real,
	[FTXthGndText]	varchar(200) COLLATE NOCASE,
	[FCXthPaid]	real,
	[FCXthLeft]	real,
	[FTXthStaRefund]	varchar(1) COLLATE NOCASE,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcDoc]	varchar(255) COLLATE NOCASE,
	[FTXthStaPaid]	varchar(1) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtSerial] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSrnCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTSrnStaNow]	varchar(1) COLLATE NOCASE,
	[FTSrnStaBook]	varchar(1) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTSrnDocNo]	varchar(20) COLLATE NOCASE,
	[FTSrnDocType]	varchar(2) COLLATE NOCASE,
	[FTSrnBchFrom]	varchar(5) COLLATE NOCASE,
	[FTSrnBchTo]	varchar(5) COLLATE NOCASE,
	[FDSrnChgDate]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FTSrnCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtSerialSet] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSrnCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTPdtCodeSet]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSrnCodeSet]	varchar(50) NOT NULL COLLATE NOCASE,
	[FNPsdSeqNo]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FTSrnCode], [FTPdtCodeSet], [FTSrnCodeSet])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtSet] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCodeSet]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FCXsdFactor]	real,
	[FCPstQty]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FTPdtCodeSet])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtSetTmp] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCodeSet]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FCXsdFactor]	real,
	[FCPstQty]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkBal] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCStkQty]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTWahCode], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkBalBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FCStkQty]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTWahCode], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkCrd] (
	[FNStkCrdID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkDocNo]	varchar(20) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FCStkQty]	real,
	[FCStkSetPrice]	real,
	[FCStkCostIn]	real,
	[FCStkCostEx]	real,
	[FTPdtParent]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkCrdBch] (
	[FNStkCrdID]	integer NOT NULL,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkDocNo]	varchar(20) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FCStkQty]	real,
	[FCStkSetPrice]	real,
	[FCStkCostIn]	real,
	[FCStkCostEx]	real,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FNStkCrdID], [FTBchCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkCrdME] (
	[FNStkCrdID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkDocNo]	varchar(20) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FCStkQty]	real,
	[FCStkSetPrice]	real,
	[FCStkCostIn]	real,
	[FCStkCostEx]	real,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkCrdMEBch] (
	[FNStkCrdID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkDocNo]	varchar(20) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FCStkQty]	real,
	[FCStkSetPrice]	real,
	[FCStkCostIn]	real,
	[FCStkCostEx]	real,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTPdtStkQtyUnApv] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTStkDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNStkDocType]	integer NOT NULL,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCStkQty]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTStkDocNo], [FNStkDocType], [FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbiDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FTXtdBchRef]	varchar(5) COLLATE NOCASE,
	[FTXtdDocNoRef]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbiHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXthDocType]	integer,
	[FTXthRsnType]	varchar(1) COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthBchFrm]	varchar(5) COLLATE NOCASE,
	[FTXthBchTo]	varchar(5) COLLATE NOCASE,
	[FTXthMerchantFrm]	varchar(10) COLLATE NOCASE,
	[FTXthMerchantTo]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTSplCode]	varchar(20) COLLATE NOCASE,
	[FTXthOther]	varchar(100) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbiHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTboDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTboHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthBchFrm]	varchar(5) COLLATE NOCASE,
	[FTXthBchTo]	varchar(5) COLLATE NOCASE,
	[FTXthMerchantFrm]	varchar(10) COLLATE NOCASE,
	[FTXthMerchantTo]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTboHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbxDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbxHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthBchFrm]	varchar(5) COLLATE NOCASE,
	[FTXthBchTo]	varchar(5) COLLATE NOCASE,
	[FTXthMerchantFrm]	varchar(10) COLLATE NOCASE,
	[FTXthMerchantTo]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(255) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTbxHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwiDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FTXtdBchRef]	varchar(5) COLLATE NOCASE,
	[FTXtdDocNoRef]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwiDTVD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FNCabSeq]	integer NOT NULL,
	[FNLayRow]	integer NOT NULL,
	[FNLayCol]	integer NOT NULL,
	[FTXtdBarcode]	varchar(25) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FCXtdQty]	real,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo], [FNCabSeq], [FNLayRow], [FNLayCol])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwiHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXthDocType]	integer,
	[FTXthRsnType]	varchar(1) COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthMerCode]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTXthPosFrm]	varchar(5) COLLATE NOCASE,
	[FTXthPosTo]	varchar(5) COLLATE NOCASE,
	[FTSplCode]	varchar(20) COLLATE NOCASE,
	[FTXthOther]	varchar(100) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FTXthTypRefFrm]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwiHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwoDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwoDTVD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FNCabSeq]	integer NOT NULL,
	[FNLayRow]	integer NOT NULL,
	[FNLayCol]	integer NOT NULL,
	[FTXtdBarcode]	varchar(25) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FCXtdQty]	real,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo], [FNCabSeq], [FNLayRow], [FNLayCol])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwoHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXthDocType]	integer,
	[FTXthRsnType]	varchar(1) COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthMerCode]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTXthPosFrm]	varchar(5) COLLATE NOCASE,
	[FTXthPosTo]	varchar(5) COLLATE NOCASE,
	[FTSplCode]	varchar(20) COLLATE NOCASE,
	[FTXthOther]	varchar(100) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwoHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwxDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXtdSeqNo]	integer NOT NULL,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTXtdPdtName]	varchar(100) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FCXtdFactor]	real,
	[FTXtdBarCode]	varchar(25) COLLATE NOCASE,
	[FTXtdVatType]	varchar(1) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FCXtdVatRate]	real,
	[FCXtdQty]	real,
	[FCXtdQtyAll]	real,
	[FCXtdSetPrice]	real,
	[FCXtdAmt]	real,
	[FCXtdVat]	real,
	[FCXtdVatable]	real,
	[FCXtdNet]	real,
	[FCXtdCostIn]	real,
	[FCXtdCostEx]	real,
	[FTXtdStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FNXtdPdtLevel]	integer,
	[FTXtdPdtParent]	varchar(20) COLLATE NOCASE,
	[FCXtdQtySet]	real,
	[FTXtdPdtStaSet]	varchar(1) COLLATE NOCASE,
	[FTXtdRmk]	varchar(200) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo], [FNXtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwxHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDXthDocDate]	datetime,
	[FTXthVATInOrEx]	varchar(1) COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTXthMerCode]	varchar(10) COLLATE NOCASE,
	[FTXthShopFrm]	varchar(5) COLLATE NOCASE,
	[FTXthShopTo]	varchar(5) COLLATE NOCASE,
	[FTXthWhFrm]	varchar(5) COLLATE NOCASE,
	[FTXthWhTo]	varchar(5) COLLATE NOCASE,
	[FTXthPosFrm]	varchar(5) COLLATE NOCASE,
	[FTXthPosTo]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXthApvCode]	varchar(20) COLLATE NOCASE,
	[FTXthRefExt]	varchar(20) COLLATE NOCASE,
	[FDXthRefExtDate]	datetime,
	[FTXthRefInt]	varchar(20) COLLATE NOCASE,
	[FDXthRefIntDate]	datetime,
	[FNXthDocPrint]	integer,
	[FCXthTotal]	real,
	[FCXthVat]	real,
	[FCXthVatable]	real,
	[FTXthRmk]	varchar(200) COLLATE NOCASE,
	[FTXthStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXthStaApv]	varchar(1) COLLATE NOCASE,
	[FTXthStaPrcStk]	varchar(1) COLLATE NOCASE,
	[FTXthStaDelMQ]	varchar(255) COLLATE NOCASE,
	[FNXthStaDocAct]	integer,
	[FNXthStaRef]	integer,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTPdtTwxHDRef] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXthCtrName]	varchar(100) COLLATE NOCASE,
	[FDXthTnfDate]	datetime,
	[FTXthRefTnfID]	varchar(20) COLLATE NOCASE,
	[FTXthRefVehID]	varchar(50) COLLATE NOCASE,
	[FTXthQtyAndTypeUnit]	varchar(30) COLLATE NOCASE,
	[FNXthShipAdd]	integer,
	[FTViaCode]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXthDocNo])

);
CREATE TABLE IF NOT EXISTS [TCNTSpnGroup] (
	[FTSpnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTSpnStaShop]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FDSpnStart]	datetime,
	[FDSpnStop]	datetime,
    PRIMARY KEY ([FTSpnCode], [FTBchCode], [FTSpnStaShop])

);
CREATE TABLE IF NOT EXISTS [TCNTStkFiFoIn] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTStkDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkTime]	varchar(10) COLLATE NOCASE,
	[FCStkQtyAll]	real,
	[FCStkQtyUsed]	real,
	[FCStkQtyAvai]	real,
	[FCStkAmtPerUnit]	real,
	[FCStkTotVatPerUnit]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTStkDocNo], [FTPdtCode], [FTWahCode])

);
CREATE TABLE IF NOT EXISTS [TCNTStkFiFoOut] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTStkDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTWahCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNStkSeqNo]	integer NOT NULL,
	[FTStkType]	varchar(1) COLLATE NOCASE,
	[FDStkDate]	datetime,
	[FTStkTime]	varchar(10) COLLATE NOCASE,
	[FCStkQtyAll]	real,
	[FCStkQtyUsed]	real,
	[FTStkDocRef]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTStkDocNo], [FTPdtCode], [FTWahCode], [FNStkSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTSync2BchHis] (
	[FNSynSeqNo]	integer NOT NULL,
	[FDSynDate]	datetime,
	[FTSynBchFrm]	varchar(5) COLLATE NOCASE,
	[FTSynBchTo]	varchar(5) COLLATE NOCASE,
	[FTSynFileName]	varchar(255) COLLATE NOCASE,
	[FTSynFileURL]	varchar(255) COLLATE NOCASE,
	[FDSynRcvDate]	datetime,
	[FTSynStatus]	varchar(1) COLLATE NOCASE,
	[FTSynRmk]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FNSynSeqNo])

);
CREATE TABLE IF NOT EXISTS [TCNTTblFmtKey] (
	[FTSatTblName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatFedCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatStaDocType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTSatGroup]	varchar(1) COLLATE NOCASE,
	[FTSatDocTypeName]	varchar(50) COLLATE NOCASE,
	[FNSatFedSize]	integer,
	[FTSatDefChar]	varchar(50) COLLATE NOCASE,
	[FTSatDefYear]	varchar(50) COLLATE NOCASE,
	[FTSatDefMonth]	varchar(50) COLLATE NOCASE,
	[FTSatDefDay]	varchar(50) COLLATE NOCASE,
	[FTSatDefNum]	varchar(50) COLLATE NOCASE,
	[FTSatDefFmtAll]	varchar(50) COLLATE NOCASE,
	[FTSatStaDefUsage]	varchar(1) COLLATE NOCASE,
	[FTSatUsrFmtChar]	varchar(50) COLLATE NOCASE,
	[FTSatUsrStaBch]	varchar(1) COLLATE NOCASE,
	[FTSatUsrFmtYear]	varchar(50) COLLATE NOCASE,
	[FTSatUsrFmtMonth]	varchar(50) COLLATE NOCASE,
	[FTSatUsrFmtDay]	varchar(50) COLLATE NOCASE,
	[FTSatUsrFmtNum]	varchar(50) COLLATE NOCASE,
	[FTSatUsrFmtAll]	varchar(50) COLLATE NOCASE,
	[FTSatStaResetBill]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSatTblName], [FTSatFedCode], [FTSatStaDocType])

);
CREATE TABLE IF NOT EXISTS [TCNTTblFmtKey_L] (
	[FTSatTblName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatFedCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FTSatStaDocType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSatTblDesc]	varchar(255) COLLATE NOCASE,
	[FTSatRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTSatTblName], [FTSatFedCode], [FTSatStaDocType], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TCNTTmpLogChg] (
	[FTLogCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTLogDocNo]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLogType]	integer NOT NULL,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FTLogStaPrc]	varchar(2) COLLATE NOCASE,
	[FDCreateOn]	datetime NOT NULL,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTLogCode], [FTLogDocNo], [FNLogType], [FDCreateOn])

);
CREATE TABLE IF NOT EXISTS [TCNTUrlObject] (
	[FNUrlID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTUrlRefID]	varchar(30) COLLATE NOCASE,
	[FNUrlSeq]	integer,
	[FNUrlType]	integer,
	[FTUrlTable]	varchar(50) COLLATE NOCASE,
	[FTUrlKey]	varchar(20) COLLATE NOCASE,
	[FTUrlAddress]	varchar(255) COLLATE NOCASE,
	[FTUrlPort]	varchar(20) COLLATE NOCASE,
	[FTUrlLogo]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTUrlObjectLogin] (
	[FTUrlRefID]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTUrlAddress]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTUolVhost]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTUolUser]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUolPassword]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUolKey]	varchar(255) COLLATE NOCASE,
	[FTUolStaActive]	varchar(1) COLLATE NOCASE,
	[FTUolgRmk]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTUrlRefID], [FTUrlAddress], [FTUolVhost], [FTUolUser])

);
CREATE TABLE IF NOT EXISTS [TCNTUrlObjectLoginTmp] (
	[FTUrlRefID]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTUrlAddress]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTUolVhost]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTUolUser]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUolPassword]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTUolKey]	varchar(255) COLLATE NOCASE,
	[FTUolStaActive]	varchar(1) COLLATE NOCASE,
	[FTUolgRmk]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTUrlObjectTmp] (
	[FNUrlID]	integer NOT NULL,
	[FTUrlRefID]	varchar(30) COLLATE NOCASE,
	[FNUrlSeq]	integer,
	[FNUrlType]	integer,
	[FTUrlTable]	varchar(50) COLLATE NOCASE,
	[FTUrlKey]	varchar(20) COLLATE NOCASE,
	[FTUrlAddress]	varchar(255) COLLATE NOCASE,
	[FTUrlPort]	varchar(20) COLLATE NOCASE,
	[FTUrlLogo]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTUsrBch] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTUsrCode], [FTBchCode], [FTShpCode], [FTMerCode])

);
CREATE TABLE IF NOT EXISTS [TCNTUsrFuncRpt] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTUfrType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTUfrGrpRef]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTUfrRef]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTGhdApp]	varchar(50) COLLATE NOCASE,
	[FTUfrStaAlw]	varchar(1) COLLATE NOCASE,
	[FTUfrStaFavorite]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode], [FTUfrType], [FTUfrGrpRef], [FTUfrRef])

);
CREATE TABLE IF NOT EXISTS [TCNTUsrFuncRptTmp] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTUfrType]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTUfrGrpRef]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTUfrRef]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTGhdApp]	varchar(50) COLLATE NOCASE,
	[FTUfrStaAlw]	varchar(1) COLLATE NOCASE,
	[FTUfrStaFavorite]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTUsrGroup] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTUsrCode], [FTBchCode], [FTShpCode], [FTMerCode], [FTAgnCode])

);
CREATE TABLE IF NOT EXISTS [TCNTUsrGroupTmp] (
	[FTUsrCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTUsrMenu] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTGmnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuParent]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAutStaFull]	varchar(1) COLLATE NOCASE,
	[FTAutStaRead]	varchar(1) COLLATE NOCASE,
	[FTAutStaAdd]	varchar(1) COLLATE NOCASE,
	[FTAutStaEdit]	varchar(1) COLLATE NOCASE,
	[FTAutStaDelete]	varchar(1) COLLATE NOCASE,
	[FTAutStaCancel]	varchar(1) COLLATE NOCASE,
	[FTAutStaAppv]	varchar(1) COLLATE NOCASE,
	[FTAutStaPrint]	varchar(1) COLLATE NOCASE,
	[FTAutStaPrintMore]	varchar(1) COLLATE NOCASE,
	[FTAutStaFavorite]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRolCode], [FTGmnCode], [FTMnuParent], [FTMnuCode])

);
CREATE TABLE IF NOT EXISTS [TCNTUsrMenuTmp] (
	[FTRolCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTGmnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuParent]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAutStaFull]	varchar(1) COLLATE NOCASE,
	[FTAutStaRead]	varchar(1) COLLATE NOCASE,
	[FTAutStaAdd]	varchar(1) COLLATE NOCASE,
	[FTAutStaEdit]	varchar(1) COLLATE NOCASE,
	[FTAutStaDelete]	varchar(1) COLLATE NOCASE,
	[FTAutStaCancel]	varchar(1) COLLATE NOCASE,
	[FTAutStaAppv]	varchar(1) COLLATE NOCASE,
	[FTAutStaPrint]	varchar(1) COLLATE NOCASE,
	[FTAutStaPrintMore]	varchar(1) COLLATE NOCASE,
	[FTAutStaFavorite]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TCNTWorkTime] (
	[FNWrtID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FDWrtDate]	datetime,
	[FTWrtClockIn]	varchar(20) COLLATE NOCASE,
	[FDWrtDateOut]	datetime,
	[FTWrtClockOut]	varchar(50) COLLATE NOCASE,
	[FTWrtApvCode]	varchar(20) COLLATE NOCASE,
	[FTWrtRemark]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMBank] (
	[FTBnkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBnkCode])

);
CREATE TABLE IF NOT EXISTS [TFNMBank_L] (
	[FTBnkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBnkName]	varchar(100) COLLATE NOCASE,
	[FTBnkRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTBnkCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMBankNote] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntStaShw]	varchar(1) COLLATE NOCASE,
	[FCBntRateAmt]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTRteCode], [FTBntCode])

);
CREATE TABLE IF NOT EXISTS [TFNMBankNote_L] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBntName]	varchar(100) COLLATE NOCASE,
	[FTBntRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTRteCode], [FTBntCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMBankNoteTmp] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntStaShw]	varchar(1) COLLATE NOCASE,
	[FCBntRateAmt]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMBankNoteTmp_L] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBntCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBntName]	varchar(100) COLLATE NOCASE,
	[FTBntRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMBankTmp] (
	[FTBnkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMBankTmp_L] (
	[FTBnkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBnkName]	varchar(100) COLLATE NOCASE,
	[FTBnkRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMBnkDepType] (
	[FTBdtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBdtCode])

);
CREATE TABLE IF NOT EXISTS [TFNMBnkDepType_L] (
	[FTBdtCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBdtName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTBdtCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMBookBank] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTBbkType]	varchar(1) COLLATE NOCASE,
	[FTBbkAccNo]	varchar(20) COLLATE NOCASE,
	[FTBnkCode]	varchar(5) COLLATE NOCASE,
	[FDBbkOpen]	datetime,
	[FCBbkBalance]	real,
	[FDBbkUpd]	datetime,
	[FTBbkStaActive]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTBbkCode])

);
CREATE TABLE IF NOT EXISTS [TFNMBookBank_L] (
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTBbkName]	varchar(100) COLLATE NOCASE,
	[FTBbkBranch]	varchar(50) COLLATE NOCASE,
	[FTBbkRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTBbkCode], [FTBchCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMBookCheque] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTChqCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNChqMin]	integer,
	[FNChqMax]	integer,
	[FTChqStaAct]	varchar(1) COLLATE NOCASE,
	[FTChqStaPrcDoc]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTChqCode])

);
CREATE TABLE IF NOT EXISTS [TFNMBookCheque_L] (
	[FTChqCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTChqName]	varchar(100) COLLATE NOCASE,
	[FTChqRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTChqCode], [FTBchCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCard] (
	[FTCrdCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDCrdStartDate]	datetime,
	[FDCrdExpireDate]	datetime,
	[FDCrdResetDate]	datetime,
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTDptCode]	varchar(5) COLLATE NOCASE,
	[FTCrdHolderID]	varchar(30) COLLATE NOCASE,
	[FTCrdRefID]	varchar(30) COLLATE NOCASE,
	[FTCrdStaShift]	varchar(1) COLLATE NOCASE,
	[FTCrdStaActive]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCard_L] (
	[FTCrdCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCrdName]	varchar(200) COLLATE NOCASE,
	[FTCrdRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCardBal] (
	[FTCrdCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTCrdTxnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FCCrdValue]	real,
	[FDLastUpdOn]	datetime,
    PRIMARY KEY ([FTCrdCode], [FTCrdTxnCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCardMan] (
	[FTCrdCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTCrdRefCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTCrdTable]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FTCrdRefCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCardType] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCtyStaShift]	varchar(1) COLLATE NOCASE,
	[FTCtyStaAlwRet]	varchar(255) COLLATE NOCASE,
	[FCCtyDeposit]	real,
	[FNCtyExpirePeriod]	integer,
	[FNCtyExpiredType]	integer,
	[FCCtyTopupAuto]	real,
	[FTCtyStaPay]	varchar(255) COLLATE NOCASE,
	[FCCtyCreditLimit]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCardType_L] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCtyName]	varchar(100) COLLATE NOCASE,
	[FTCtyRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCtyCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCardTypeTmp] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCtyStaShift]	varchar(1) COLLATE NOCASE,
	[FTCtyStaAlwRet]	varchar(255) COLLATE NOCASE,
	[FCCtyDeposit]	real,
	[FNCtyExpirePeriod]	integer,
	[FNCtyExpiredType]	integer,
	[FCCtyTopupAuto]	real,
	[FTCtyStaPay]	varchar(255) COLLATE NOCASE,
	[FCCtyCreditLimit]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMCardTypeTmp_L] (
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCtyName]	varchar(100) COLLATE NOCASE,
	[FTCtyRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMCouponType] (
	[FTCptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCptType]	varchar(1) COLLATE NOCASE,
	[FTCptStaChk]	varchar(1) COLLATE NOCASE,
	[FTCptStaChkHQ]	varchar(1) COLLATE NOCASE,
	[FTCptStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCptCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCouponType_L] (
	[FTCptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCptName]	varchar(100) COLLATE NOCASE,
	[FTCptRemark]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCptCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCouponTypeTmp] (
	[FTCptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCptType]	varchar(1) COLLATE NOCASE,
	[FTCptStaChk]	varchar(1) COLLATE NOCASE,
	[FTCptStaChkHQ]	varchar(1) COLLATE NOCASE,
	[FTCptStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMCouponTypeTmp_L] (
	[FTCptCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCptName]	varchar(100) COLLATE NOCASE,
	[FTCptRemark]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMCrdCpnList] (
	[FTCclCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCCclAmt]	real,
	[FDCclStartDate]	datetime,
	[FDCclEndDate]	datetime,
	[FTCclStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCclCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCrdCpnList_L] (
	[FTCclCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCclName]	varchar(100) COLLATE NOCASE,
	[FTCclPrnCond]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTCclCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCrdLogin] (
	[FTCrdCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCrdLogType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FDCrdPwdStart]	datetime NOT NULL,
	[FDCrdPwdExpired]	datetime,
	[FTCrdLogin]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCrdLoginPwd]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTCrdRmk]	varchar(50) COLLATE NOCASE,
	[FTCrdStaActive]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FTCrdLogType], [FDCrdPwdStart])

);
CREATE TABLE IF NOT EXISTS [TFNMCreditCard] (
	[FTCrdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBnkCode]	varchar(5) COLLATE NOCASE,
	[FCCrdChgPer]	real,
	[FTCrdCrdFmt]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode])

);
CREATE TABLE IF NOT EXISTS [TFNMCreditCard_L] (
	[FTCrdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCrdName]	varchar(100) COLLATE NOCASE,
	[FTCrdRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMCreditCardBIN] (
	[FTCrdCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCrdBinCode]	varchar(6) NOT NULL COLLATE NOCASE,
	[FTCrdBrand]	varchar(20) COLLATE NOCASE,
	[FTCrdType]	varchar(1) COLLATE NOCASE,
	[FTCrdCategory]	varchar(30) COLLATE NOCASE,
	[FTCrdRef]	varchar(5) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdCode], [FTCrdBinCode])

);
CREATE TABLE IF NOT EXISTS [TFNMEdc] (
	[FTEdcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTSedCode]	varchar(3) COLLATE NOCASE,
	[FTBnkCode]	varchar(5) COLLATE NOCASE,
	[FTEdcShwFont]	varchar(50) COLLATE NOCASE,
	[FTEdcShwBkg]	varchar(50) COLLATE NOCASE,
	[FTEdcOther]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTEdcCode])

);
CREATE TABLE IF NOT EXISTS [TFNMEdc_L] (
	[FTEdcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTEdcName]	varchar(100) COLLATE NOCASE,
	[FTEdcRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTEdcCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMEdcTmp] (
	[FTEdcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTSedCode]	varchar(3) COLLATE NOCASE,
	[FTBnkCode]	varchar(5) COLLATE NOCASE,
	[FTEdcShwFont]	varchar(50) COLLATE NOCASE,
	[FTEdcShwBkg]	varchar(50) COLLATE NOCASE,
	[FTEdcOther]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMEdcTmp_L] (
	[FTEdcCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTEdcName]	varchar(100) COLLATE NOCASE,
	[FTEdcRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRate] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCRteRate]	real,
	[FCRteFraction]	real,
	[FTRteType]	varchar(1) COLLATE NOCASE,
	[FTRteTypeChg]	varchar(1) COLLATE NOCASE,
	[FTRteSign]	varchar(10) COLLATE NOCASE,
	[FTRteStaLocal]	varchar(1) COLLATE NOCASE,
	[FTRteStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTAgnCode TEXT COLLATE NOCASE DEFAULT '', FCRteLastRate REAL, FTRteIsoCode TEXT COLLATE NOCASE, FTRteStaAlwChange TEXT COLLATE NOCASE, FCRteMaxUnit REAL, FDRteLastUpdOn TEXT,
    PRIMARY KEY ([FTRteCode])

);
CREATE TABLE IF NOT EXISTS [TFNMRate_L] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRteName]	varchar(100) COLLATE NOCASE,
	[FTRteShtName]	varchar(10) COLLATE NOCASE,
	[FTRteNameText]	varchar(50) COLLATE NOCASE,
	[FTRteDecText]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTRteCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMRateTmp] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCRteRate]	real,
	[FCRteFraction]	real,
	[FTRteType]	varchar(1) COLLATE NOCASE,
	[FTRteTypeChg]	varchar(1) COLLATE NOCASE,
	[FTRteSign]	varchar(10) COLLATE NOCASE,
	[FTRteStaLocal]	varchar(1) COLLATE NOCASE,
	[FTRteStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRateTmp_L] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRteName]	varchar(100) COLLATE NOCASE,
	[FTRteShtName]	varchar(10) COLLATE NOCASE,
	[FTRteNameText]	varchar(50) COLLATE NOCASE,
	[FTRteDecText]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRateUnit] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRtuSeq]	integer NOT NULL,
	[FCRtuFac]	real,
    PRIMARY KEY ([FTRteCode], [FNRtuSeq])

);
CREATE TABLE IF NOT EXISTS [TFNMRateUnitTmp] (
	[FTRteCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRtuSeq]	integer NOT NULL,
	[FCRtuFac]	real

);
CREATE TABLE IF NOT EXISTS [TFNMRcv] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTFmtCode]	varchar(5) COLLATE NOCASE,
	[FTRcvStaUse]	varchar(1) COLLATE NOCASE,
	[FTRcvStaShwInSlip]	varchar(1) COLLATE NOCASE,
	[FTRcv4Ret]	varchar(5) COLLATE NOCASE,
	[FTRcv4ChkOut]	varchar(5) COLLATE NOCASE,
	[FTAppStaAlwRet]	varchar(1) COLLATE NOCASE,
	[FTAppStaAlwCancel]	varchar(1) COLLATE NOCASE,
	[FTAppStaPayLast]	varchar(1) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE, FTRcvRefID VARCHAR(30), FTRcvRefEdc VARCHAR(5), FTRcvRefDoc VARCHAR(20), FNRcvQtySlip integer, FTRcvStaCshOrCrd  VARCHAR(1), FTRcvStaAlwAccPoint VARCHAR(1), FTRcvStaAlwDrawer VARCHAR(1), FTRcvStaReason VARCHAR(1), FTRcvStaShwSum TEXT COLLATE NOCASE DEFAULT '2',
    PRIMARY KEY ([FTRcvCode])

);
CREATE TABLE IF NOT EXISTS [TFNMRcv_L] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRcvName]	varchar(50) COLLATE NOCASE,
	[FTRcvRmk]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTRcvCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMRcvSpc] (
	[FTRcvCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAppCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAggCode]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRcvSeq]	integer,
	[FTPdtRmk]	varchar(50) COLLATE NOCASE,
	[FTAppStaAlwRet]	varchar(1) COLLATE NOCASE,
	[FTAppStaAlwCancel]	varchar(1) COLLATE NOCASE,
	[FTAppStaPayLast]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTRcvCode], [FTAppCode], [FTAggCode], [FTBchCode], [FTMerCode], [FTShpCode], [FTPosCode])

);
CREATE TABLE IF NOT EXISTS [TFNMRcvSpcConfig] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRcvSeq]	integer NOT NULL,
	[FNSysSeq]	integer NOT NULL,
	[FTSysKey]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSysStaUsrValue]	varchar(255) COLLATE NOCASE,
	[FTSysStaUsrRef]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTRcvCode], [FNRcvSeq], [FNSysSeq], [FTSysKey])

);
CREATE TABLE IF NOT EXISTS [TFNMRcvSpcConfigTmp] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRcvSeq]	integer NOT NULL,
	[FNSysSeq]	integer NOT NULL,
	[FTSysKey]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSysStaUsrValue]	varchar(255) COLLATE NOCASE,
	[FTSysStaUsrRef]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRcvSpcTmp] (
	[FTRcvCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTAppCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTAggCode]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNRcvSeq]	integer,
	[FTPdtRmk]	varchar(50) COLLATE NOCASE,
	[FTAppStaAlwRet]	varchar(1) COLLATE NOCASE,
	[FTAppStaAlwCancel]	varchar(1) COLLATE NOCASE,
	[FTAppStaPayLast]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRcvTmp] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTFmtCode]	varchar(5) COLLATE NOCASE,
	[FTRcvStaUse]	varchar(1) COLLATE NOCASE,
	[FTRcvStaShwInSlip]	varchar(1) COLLATE NOCASE,
	[FTRcv4Ret]	varchar(5) COLLATE NOCASE,
	[FTRcv4ChkOut]	varchar(5) COLLATE NOCASE,
	[FTAppStaAlwRet]	varchar(1) COLLATE NOCASE,
	[FTAppStaAlwCancel]	varchar(1) COLLATE NOCASE,
	[FTAppStaPayLast]	varchar(1) COLLATE NOCASE,
	[FTAgnCode]	varchar(10) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMRcvTmp_L] (
	[FTRcvCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRcvName]	varchar(50) COLLATE NOCASE,
	[FTRcvRmk]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNMVoucher] (
	[FTVocCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTVocBarCode]	varchar(30) COLLATE NOCASE,
	[FDVocExpired]	datetime,
	[FTVotCode]	varchar(5) COLLATE NOCASE,
	[FCVocValue]	real,
	[FCVocSalePri]	real,
	[FCVocBalance]	real,
	[FTVocComBook]	varchar(50) COLLATE NOCASE,
	[FTVocStaBook]	varchar(1) COLLATE NOCASE,
	[FTVocStaSale]	varchar(1) COLLATE NOCASE,
	[FTVocStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTVocCode])

);
CREATE TABLE IF NOT EXISTS [TFNMVoucher_L] (
	[FTVocCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTVocName]	varchar(100) COLLATE NOCASE,
	[FTVocRemark]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTVocCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNMVoucherType] (
	[FTVotCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTVotStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTVotCode])

);
CREATE TABLE IF NOT EXISTS [TFNMVoucherType_L] (
	[FTVotCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTVotName]	varchar(100) COLLATE NOCASE,
	[FTVotRemark]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTVotCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNSCrdBalType] (
	[FTCrdTxnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FTCrdStaAvai]	varchar(1) COLLATE NOCASE,
	[FNCrdStsSign]	integer,
    PRIMARY KEY ([FTCrdTxnCode])

);
CREATE TABLE IF NOT EXISTS [TFNSCrdBalType_L] (
	[FTCrdTxnCode]	varchar(3) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCrdName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCrdTxnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNSTxnType_L] (
	[FTTxnType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTTxnName]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTTxnType], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNSysPrc] (
	[FTSysCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTSysComName]	varchar(50) COLLATE NOCASE,
	[FTSysStaPrc]	varchar(1) COLLATE NOCASE,
	[FDSysDateLastPrc]	datetime,
    PRIMARY KEY ([FTSysCode])

);
CREATE TABLE IF NOT EXISTS [TFNTBnkDplDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBdhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNBddSeq]	integer NOT NULL,
	[FTBddType]	varchar(255) COLLATE NOCASE,
	[FTBddRefNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDBddRefDate]	datetime,
	[FCBddRefAmt]	real,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTBdhDocNo], [FNBddSeq])

);
CREATE TABLE IF NOT EXISTS [TFNTBnkDplHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBdhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTBdtCode]	varchar(5) COLLATE NOCASE,
	[FDBdhDate]	datetime,
	[FTMerCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTBdhUsrSender]	varchar(50) COLLATE NOCASE,
	[FTBdhUsrApv]	varchar(20) COLLATE NOCASE,
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBdhRefExt]	varchar(20) COLLATE NOCASE,
	[FDBdhRefExtDate]	datetime,
	[FCBdhTotCash]	real,
	[FCBdhTotCheque]	real,
	[FCBdhTotChqChg]	real,
	[FCBdhTotChqVat]	real,
	[FCBdhTotal]	real,
	[FTBdhRmk]	varchar(200) COLLATE NOCASE,
	[FTBdhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTBdhStaApv]	varchar(1) COLLATE NOCASE,
	[FNBdhStaDocAct]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTBdhDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTBnkStatement] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTBktType]	varchar(3) NOT NULL COLLATE NOCASE,
	[FDBktDate]	datetime,
	[FTBktAccFrom]	varchar(5) COLLATE NOCASE,
	[FTBktAccTo]	varchar(5) COLLATE NOCASE,
	[FTBktRefChq]	varchar(50) COLLATE NOCASE,
	[FCBktAmt]	real,
	[FCBktFree]	real,
	[FTBktRmk]	varchar(255) COLLATE NOCASE,
	[FTBktStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTBktApvCode]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTBbkCode])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCpdBarCpn]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCpdSeqNo]	integer,
	[FNCpdAlwMaxUse]	integer,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTCpdBarCpn])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponDTCst] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCpdBarCpn]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCpdSeqNo]	integer,
	[FTCpdRefCst]	varchar(30) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTCpdBarCpn])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponDTHis] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCpdBarCpn]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCpbSeqID]	integer NOT NULL,
	[FDCpbFrmStart]	datetime,
	[FTCpbFrmBch]	varchar(5) COLLATE NOCASE,
	[FTCpbFrmPos]	varchar(5) COLLATE NOCASE,
	[FTCpbFrmSalRef]	varchar(20) COLLATE NOCASE,
	[FTCpbStaBook]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTCpdBarCpn], [FNCpbSeqID])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponDTTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCpdBarCpn]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCpdSeqNo]	integer,
	[FNCpdAlwMaxUse]	integer

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCptCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FDCphDocDate]	datetime,
	[FTCphDisType]	varchar(1) COLLATE NOCASE,
	[FCCphDisValue]	real,
	[FTPplCode]	varchar(20) COLLATE NOCASE,
	[FDCphDateStart]	datetime,
	[FDCphDateStop]	datetime,
	[FTCphTimeStart]	varchar(8) COLLATE NOCASE,
	[FTCphTimeStop]	varchar(8) COLLATE NOCASE,
	[FTCphStaClosed]	varchar(255) COLLATE NOCASE,
	[FTUsrCode]	varchar(5) COLLATE NOCASE,
	[FTCphUsrApv]	varchar(5) COLLATE NOCASE,
	[FTCphStaDoc]	varchar(1) COLLATE NOCASE,
	[FTCphStaApv]	varchar(1) COLLATE NOCASE,
	[FTCphStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTCphStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FCCphMinValue]	real,
	[FTCphStaOnTopPmt]	varchar(1) COLLATE NOCASE,
	[FNCphLimitUsePerBill]	integer,
	[FTCphRefAccCode]	varchar(20) COLLATE NOCASE,
	[FTStaChkMember]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHD_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCpnName]	varchar(250) COLLATE NOCASE,
	[FTCpnMsg1]	varchar(255) COLLATE NOCASE,
	[FTCpnMsg2]	varchar(255) COLLATE NOCASE,
	[FTCpnCond]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCphBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTCphShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTCphBchTo], [FTCphMerTo], [FTCphShpTo])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDBchTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCphBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTCphShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDCstPri] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTPplCode])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDCstPriTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPplCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDPdt] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCphDocNo], [FTPdtCode], [FTPunCode])

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDPdtTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPunCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCptCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FDCphDocDate]	datetime,
	[FTCphDisType]	varchar(1) COLLATE NOCASE,
	[FCCphDisValue]	real,
	[FTPplCode]	varchar(20) COLLATE NOCASE,
	[FDCphDateStart]	datetime,
	[FDCphDateStop]	datetime,
	[FTCphTimeStart]	varchar(8) COLLATE NOCASE,
	[FTCphTimeStop]	varchar(8) COLLATE NOCASE,
	[FTCphStaClosed]	varchar(255) COLLATE NOCASE,
	[FTUsrCode]	varchar(5) COLLATE NOCASE,
	[FTCphUsrApv]	varchar(5) COLLATE NOCASE,
	[FTCphStaDoc]	varchar(1) COLLATE NOCASE,
	[FTCphStaApv]	varchar(1) COLLATE NOCASE,
	[FTCphStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTCphStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FCCphMinValue]	real,
	[FTCphStaOnTopPmt]	varchar(1) COLLATE NOCASE,
	[FNCphLimitUsePerBill]	integer,
	[FTCphRefAccCode]	varchar(20) COLLATE NOCASE,
	[FTStaChkMember]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCouponHDTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCphDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTCpnName]	varchar(250) COLLATE NOCASE,
	[FTCpnMsg1]	varchar(255) COLLATE NOCASE,
	[FTCpnMsg2]	varchar(255) COLLATE NOCASE,
	[FTCpnCond]	varchar(255) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdHis] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FTTxnStaOffLine]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCodeRef]	varchar(5) COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnDeposit]	real,
	[FCTxnNoRfn]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FCTxnPmt]	real,
    PRIMARY KEY ([FTBchCode], [FNTxnID], [FTTxnDocType], [FTTxnStaOffLine])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdHisBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FTTxnStaOffLine]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCodeRef]	varchar(5) COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnDeposit]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FNTxnID], [FTTxnDocType], [FTTxnStaOffLine])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdHisME] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FTTxnStaOffLine]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCodeRef]	varchar(5) COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnDeposit]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FNTxnID], [FTTxnDocType], [FTTxnStaOffLine])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdImpDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCihDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCidSeqNo]	integer NOT NULL,
	[FTCidCrdCode]	varchar(30) COLLATE NOCASE,
	[FCCvdOldBal]	real,
	[FTCidCrdRef]	varchar(30) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTCidCrdName]	varchar(200) COLLATE NOCASE,
	[FTCidCrdHolderID]	varchar(30) COLLATE NOCASE,
	[FTCidCrdDepart]	varchar(100) COLLATE NOCASE,
	[FTCidStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCidStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCidRmk]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCihDocNo], [FNCidSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdImpHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCihDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCihDocType]	varchar(1) COLLATE NOCASE,
	[FDCihDocDate]	datetime,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTCihRmk]	varchar(200) COLLATE NOCASE,
	[FTCihApvCode]	varchar(20) COLLATE NOCASE,
	[FDCihApvDate]	datetime,
	[FTCihStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTCihStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNCihStaDocAct]	integer,
	[FTCihStaDoc]	varchar(1) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCihDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdImpTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCihDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCidSeqNo]	integer NOT NULL,
	[FTCidCrdCode]	varchar(30) COLLATE NOCASE,
	[FCCvdOldBal]	real,
	[FTCidCrdRef]	varchar(30) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTCidCrdName]	varchar(200) COLLATE NOCASE,
	[FTCidCrdHolderID]	varchar(30) COLLATE NOCASE,
	[FTCidCrdDepart]	varchar(100) COLLATE NOCASE,
	[FTCidStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCidStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCidRmk]	varchar(255) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTSessionID]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCihDocNo], [FNCidSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdOffline] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCodeRef]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnDeposit]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
	[FTTxnStaOffLine]	varchar(1) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FNTxnID])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtCD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmcSeq]	integer NOT NULL,
	[FTPmcRefIn]	varchar(30) COLLATE NOCASE,
	[FTPmcRefEx]	varchar(30) COLLATE NOCASE,
	[FCPmcAmtPay]	real,
	[FCPmcAmtGet]	real,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNPmcSeq])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtCDTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmcSeq]	integer NOT NULL,
	[FTPmcRefIn]	varchar(30) COLLATE NOCASE,
	[FTPmcRefEx]	varchar(30) COLLATE NOCASE,
	[FCPmcAmtPay]	real,
	[FCPmcAmtGet]	real

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtColTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPmhDocDate]	datetime,
	[FTPmhDocType]	varchar(1) COLLATE NOCASE,
	[FTPmhCalType]	varchar(1) COLLATE NOCASE,
	[FDPmhDStart]	datetime,
	[FDPmhDStop]	datetime,
	[FTPmhTStart]	varchar(8) COLLATE NOCASE,
	[FTPmhTStop]	varchar(8) COLLATE NOCASE,
	[FTPmhStaAlwRetMnyPay]	varchar(1) COLLATE NOCASE,
	[FTPmhStaAlwRetMnyGet]	varchar(1) COLLATE NOCASE,
	[FTCtyCode]	varchar(5) COLLATE NOCASE,
	[FTCtyStaType]	varchar(1) COLLATE NOCASE,
	[FCPmcAmtPay]	real,
	[FCPmcAmtGet]	real,
	[FTPmcRefIn]	varchar(50) COLLATE NOCASE,
	[FTPmhBchTo]	varchar(5) COLLATE NOCASE,
	[FTPmhMerTo]	varchar(10) COLLATE NOCASE,
	[FTPmhShpTo]	varchar(5) COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE,
	[FTPmhName]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmdSeq]	integer NOT NULL,
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmdStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNPmdSeq])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtDTTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNPmdSeq]	integer NOT NULL,
	[FTCtyCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmdStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPmhDocDate]	datetime,
	[FTPmhDocType]	varchar(1) COLLATE NOCASE,
	[FTPmhCalType]	varchar(1) COLLATE NOCASE,
	[FDPmhDStart]	datetime,
	[FDPmhDStop]	datetime,
	[FTPmhTStart]	varchar(8) COLLATE NOCASE,
	[FTPmhTStop]	varchar(8) COLLATE NOCASE,
	[FTPmhStaClosed]	varchar(1) COLLATE NOCASE,
	[FTPmhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhStaApv]	varchar(1) COLLATE NOCASE,
	[FTPmhStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhRefAccCode]	varchar(20) COLLATE NOCASE,
	[FNPmhStaDocAct]	integer,
	[FTPmhStaAlwRetMnyPay]	varchar(1) COLLATE NOCASE,
	[FTPmhStaAlwRetMnyGet]	varchar(1) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTPmhUsrApv]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHD_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmhName]	varchar(200) COLLATE NOCASE,
	[FTPmhNameSlip]	varchar(25) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHDBch] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPmhShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTPmhDocNo], [FTPmhBchTo], [FTPmhMerTo], [FTPmhShpTo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHDBchTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhBchTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhMerTo]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTPmhShpTo]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhStaType]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHDTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FDPmhDocDate]	datetime,
	[FTPmhDocType]	varchar(1) COLLATE NOCASE,
	[FTPmhCalType]	varchar(1) COLLATE NOCASE,
	[FDPmhDStart]	datetime,
	[FDPmhDStop]	datetime,
	[FTPmhTStart]	varchar(8) COLLATE NOCASE,
	[FTPmhTStop]	varchar(8) COLLATE NOCASE,
	[FTPmhStaClosed]	varchar(1) COLLATE NOCASE,
	[FTPmhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhStaApv]	varchar(1) COLLATE NOCASE,
	[FTPmhStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTPmhRefAccCode]	varchar(20) COLLATE NOCASE,
	[FNPmhStaDocAct]	integer,
	[FTPmhStaAlwRetMnyPay]	varchar(1) COLLATE NOCASE,
	[FTPmhStaAlwRetMnyGet]	varchar(1) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTPmhUsrApv]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdPmtHDTmp_L] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPmhName]	varchar(200) COLLATE NOCASE,
	[FTPmhNameSlip]	varchar(25) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TFNTCrdSale] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCodeRef]	varchar(5) COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnDeposit]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
	[FTTxnStaOffLine]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FNTxnID])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdShiftDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXsdSeqNo]	integer NOT NULL,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FCXsdCardBal]	real,
	[FTXsdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTXsdRmk]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo], [FNXsdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdShiftHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXshDocType]	integer,
	[FDXshDocDate]	datetime,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTPosCode]	varchar(5) COLLATE NOCASE,
	[FTShfCode]	varchar(20) COLLATE NOCASE,
	[FNSdtSeqNo]	integer,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTXshApvCode]	varchar(20) COLLATE NOCASE,
	[FTXshRefInt]	varchar(20) COLLATE NOCASE,
	[FDXshRefIntDate]	datetime,
	[FNXshCardQty]	integer,
	[FTXshStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXshStaApv]	varchar(1) COLLATE NOCASE,
	[FNXshStaDocAct]	integer,
	[FNXshStaRef]	integer,
	[FTXshRmk]	varchar(255) COLLATE NOCASE,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FTXshAppVer]	varchar(20) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdShiftTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCsdSeqNo]	integer NOT NULL,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FCCtdCardBal]	real,
	[FTCrdStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCrdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCrdRmk]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTsessionID]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCshDocNo], [FNCsdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUp] (
	[FNTxnID]	integer NOT NULL,
	[FTTxnDocType]	varchar(2) NOT NULL COLLATE NOCASE,
	[FDTxnDocDate]	datetime,
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTTxnDocNoRef]	varchar(30) COLLATE NOCASE,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTTxnPosCode]	varchar(5) COLLATE NOCASE,
	[FCTxnCrdValue]	real,
	[FCTxnValue]	real,
	[FCTxnPmt]	real,
	[FCTxnNoRfn]	real,
	[FNTxnIDRef]	integer,
	[FTTxnStaPrc]	varchar(1) COLLATE NOCASE,
	[FTTxnStaCancel]	varchar(1) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FNTxnID])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUpDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXsdSeqNo]	integer NOT NULL,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FCXsdAmt]	real,
	[FCXsdAmtPmt]	real,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTXsdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTXsdRmk]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo], [FNXsdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUpHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXshDocType]	integer,
	[FDXshDocDate]	datetime,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FTPosCode]	varchar(5) COLLATE NOCASE,
	[FTShfCode]	varchar(20) COLLATE NOCASE,
	[FNSdtSeqNo]	integer,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTSpnCode]	varchar(20) COLLATE NOCASE,
	[FTXshApvCode]	varchar(20) COLLATE NOCASE,
	[FTXshRefInt]	varchar(20) COLLATE NOCASE,
	[FDXshRefIntDate]	datetime,
	[FNXshDocPrint]	integer,
	[FCXshTotal]	real,
	[FCXshToalPmt]	real,
	[FTXshStaDoc]	varchar(1) COLLATE NOCASE,
	[FTXshStaApv]	varchar(1) COLLATE NOCASE,
	[FNXshStaDocAct]	integer,
	[FNXshStaRef]	integer,
	[FTXshRmk]	varchar(255) COLLATE NOCASE,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FTXshAppVer]	varchar(20) COLLATE NOCASE,
	[FNXshCardQty]	integer,
	[FTXshStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FTXshStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUpPD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPmhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXsdSeqNo]	integer NOT NULL,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FCXpdAmtPay]	real,
	[FCXpdAmtGet]	real,
	[FTXpdRef1]	varchar(30) COLLATE NOCASE,
	[FTXpdRef2]	varchar(30) COLLATE NOCASE,
	[FNXpdQty]	integer,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo], [FTPmhDocNo], [FNXsdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUpRC] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTXshDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNXrcSeqNo]	integer NOT NULL,
	[FTRcvCode]	varchar(5) COLLATE NOCASE,
	[FTRcvName]	varchar(50) COLLATE NOCASE,
	[FTXrcRefNo1]	varchar(100) COLLATE NOCASE,
	[FTXrcRefNo2]	varchar(100) COLLATE NOCASE,
	[FDXrcRefDate]	datetime,
	[FTXrcRefDesc]	varchar(100) COLLATE NOCASE,
	[FTBnkCode]	varchar(5) COLLATE NOCASE,
	[FTRteCode]	varchar(5) COLLATE NOCASE,
	[FCXrcRteFac]	real,
	[FCXrcFrmLeftAmt]	real,
	[FCXrcUsrPayAmt]	real,
	[FCXrcNet]	real,
	[FCXrcChg]	real,
	[FTXrcRmk]	varchar(250) COLLATE NOCASE,
	[FTPhwCode]	varchar(30) COLLATE NOCASE,
	[FTXrcRetDocRef]	varchar(20) COLLATE NOCASE,
	[FTXrcStaPayOffline]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(50) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTXshDocNo], [FNXrcSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdTopUpTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCthDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCtdSeqNo]	integer NOT NULL,
	[FTCrdCode]	varchar(30) COLLATE NOCASE,
	[FCCtdCrdTP]	real,
	[FTCtdStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCtdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCtdRmk]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTSessionID]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCthDocNo], [FNCtdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdVoidDT] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCvhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCvdSeqNo]	integer NOT NULL,
	[FTCvdOldCode]	varchar(30) COLLATE NOCASE,
	[FCCvdOldBal]	real,
	[FTCvdNewCode]	varchar(30) COLLATE NOCASE,
	[FTCvdStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCvdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCvdRmk]	varchar(255) COLLATE NOCASE,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCvhDocNo], [FNCvdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdVoidHD] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCvhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCvhDocType]	varchar(1) COLLATE NOCASE,
	[FDCvhDocDate]	datetime,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTCvhRmk]	varchar(200) COLLATE NOCASE,
	[FTCvhApvCode]	varchar(20) COLLATE NOCASE,
	[FDCvhApvDate]	datetime,
	[FTCvhStaPrcDoc]	varchar(1) COLLATE NOCASE,
	[FNCvhStaDocAct]	integer,
	[FTCvhStaDoc]	varchar(1) COLLATE NOCASE,
	[FTCvhStaDelMQ]	varchar(1) COLLATE NOCASE,
	[FNCvhCardQty]	integer,
	[FTCvhStaCrdActive]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCvhDocNo])

);
CREATE TABLE IF NOT EXISTS [TFNTCrdVoidTmp] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTCvhDocNo]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCvdSeqNo]	integer NOT NULL,
	[FTCvdOldCode]	varchar(30) COLLATE NOCASE,
	[FCCvdOldBal]	real,
	[FTCvdNewCode]	varchar(30) COLLATE NOCASE,
	[FTCvdStaCrd]	varchar(30) COLLATE NOCASE,
	[FTCvdStaPrc]	varchar(30) COLLATE NOCASE,
	[FTCvdRmk]	varchar(255) COLLATE NOCASE,
	[FTRsnCode]	varchar(5) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTSessionID]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTCvhDocNo], [FNCvdSeqNo])

);
CREATE TABLE IF NOT EXISTS [TFNTMnyInBnk] (
	[FTBchCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTMnyDocNo]	varchar(50) NOT NULL COLLATE NOCASE,
	[FDMnyDocDate]	datetime NOT NULL,
	[FTMnyDocTime]	varchar(50) NOT NULL COLLATE NOCASE,
	[FDMnySalStart]	datetime,
	[FDMnySalEnd]	datetime,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTBbkCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FCMibQty]	real,
	[FTUsrSndBnk]	varchar(20) COLLATE NOCASE,
	[FTMnyStaSndBnk]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTBchCode], [FTMnyDocNo])

);
CREATE TABLE IF NOT EXISTS [TRGMCstBch] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNCbrSeq]	integer NOT NULL,
	[FTCbrRefBch]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNCbrQtyPos]	integer,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTCstCode], [FNCbrSeq])

);
CREATE TABLE IF NOT EXISTS [TRGMCstRegis] (
	[FNRegID]	integer NOT NULL,
	[FTRegBusName]	varchar(100) NOT NULL COLLATE NOCASE,
	[FNRegQtyBch]	integer NOT NULL,
	[FTRegLicType]	varchar(1) COLLATE NOCASE,
	[FTRegBusType]	varchar(1) COLLATE NOCASE,
	[FTRegBusOth]	varchar(100) COLLATE NOCASE,
	[FTRegRefCst]	varchar(10) COLLATE NOCASE,
	[FTRegStaConfirm]	varchar(1) COLLATE NOCASE,
	[FTRegStaActive]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FNRegID])

);
CREATE TABLE IF NOT EXISTS [TRGTCstBchLic] (
	[FTCstCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCbrRefBch]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTLicPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTLicRefSaleDoc]	varchar(20) COLLATE NOCASE,
	[FDLicStart]	datetime,
	[FDLicFinish]	datetime,
	[FTLicStaUse]	varchar(1) COLLATE NOCASE,
	[FTLicRefUUID]	varchar(100) COLLATE NOCASE,
    PRIMARY KEY ([FTCstCode], [FTCbrRefBch], [FTLicPdtCode])

);
CREATE TABLE IF NOT EXISTS [TRGTCstKey] (
	[FTCstKey]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTCbrRefBch]	varchar(5) NOT NULL COLLATE NOCASE,
	[FTKeyRefPdt]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTKeyByPdt]	varchar(4000) COLLATE NOCASE,
	[FTKeyRefDevice]	varchar(4000) COLLATE NOCASE,
	[FTKeyRefLicKey]	varchar(4000) COLLATE NOCASE,
    PRIMARY KEY ([FTCstKey], [FTCbrRefBch], [FTKeyRefPdt])

);
CREATE TABLE IF NOT EXISTS [TRGTLicKey] (
	[FTCstKey]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTRlkSrvType]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTRlkBchCode]	varchar(4000) COLLATE NOCASE,
    PRIMARY KEY ([FTCstKey], [FTRlkSrvType])

);
CREATE TABLE IF NOT EXISTS [TRGTPdtSet] (
	[FNKeyID]	integer NOT NULL,
	[FTKeyRefPdt]	varchar(4000) NOT NULL COLLATE NOCASE,
	[FTPdtCodeSet]	varchar(4000) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FNKeyID])

);
CREATE TABLE IF NOT EXISTS [TSysAPI2Ticket] (
	[FNApiID]	integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	[FTApiClass]	varchar(50) COLLATE NOCASE,
	[FTApiMethod]	varchar(100) COLLATE NOCASE,
	[FTApiStaActive]	varchar(1) COLLATE NOCASE,
	[FTApiFuncRef]	varchar(100) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysAPI2Ticket_L] (
	[FNApiID]	integer NOT NULL,
	[FNLngID]	integer NOT NULL,
	[FTApiDesc]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FNApiID], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysHisExport] (
	[FNRowSeq]	integer NOT NULL,
	[FTComName]	varchar(50) COLLATE NOCASE,
	[FTUsrCode]	varchar(20) COLLATE NOCASE,
	[FTUsrSession]	varchar(255) COLLATE NOCASE,
	[FTRptCode]	varchar(50) COLLATE NOCASE,
	[FDCreateDate]	datetime,
	[FDExprDate]	datetime,
	[FTExpType]	varchar(20) COLLATE NOCASE,
	[FTFilter]	varchar(255) COLLATE NOCASE,
	[FTZipName]	varchar(255) COLLATE NOCASE,
	[FTZipPath]	varchar(255) COLLATE NOCASE,
	[FNTotalFile]	integer,
	[FNSuccessFile]	integer,
	[FTStaDownload]	varchar(1) COLLATE NOCASE,
	[FTStaZip]	varchar(1) COLLATE NOCASE,
	[FTStaCancelDownload]	varchar(1) COLLATE NOCASE,
	[FTStaExpFail]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FNRowSeq])

);
CREATE TABLE IF NOT EXISTS [TSysLanguage] (
	[FNLngID]	integer NOT NULL,
	[FTLngName]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTLngNameEng]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTLngShortName]	varchar(5) COLLATE NOCASE,
	[FTLngStaLocal]	varchar(1) COLLATE NOCASE,
	[FTLngStaUse]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysLanguageTmp] (
	[FNLngID]	integer NOT NULL,
	[FTLngName]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTLngNameEng]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTLngShortName]	varchar(5) COLLATE NOCASE,
	[FTLngStaLocal]	varchar(1) COLLATE NOCASE,
	[FTLngStaUse]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysLnk] (
	[FTInfCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTInfTypeDoc]	varchar(1) COLLATE NOCASE,
	[FTInfStaUse]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTInfCode])

);
CREATE TABLE IF NOT EXISTS [TSysLnk_L] (
	[FTInfCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTInfName]	varchar(100) COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
    PRIMARY KEY ([FTInfCode])

);
CREATE TABLE IF NOT EXISTS [TsysMasTmp] (
	[FTMttTableKey]	varchar(255) COLLATE NOCASE,
	[FTMttRefKey]	varchar(255) COLLATE NOCASE,
	[FTMttSessionID]	varchar(255) COLLATE NOCASE,
	[FTRefPdtCode]	varchar(20) COLLATE NOCASE,
	[FTPdtCode]	varchar(20) COLLATE NOCASE,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTBarCode]	varchar(25) COLLATE NOCASE,
	[FTPunCode]	varchar(5) COLLATE NOCASE,
	[FTPunName]	varchar(50) COLLATE NOCASE,
	[FTPlcCode]	varchar(5) COLLATE NOCASE,
	[FTPlcName]	varchar(100) COLLATE NOCASE,
	[FTSplCode]	varchar(20) COLLATE NOCASE,
	[FTSplName]	varchar(200) COLLATE NOCASE,
	[FTSplStaAlwPO]	varchar(1) COLLATE NOCASE,
	[FTBarStaUse]	varchar(1) COLLATE NOCASE,
	[FTBarStaAlwSale]	varchar(1) COLLATE NOCASE,
	[FCPdtUnitFact]	real,
	[FTPdtGrade]	varchar(50) COLLATE NOCASE,
	[FCPdtWeight]	real,
	[FTClrCode]	varchar(5) COLLATE NOCASE,
	[FTClrName]	varchar(50) COLLATE NOCASE,
	[FTPszCode]	varchar(5) COLLATE NOCASE,
	[FTPszName]	varchar(50) COLLATE NOCASE,
	[FTPdtUnitDim]	varchar(50) COLLATE NOCASE,
	[FTPdtPkgDim]	varchar(50) COLLATE NOCASE,
	[FTPdtStaAlwPick]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwPoHQ]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwBuy]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwSale]	varchar(1) COLLATE NOCASE,
	[FCPgdPriceRet]	real,
	[FCPgdPriceWhs]	real,
	[FCPgdPriceNet]	real,
	[FDLastUpdOn]	datetime,
	[FDCreateOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
	[FTBchCode]	varchar(5) COLLATE NOCASE,
	[FTShpCode]	varchar(5) COLLATE NOCASE,
	[FDSgpStart]	datetime,
	[FCSgpPerSun]	real,
	[FCSgpPerMon]	real,
	[FCSgpPerTue]	real,
	[FCSgpPerWed]	real,
	[FCSgpPerThu]	real,
	[FCSgpPerFri]	real,
	[FCSgpPerSat]	real,
	[FTMerCode]	varchar(5) COLLATE NOCASE,
	[FNLayNo]	integer,
	[FNLayRow]	integer,
	[FNLayCol]	integer,
	[FTLayStaUse]	varchar(1) COLLATE NOCASE,
	[FTPosCode]	varchar(5) COLLATE NOCASE,
	[FTRakCode]	varchar(5) COLLATE NOCASE,
	[FTGhdApp]	varchar(50) COLLATE NOCASE,
	[FTAppName]	varchar(50) COLLATE NOCASE,
	[FTGdtName]	varchar(200) COLLATE NOCASE,
	[FTGhdCode]	varchar(5) COLLATE NOCASE,
	[FTSysCode]	varchar(5) COLLATE NOCASE,
	[FTGdtCallByName]	varchar(100) COLLATE NOCASE,
	[FNGdtFuncLevel]	integer,
	[FTGdtStaUse]	varchar(1) COLLATE NOCASE,
	[FTWahCode]	varchar(5) COLLATE NOCASE,
	[FCSpwQtyMin]	real,
	[FCSpwQtyMax]	real,
	[FTSpwRmk]	varchar(100) COLLATE NOCASE,
	[FTStaEdit]	varchar(2) COLLATE NOCASE,
	[FTKbdScreen]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysMenuAlbAct] (
	[FTMnuCode]	varchar(255) NOT NULL COLLATE NOCASE,
	[FTAutStaRead]	varchar(255) COLLATE NOCASE,
	[FTAutStaAdd]	varchar(255) COLLATE NOCASE,
	[FTAutStaEdit]	varchar(255) COLLATE NOCASE,
	[FTAutStaDelete]	varchar(255) COLLATE NOCASE,
	[FTAutStaCancel]	varchar(255) COLLATE NOCASE,
	[FTAutStaAppv]	varchar(255) COLLATE NOCASE,
	[FTAutStaPrint]	varchar(255) COLLATE NOCASE,
	[FTAutStaPrintMore]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTMnuCode])

);
CREATE TABLE IF NOT EXISTS [TSysMenuGrp] (
	[FTGmnCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTGmnModCode]	varchar(20) COLLATE NOCASE,
	[FNGmnShwSeq]	integer,
	[FTGmnStaUse]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTGmnCode])

);
CREATE TABLE IF NOT EXISTS [TSysMenuGrp_L] (
	[FTGmnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	float NOT NULL,
	[FTGmnName]	varchar(50) COLLATE NOCASE,
	[FTGmnSystem]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTGmnCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysMenuGrpModule] (
	[FTGmnModCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNGmnModShwSeq]	float NOT NULL,
	[FTGmnModStaUse]	varchar(1) COLLATE NOCASE,
	[FTGmmModPathIcon]	varchar(255) COLLATE NOCASE,
	[FTGmmModColorBtn]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTGmnModCode])

);
CREATE TABLE IF NOT EXISTS [TSysMenuGrpModule_L] (
	[FTGmnModCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNLngID]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTGmnModName]	varchar(255) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FTGmnModCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysMenuList] (
	[FTGmnCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTMnuParent]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTMnuCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTLicPdtCode]	varchar(10) COLLATE NOCASE,
	[FNMnuSeq]	integer,
	[FTMnuCtlName]	varchar(255) COLLATE NOCASE,
	[FNMnuLevel]	integer,
	[FTMnuStaPosHpm]	varchar(1) COLLATE NOCASE,
	[FTMnuStaPosFhn]	varchar(1) COLLATE NOCASE,
	[FTMnuStaSmartHpm]	varchar(1) COLLATE NOCASE,
	[FTMnuStaSmartFhn]	varchar(1) COLLATE NOCASE,
	[FTMnuStaMoreHpm]	varchar(1) COLLATE NOCASE,
	[FTMnuStaMoreFhn]	varchar(1) COLLATE NOCASE,
	[FTMnuType]	varchar(1) COLLATE NOCASE,
	[FTMnuStaAPIPos]	varchar(255) COLLATE NOCASE,
	[FTMnuStaAPISmart]	varchar(1) COLLATE NOCASE,
	[FTMnuStaUse]	varchar(1) COLLATE NOCASE,
	[FTMnuPath]	varchar(255) COLLATE NOCASE,
	[FTGmnModCode]	varchar(10) COLLATE NOCASE,
	[FTMnuImgPath]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTGmnCode], [FTMnuParent], [FTMnuCode])

);
CREATE TABLE IF NOT EXISTS [TSysMenuList_L] (
	[FTMnuCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	float NOT NULL,
	[FTMnuName]	varchar(255) COLLATE NOCASE,
	[FTMnuRmk]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTMnuCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysMPdt] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FTPdtStkControl]	varchar(1) COLLATE NOCASE,
	[FTPdtGrpControl]	varchar(1) COLLATE NOCASE,
	[FTPdtForSystem]	varchar(30) COLLATE NOCASE,
	[FCPdtQtyOrdBuy]	real,
	[FCPdtCostDef]	real,
	[FCPdtCostOth]	real,
	[FCPdtCostStd]	real,
	[FCPdtMin]	real,
	[FCPdtMax]	real,
	[FTPdtPoint]	varchar(1) COLLATE NOCASE,
	[FCPdtPointTime]	real,
	[FTPdtType]	varchar(1) COLLATE NOCASE,
	[FTPdtSaleType]	varchar(1) COLLATE NOCASE,
	[FTPdtSetOrSN]	varchar(1) COLLATE NOCASE,
	[FTPdtStaSetPri]	varchar(1) COLLATE NOCASE,
	[FTPdtStaSetShwDT]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwDis]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwReturn]	varchar(1) COLLATE NOCASE,
	[FTPdtStaVatBuy]	varchar(1) COLLATE NOCASE,
	[FTPdtStaVat]	varchar(1) COLLATE NOCASE,
	[FTPdtStaActive]	varchar(1) COLLATE NOCASE,
	[FTPdtStaAlwReCalOpt]	varchar(1) COLLATE NOCASE,
	[FTPdtStaCsm]	varchar(1) COLLATE NOCASE,
	[FTTcgCode]	varchar(5) COLLATE NOCASE,
	[FTPgpChain]	varchar(30) COLLATE NOCASE,
	[FTPtyCode]	varchar(5) COLLATE NOCASE,
	[FTPbnCode]	varchar(5) COLLATE NOCASE,
	[FTPmoCode]	varchar(5) COLLATE NOCASE,
	[FTVatCode]	varchar(5) COLLATE NOCASE,
	[FTEvhCode]	varchar(5) COLLATE NOCASE,
	[FDPdtSaleStart]	datetime,
	[FDPdtSaleStop]	datetime,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TSysMPdt_L] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTPdtNameOth]	varchar(100) COLLATE NOCASE,
	[FTPdtNameABB]	varchar(50) COLLATE NOCASE,
	[FTPdtRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysMsgAlipay_L] (
	[FTErrFunc]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTErrCode]	varchar(100) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTErrDesc]	varchar(4000) COLLATE NOCASE,
	[FTErrRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTErrFunc], [FTErrCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysPdt] (
	[FTPdtCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTPdtStaVat]	varchar(1) COLLATE NOCASE,
	[FTPdtSysTable]	varchar(255) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode])

);
CREATE TABLE IF NOT EXISTS [TSysPdt_L] (
	[FTPdtCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTPdtName]	varchar(100) COLLATE NOCASE,
	[FTPdtRmk]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTPdtCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysPortPrn] (
	[FTSppCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FTSppValue]	varchar(100) COLLATE NOCASE,
	[FTSppRef]	varchar(100) COLLATE NOCASE,
	[FTSppType]	varchar(3) COLLATE NOCASE,
	[FTSppStaUse]	varchar(1) COLLATE NOCASE,
	[FDLastUpdOn]	datetime,
	[FTLastUpdBy]	varchar(20) COLLATE NOCASE,
	[FDCreateOn]	datetime,
	[FTCreateBy]	varchar(20) COLLATE NOCASE,
    PRIMARY KEY ([FTSppCode])

);
CREATE TABLE IF NOT EXISTS [TSysPortPrn_L] (
	[FTSppCode]	varchar(50) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTSppName]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTSppCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysPosHW] (
	[FTShwCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShwHWKey]	varchar(10) COLLATE NOCASE,
	[FTShwName]	varchar(150) COLLATE NOCASE,
	[FTShwNameEng]	varchar(150) COLLATE NOCASE,
	[FTShwSystem]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwPrinter]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwCom]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwTCP]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwBT]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTShwCode])

);
CREATE TABLE IF NOT EXISTS [TSysPosHWTmp] (
	[FTShwCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTShwHWKey]	varchar(10) COLLATE NOCASE,
	[FTShwName]	varchar(150) COLLATE NOCASE,
	[FTShwNameEng]	varchar(150) COLLATE NOCASE,
	[FTShwSystem]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwPrinter]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwCom]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwTCP]	varchar(50) COLLATE NOCASE,
	[FTShwStaAlwBT]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysPosModel] (
	[FTSpmCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FTSpmBrand]	varchar(10) COLLATE NOCASE,
	[FTSpmName]	varchar(150) COLLATE NOCASE,
	[FTSpmNameEng]	varchar(150) COLLATE NOCASE,
	[FTSpmSystem]	varchar(50) COLLATE NOCASE,
	[FTSpmRemark]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTSpmCode])

);
CREATE TABLE IF NOT EXISTS [TSysReport] (
	[FTRptCode]	varchar(20) COLLATE NOCASE,
	[FTGrpRptModCode]	varchar(20) COLLATE NOCASE,
	[FTGrpRptCode]	varchar(20) COLLATE NOCASE,
	[FTRptRoute]	varchar(255) COLLATE NOCASE,
	[FTRptStaUseFrm]	varchar(255) COLLATE NOCASE,
	[FTRptTblView]	varchar(255) COLLATE NOCASE,
	[FTRptFilterCol]	varchar(255) COLLATE NOCASE,
	[FTRptFileName]	varchar(255) COLLATE NOCASE,
	[FTRptFilterValidate]	varchar(255) COLLATE NOCASE,
	[FTRptStaShwBch]	varchar(255) COLLATE NOCASE,
	[FTRptStaShwYear]	varchar(1) COLLATE NOCASE,
	[FTRptSeqNo]	integer,
	[FTRptStaUse]	varchar(1) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysReport_L] (
	[FTRptCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRptName]	varchar(100) COLLATE NOCASE,
	[FTRptDes]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTRptCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysReportFilter] (
	[FTRptFltCode]	integer NOT NULL,
	[FTRptFltStaFrm]	integer,
	[FTRptFltStaTo]	integer,
	[FTRptGrpFlt]	varchar(10) COLLATE NOCASE,
    PRIMARY KEY ([FTRptFltCode])

);
CREATE TABLE IF NOT EXISTS [TSysReportFilter_L] (
	[FTRptFltCode]	integer NOT NULL,
	[FNLngID]	varchar(1) NOT NULL COLLATE NOCASE,
	[FTRptFltName]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTRptFltCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysReportGrp] (
	[FTGrpRptCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNGrpRptShwSeq]	integer,
	[FTGrpRptStaUse]	varchar(1) COLLATE NOCASE,
	[FTGrpRptModCode]	varchar(20) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FTGrpRptCode], [FTGrpRptModCode])

);
CREATE TABLE IF NOT EXISTS [TSysReportGrp_L] (
	[FTGrpRptCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTGrpRptName]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTGrpRptCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysReportModule] (
	[FTGrpRptModCode]	varchar(10) NOT NULL COLLATE NOCASE,
	[FNGrpRptModShwSeq]	integer,
	[FTGrpRptModStaUse]	varchar(1) COLLATE NOCASE,
	[FTGrpRptModRoute]	varchar(255) COLLATE NOCASE,
    PRIMARY KEY ([FTGrpRptModCode])

);
CREATE TABLE IF NOT EXISTS [TSysReportModule_L] (
	[FTGrpRptModCode]	varchar(20) NOT NULL COLLATE NOCASE,
	[FNGrpRptModName]	varchar(255) NOT NULL COLLATE NOCASE,
	[FNLngID]	varchar(1) NOT NULL COLLATE NOCASE,
    PRIMARY KEY ([FTGrpRptModCode], [FNGrpRptModName], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysRsnGrp_L] (
	[FTRsgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRsgName]	varchar(200) COLLATE NOCASE,
	[FTRsgRmk]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTRsgCode], [FNLngID])

);
CREATE TABLE IF NOT EXISTS [TSysRsnGrpTmp_L] (
	[FTRsgCode]	varchar(5) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTRsgName]	varchar(200) COLLATE NOCASE,
	[FTRsgRmk]	varchar(200) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysScale] (
	[FTSslCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTSslName]	varchar(150) COLLATE NOCASE,
	[FTSslComSetting]	varchar(100) COLLATE NOCASE,
	[FTSslStaCom]	varchar(50) COLLATE NOCASE,
	[FTSslStaTCP]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTSslCode])

);
CREATE TABLE IF NOT EXISTS [TSysScaleMap] (
	[FTSslCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNSsmSeq]	integer NOT NULL,
	[FTSsmDesc]	varchar(100) COLLATE NOCASE,
	[FNSsmFixLen]	integer,
	[FTSsmChrSplit]	varchar(30) COLLATE NOCASE,
	[FTSsmChrExtra]	varchar(30) COLLATE NOCASE,
	[FTsmKey]	varchar(50) COLLATE NOCASE,
    PRIMARY KEY ([FTSslCode], [FNSsmSeq])

);
CREATE TABLE IF NOT EXISTS [TSysScaleMapTmp] (
	[FTSslCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FNSsmSeq]	integer NOT NULL,
	[FTSsmDesc]	varchar(100) COLLATE NOCASE,
	[FNSsmFixLen]	integer,
	[FTSsmChrSplit]	varchar(30) COLLATE NOCASE,
	[FTSsmChrExtra]	varchar(30) COLLATE NOCASE,
	[FTsmKey]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysScaleTmp] (
	[FTSslCode]	varchar(30) NOT NULL COLLATE NOCASE,
	[FTSslName]	varchar(150) COLLATE NOCASE,
	[FTSslComSetting]	varchar(100) COLLATE NOCASE,
	[FTSslStaCom]	varchar(50) COLLATE NOCASE,
	[FTSslStaTCP]	varchar(50) COLLATE NOCASE

);
CREATE TABLE IF NOT EXISTS [TSysShwDT] (
	[FTShwTblDT]	varchar(200) NOT NULL COLLATE NOCASE,
	[FNShwSeq]	integer,
	[FTShwFedShw]	varchar(250) NOT NULL COLLATE NOCASE,
	[FTShwFedStaUsed]	varchar(1) COLLATE NOCASE,
	[FTShwFedSetByDef]	varchar(1) COLLATE NOCASE,
	[FTShwFedSetByUsr]	varchar(1) COLLATE NOCASE,
	[FNShwColWidth]	integer,
	[FTShwStaAlwEdit]	varchar(1) COLLATE NOCASE,
    PRIMARY KEY ([FTShwTblDT], [FTShwFedShw])

);
CREATE TABLE IF NOT EXISTS [TSysShwDT_L] (
	[FTShwTblDT]	varchar(200) NOT NULL COLLATE NOCASE,
	[FNShwSeq]	integer,
	[FTShwFedShw]	varchar(250) NOT NULL COLLATE NOCASE,
	[FNLngID]	integer NOT NULL,
	[FTShwNameDef]	varchar(200) COLLATE NOCASE,
	[FTShwNameUsr]	varchar(200) COLLATE NOCASE,
    PRIMARY KEY ([FTShwTblDT], [FTShwFedShw], [FNLngID])

);
CREATE TABLE IF NOT EXISTS TSysRcvFmt (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FTFmtKbRef varchar (20) COLLATE NOCASE, FTFmtRef varchar (50) COLLATE NOCASE, FTFmtStaUsed varchar (1) COLLATE NOCASE, FTFmtStaAlwKeySum varchar (1) COLLATE NOCASE, PRIMARY KEY (FTFmtCode));
CREATE TABLE IF NOT EXISTS TSysDisPolicyTmp_L (FTDisCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTDisName varchar (255) COLLATE NOCASE, FTDisRemark varchar (255) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysEdc (FTSedCode varchar (3) NOT NULL COLLATE NOCASE, FTSedModel varchar (50) COLLATE NOCASE, FNSedAck integer, FTSedDllVer varchar (1) COLLATE NOCASE, FNSedTimeOut integer, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTSedCode));
CREATE TABLE IF NOT EXISTS TSysFormatAPI_L (FTApiFmtCode varchar (10) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTApiFmtName varchar (50) COLLATE NOCASE, FTApiRmk varchar (100) COLLATE NOCASE, PRIMARY KEY (FTApiFmtCode, FNLngID));
CREATE TABLE IF NOT EXISTS TSysDisPolicyTmp (FTDisCode varchar (5) NOT NULL COLLATE NOCASE, FTDisGroup varchar (1) COLLATE NOCASE, FNDisLevel integer, FTDisStaUse varchar (1) COLLATE NOCASE, FTDisPosFunc varchar (50) COLLATE NOCASE, FTDisStaPrice varchar (1) COLLATE NOCASE, FTDisCodeRef varchar (20) COLLATE NOCASE, FNDisType integer, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysDocType (FNSdtID integer PRIMARY KEY AUTOINCREMENT NOT NULL, FTSdtTblName varchar (100) NOT NULL COLLATE NOCASE, FTSdtFedTypeName varchar (50) NOT NULL COLLATE NOCASE, FTSdtDocGrp varchar (2) COLLATE NOCASE, FNSdtDocType integer NOT NULL, FTSdtDocName varchar (50) COLLATE NOCASE, FTSdtDocTypeRef varchar (10) NOT NULL COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysApp_L (FTAppCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTAppName varchar (50) COLLATE NOCASE, FTAppRmk varchar (50) COLLATE NOCASE, PRIMARY KEY (FTAppCode, FNLngID));
CREATE TABLE IF NOT EXISTS TSysConfigList_L (FTSysCode varchar (20) NOT NULL COLLATE NOCASE, FTSysApp varchar (50) NOT NULL COLLATE NOCASE, FTSysKey varchar (50) NOT NULL COLLATE NOCASE, FTSysSeq varchar (3) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTSysName varchar (255) COLLATE NOCASE, F7 varchar (255) COLLATE NOCASE, F8 varchar (255) COLLATE NOCASE, PRIMARY KEY (FTSysCode, FTSysApp, FTSysSeq, FTSysKey, FNLngID));
CREATE TABLE IF NOT EXISTS TSysRcvFmtTmp (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FTFmtKbRef varchar (20) COLLATE NOCASE, FTFmtRef varchar (50) COLLATE NOCASE, FTFmtStaUsed varchar (1) COLLATE NOCASE, FTFmtStaAlwKeySum varchar (1) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysRcvFmt_L (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTFmtName varchar (50) COLLATE NOCASE, FTRcvRmk varchar (255) COLLATE NOCASE, PRIMARY KEY (FTFmtCode, FNLngID));
CREATE TABLE IF NOT EXISTS TSysDocApv (FNDapID integer PRIMARY KEY AUTOINCREMENT NOT NULL, FTDapTable varchar (50) COLLATE NOCASE, FTDapRefType varchar (50) COLLATE NOCASE, FNDapSeq integer, FTDapName varchar (30) COLLATE NOCASE, FTDapNameOth varchar (30) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysDisPolicy (FTDisCode varchar (5) NOT NULL COLLATE NOCASE, FTDisGroup varchar (1) COLLATE NOCASE, FNDisLevel integer, FTDisStaUse varchar (1) COLLATE NOCASE, FTDisPosFunc varchar (50) COLLATE NOCASE, FTDisStaPrice varchar (1) COLLATE NOCASE, FTDisCodeRef varchar (20) COLLATE NOCASE, FNDisType integer, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTDisCode));
CREATE TABLE IF NOT EXISTS TSysApp (FTAppCode varchar (5) NOT NULL COLLATE NOCASE, FTAppVersion varchar (50) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTAppCode));
CREATE TABLE IF NOT EXISTS TSysAutoFmt (FTSatTblName varchar (100) NOT NULL COLLATE NOCASE, FTSatFedCode varchar (100) NOT NULL COLLATE NOCASE, FTSatStaDocType varchar (10) NOT NULL COLLATE NOCASE, FTSatGroup varchar (1) COLLATE NOCASE, FTGmnCode varchar (10) COLLATE NOCASE, FTSatDocTypeName varchar (50) COLLATE NOCASE, FTSatStaAlw1Chr varchar (1) COLLATE NOCASE, FTSatStaAlw2Bch varchar (1) COLLATE NOCASE, FTSatStaAlw3Pos varchar (1) COLLATE NOCASE, FTSatStaAlw4YY varchar (1) COLLATE NOCASE, FTSatStaAlw5MM varchar (1) COLLATE NOCASE, FTSatStaAlw6DD varchar (1) COLLATE NOCASE, FTSatStaAlw7Sep varchar (1) COLLATE NOCASE, FNSatMaxFedSize integer, FNSatMinRunning integer, FTSatDefFmtAll varchar (50) COLLATE NOCASE, PRIMARY KEY (FTSatTblName, FTSatFedCode, FTSatStaDocType));
CREATE TABLE IF NOT EXISTS TSysConfig (FTSysCode varchar (20) NOT NULL COLLATE NOCASE, FTSysApp varchar (50) NOT NULL COLLATE NOCASE, FTSysKey varchar (50) NOT NULL COLLATE NOCASE, FTSysSeq varchar (3) NOT NULL COLLATE NOCASE, FTGmnCode varchar (10) COLLATE NOCASE, FTSysStaAlwEdit varchar (1) COLLATE NOCASE, FTSysStaDataType varchar (1) COLLATE NOCASE, FNSysMaxLength varchar (10) COLLATE NOCASE, FTSysStaDefValue varchar (255) COLLATE NOCASE, FTSysStaDefRef varchar (255) COLLATE NOCASE, FTSysStaUsrValue varchar (255) COLLATE NOCASE, FTSysStaUsrRef varchar (255) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTSysCode, FTSysApp, FTSysSeq, FTSysKey));
CREATE TABLE IF NOT EXISTS TSysConfig_L (FTSysCode varchar (20) NOT NULL COLLATE NOCASE, FTSysApp varchar (50) NOT NULL COLLATE NOCASE, FTSysKey varchar (50) NOT NULL COLLATE NOCASE, FTSysSeq varchar (3) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTSysName varchar (255) COLLATE NOCASE, FTSysDesc varchar (100) COLLATE NOCASE, FTSysRmk varchar (100) COLLATE NOCASE, PRIMARY KEY (FTSysCode, FTSysApp, FTSysSeq, FTSysKey, FNLngID));
CREATE TABLE IF NOT EXISTS TSysRcvFmtTmp_L (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTFmtName varchar (50) COLLATE NOCASE, FTRcvRmk varchar (255) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysRcvConfig (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FNSysSeq integer NOT NULL, FTSysKey varchar (20) NOT NULL COLLATE NOCASE, FTSysStaUsrValue varchar (255) COLLATE NOCASE, FTSysStaUsrRef varchar (255) COLLATE NOCASE, FTBchCode varchar (5) NOT NULL COLLATE NOCASE, PRIMARY KEY (FTFmtCode, FNSysSeq, FTSysKey));
CREATE TABLE IF NOT EXISTS TSysFuncKB (FTSysCode varchar (5) NOT NULL COLLATE NOCASE, FNSysKeyShift integer, FNSysKeyAscii integer, FTSysKeyName varchar (20) COLLATE NOCASE, FTSysKeyFunc varchar (20) COLLATE NOCASE, FTSysStaUse varchar (1) COLLATE NOCASE, PRIMARY KEY (FTSysCode));
CREATE TABLE IF NOT EXISTS TSysDisPolicy_L (FTDisCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTDisName varchar (255) COLLATE NOCASE, FTDisRemark varchar (255) COLLATE NOCASE, PRIMARY KEY (FTDisCode, FNLngID));
CREATE TABLE IF NOT EXISTS TSysRcvConfigTmp (FTFmtCode varchar (5) NOT NULL COLLATE NOCASE, FNSysSeq integer NOT NULL, FTSysKey varchar (20) NOT NULL COLLATE NOCASE, FTSysStaUsrValue varchar (255) COLLATE NOCASE, FTSysStaUsrRef varchar (255) COLLATE NOCASE, FTBchCode varchar (5) NOT NULL COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysEdcTmp (FTSedCode varchar (3) NOT NULL COLLATE NOCASE, FTSedModel varchar (50) COLLATE NOCASE, FNSedAck integer, FTSedDllVer varchar (1) COLLATE NOCASE, FNSedTimeOut integer, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysSyncDataTmp (FNSynSeqNo integer NOT NULL, FTSynGroup varchar (30) COLLATE NOCASE, FTSynTable varchar (30) COLLATE NOCASE, FTSynTable_L varchar (30) COLLATE NOCASE, FTSynType varchar (1) COLLATE NOCASE, FDSynLast datetime, FNSynSchedule integer, FTSynStaUse varchar (1) COLLATE NOCASE, FTSynUriDwn varchar (200) COLLATE NOCASE, FTSynUriUld varchar (200) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysSyncModuleTmp (FTAppCode varchar (10) NOT NULL COLLATE NOCASE, FNSynSeqNo integer NOT NULL);
CREATE TABLE IF NOT EXISTS TSysSyncDataTmp_L (FNSynSeqNo integer NOT NULL, FNLngID integer NOT NULL, FTSynName varchar (100) COLLATE NOCASE, FTSynRmk varchar (100) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TSysSyncData_L (FNSynSeqNo integer NOT NULL, FNLngID integer NOT NULL, FTSynName varchar (100) COLLATE NOCASE, FTSynRmk varchar (100) COLLATE NOCASE, PRIMARY KEY (FNSynSeqNo, FNLngID));
CREATE TABLE IF NOT EXISTS TSysTable (FTStbCode varchar (3) NOT NULL COLLATE NOCASE, FTStbShort varchar (50) COLLATE NOCASE, FTStbTblName varchar (100) COLLATE NOCASE, FTStbFedName varchar (100) COLLATE NOCASE, FNStbFedSize integer, FTStbThaDesc varchar (255) COLLATE NOCASE, FTStbEngDesc varchar (255) COLLATE NOCASE, FTStbStaMst varchar (1) COLLATE NOCASE, FTStbStaBrw varchar (1) COLLATE NOCASE, FNStbStaBrwSeq integer, FTStbStaImpExp varchar (1) COLLATE NOCASE, FTStbStaTemplate varchar (1) COLLATE NOCASE, FTStbStaTmp1 varchar (1) COLLATE NOCASE, FTStbStaTmp2 varchar (1) COLLATE NOCASE, FNStbLev integer, FTStbGrp varchar (50) COLLATE NOCASE, FTStbNameOth varchar (255) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTStbCode));
CREATE TABLE IF NOT EXISTS TSysSyncData (FNSynSeqNo integer NOT NULL, FTSynGroup varchar (30) COLLATE NOCASE, FTSynTable varchar (30) COLLATE NOCASE, FTSynTable_L varchar (30) COLLATE NOCASE, FTSynType varchar (1) COLLATE NOCASE, FDSynLast datetime, FNSynSchedule integer, FTSynStaUse varchar (1) COLLATE NOCASE, FTSynUriDwn varchar (200) COLLATE NOCASE, FTSynUriUld varchar (200) COLLATE NOCASE, PRIMARY KEY (FNSynSeqNo));
CREATE TABLE IF NOT EXISTS TSysSyncModule (FTAppCode varchar (10) NOT NULL COLLATE NOCASE, FNSynSeqNo integer NOT NULL, PRIMARY KEY (FTAppCode, FNSynSeqNo));
CREATE TABLE IF NOT EXISTS TCNMChannel (FTChnCode varchar (5) NOT NULL COLLATE NOCASE, FTAppCode varchar (5) COLLATE NOCASE, FTChnGroup varchar (5) COLLATE NOCASE, FNChnSeq integer, FTChnStaUse varchar (1) COLLATE NOCASE, FTChnAppDef varchar (1) COLLATE NOCASE, FTChnRefCode varchar (20) COLLATE NOCASE, FTPplCode varchar (20) NOT NULL COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, FTWahCode varchar (10) COLLATE NOCASE, PRIMARY KEY (FTChnCode));
CREATE TABLE IF NOT EXISTS TCNTPdtPmtHDChn (FTBchCode varchar (5) NOT NULL COLLATE NOCASE, FTPmhDocNo varchar (20) NOT NULL COLLATE NOCASE, FTChnCode varchar (5) NOT NULL COLLATE NOCASE, FTPmhStaType varchar (1) COLLATE NOCASE, PRIMARY KEY (FTBchCode, FTPmhDocNo, FTChnCode));
CREATE TABLE IF NOT EXISTS TCNMChannel_L (FTChnCode varchar (5) NOT NULL COLLATE NOCASE, FNLngID integer NOT NULL, FTChnName varchar (100) COLLATE NOCASE, FTChnRmk varchar (100) COLLATE NOCASE, PRIMARY KEY (FTChnCode, FNLngID));
CREATE TABLE IF NOT EXISTS TCNMAgency (FTAgnCode varchar (10) NOT NULL COLLATE NOCASE, FTPplCode varchar (20) NOT NULL COLLATE NOCASE, FTBchCode varchar (5) COLLATE NOCASE, FTAgnKeyAPI varchar (40) COLLATE NOCASE, FTAgnPwd varchar (30) COLLATE NOCASE, FTAgnEmail varchar (50) COLLATE NOCASE, FTAgnTel varchar (50) COLLATE NOCASE, FTAgnFax varchar (50) COLLATE NOCASE, FTAgnMo varchar (50) COLLATE NOCASE, FTAgnStaApv varchar (1) COLLATE NOCASE, FTAgnStaActive varchar (1) COLLATE NOCASE, FTAtyCode varchar (5) COLLATE NOCASE, FTAggCode varchar (5) COLLATE NOCASE, FTAgnRefCode varchar (20) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, FTChnCode varchar (5) COLLATE NOCASE, PRIMARY KEY (FTAgnCode));
CREATE TABLE IF NOT EXISTS TCNMChannelSpc (FTChnCode varchar (5) NOT NULL COLLATE NOCASE, FTAgnCode varchar (5) NOT NULL COLLATE NOCASE, FTAppCode varchar (5) NOT NULL COLLATE NOCASE, FNChnSeq integer NOT NULL, FTMerCode varchar (5) COLLATE NOCASE, FTShpCode varchar (5) COLLATE NOCASE, FTBchCode varchar (5) COLLATE NOCASE, FTPosCode varchar (5) COLLATE NOCASE, PRIMARY KEY (FTChnCode, FTAgnCode, FTAppCode, FNChnSeq));
CREATE TABLE IF NOT EXISTS TCNTConfigSpc (FTSysCode varchar (20) NOT NULL COLLATE NOCASE, FTSysApp varchar (50) NOT NULL COLLATE NOCASE, FTSysKey varchar (50) NOT NULL COLLATE NOCASE, FTSysSeq varchar (3) NOT NULL COLLATE NOCASE, FTAgnCode varchar (10) NOT NULL COLLATE NOCASE, FTSysStaDataType varchar (1) COLLATE NOCASE, FTCfgStaUsrValue varchar (255) COLLATE NOCASE, FTCfgStaUsrRef varchar (255) COLLATE NOCASE, PRIMARY KEY (FTSysCode, FTSysApp, FTSysKey, FTSysSeq, FTAgnCode));
CREATE TABLE IF NOT EXISTS TMPMImgPdt (FNImgID integer PRIMARY KEY AUTOINCREMENT NOT NULL, FTImgRefID varchar (30) COLLATE NOCASE, FNImgSeq integer, FTImgTable varchar (50) COLLATE NOCASE, FTImgKey varchar (20) COLLATE NOCASE, FTImgObj varchar (255) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TMPMPdtCostAvg (FNPdtID integer NOT NULL, FTPdtCode varchar (20) NOT NULL COLLATE NOCASE, FCPdtCostEx real, FCPdtCostIn real, FCPdtQtyBal real, FCPdtCostAmt real, FCPdtCostLast real, FDLastUpdOn datetime, PRIMARY KEY (FNPdtID, FTPdtCode));
CREATE TABLE IF NOT EXISTS TMPMPdtBar (FTPdtCode varchar (20) NOT NULL COLLATE NOCASE, FTBarCode varchar (25) NOT NULL COLLATE NOCASE, FNPdtID integer NOT NULL, FTPunCode varchar (5) NOT NULL COLLATE NOCASE, FTBarStaUse varchar (1) COLLATE NOCASE, FTBarStaAlwSale varchar (1) COLLATE NOCASE, FTBarStaByGen varchar (1) COLLATE NOCASE, FTBarStaUpd varchar (1) COLLATE NOCASE, FTPlcCode varchar (5) COLLATE NOCASE, FNPldSeq integer, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FTPdtCode, FTBarCode, FNPdtID));
CREATE TABLE IF NOT EXISTS TMPMPdt_L (FTPdtCode varchar (20) NOT NULL COLLATE NOCASE, FNPdtID integer NOT NULL, FNLngID integer NOT NULL, FTPdtName varchar (100) COLLATE NOCASE, FTPdtNameOth varchar (100) COLLATE NOCASE, FTPdtNameABB varchar (50) COLLATE NOCASE, FTPdtRmk varchar (200) COLLATE NOCASE, PRIMARY KEY (FTPdtCode, FNPdtID, FNLngID));
CREATE TABLE IF NOT EXISTS TMPMPdt (FNPdtID integer PRIMARY KEY AUTOINCREMENT NOT NULL, FTPdtCode varchar (20) COLLATE NOCASE, FTPdtStkControl varchar (1) COLLATE NOCASE, FTPdtGrpControl varchar (1) COLLATE NOCASE, FTPdtForSystem varchar (30) COLLATE NOCASE, FCPdtQtyOrdBuy real, FCPdtCostDef real, FCPdtCostOth real, FCPdtCostStd real, FCPdtMin real, FCPdtMax real, FTPdtPoint varchar (1) COLLATE NOCASE, FCPdtPointTime real, FTPdtType varchar (1) COLLATE NOCASE, FTPdtSaleType varchar (1) COLLATE NOCASE, FTPdtSetOrSN varchar (1) COLLATE NOCASE, FTPdtStaSetPri varchar (1) COLLATE NOCASE, FTPdtStaSetShwDT varchar (1) COLLATE NOCASE, FTPdtStaAlwDis varchar (1) COLLATE NOCASE, FTPdtStaAlwReturn varchar (1) COLLATE NOCASE, FTPdtStaVatBuy varchar (1) COLLATE NOCASE, FTPdtStaVat varchar (1) COLLATE NOCASE, FTPdtStaActive varchar (1) COLLATE NOCASE, FTPdtStaAlwReCalOpt varchar (1) COLLATE NOCASE, FTPdtStaCsm varchar (1) COLLATE NOCASE, FTTcgCode varchar (5) COLLATE NOCASE, FTPgpChain varchar (30) COLLATE NOCASE, FTPtyCode varchar (5) COLLATE NOCASE, FTPbnCode varchar (5) COLLATE NOCASE, FTPmoCode varchar (5) COLLATE NOCASE, FTVatCode varchar (5) COLLATE NOCASE, FTEvhCode varchar (5) COLLATE NOCASE, FDPdtSaleStart datetime, FDPdtSaleStop datetime, FTPdtStaSetPrcStk varchar (1) COLLATE NOCASE, FTPdtCtrlRole varchar (10) COLLATE NOCASE, FTPdtStaUpd varchar (1) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS TMPMPdtPackSize (FNPdtID integer NOT NULL, FTPdtCode varchar (20) NOT NULL COLLATE NOCASE, FTPunCode varchar (5) NOT NULL COLLATE NOCASE, FCPdtUnitFact real, FCPdtPriceRET real, FCPdtPriceWHS real, FCPdtPriceNET real, FTPdtGrade varchar (50) COLLATE NOCASE, FCPdtWeight real, FTClrCode varchar (5) COLLATE NOCASE, FTPszCode varchar (5) COLLATE NOCASE, FTPdtUnitDim varchar (50) COLLATE NOCASE, FTPdtPkgDim varchar (50) COLLATE NOCASE, FTPdtStaAlwPick varchar (1) COLLATE NOCASE, FTPdtStaAlwPoHQ varchar (1) COLLATE NOCASE, FTPdtStaAlwBuy varchar (1) COLLATE NOCASE, FTPdtStaAlwSale varchar (1) COLLATE NOCASE, FTPszStaUpd varchar (1) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, PRIMARY KEY (FNPdtID, FTPdtCode, FTPunCode));
CREATE TABLE IF NOT EXISTS TCNMPos (FTBchCode varchar (5) NOT NULL COLLATE NOCASE, FTPosCode varchar (5) NOT NULL COLLATE NOCASE, FTPosType varchar (30) NOT NULL COLLATE NOCASE, FTPosRegNo varchar (20) COLLATE NOCASE, FTSmgCode varchar (5) COLLATE NOCASE, FTPosStaRorW varchar (1) COLLATE NOCASE, FTPosStaPrnEJ varchar (1) COLLATE NOCASE, FTPosStaVatSend varchar (1) COLLATE NOCASE, FTPosStaUse varchar (1) COLLATE NOCASE, FTPosStaShift varchar (1) COLLATE NOCASE, FTPosStaDate varchar (1) COLLATE NOCASE, FTPosStaSumScan varchar (1) COLLATE NOCASE, FTPosStaSumPrn varchar (1) COLLATE NOCASE, FTPrgRegToken varchar (200) COLLATE NOCASE, FDLastUpdOn datetime, FTLastUpdBy varchar (20) COLLATE NOCASE, FDCreateOn datetime, FTCreateBy varchar (20) COLLATE NOCASE, FTChnCode varchar (5) COLLATE NOCASE, FNPosShwRow INTEGER, FNPosShwCol INTEGER, PRIMARY KEY (FTPosCode, FTBchCode));
CREATE TABLE IF NOT EXISTS android_metadata (locale TEXT);
CREATE TABLE IF NOT EXISTS "TCNTPdtPmtCG" (
	"FTBchCode"	varchar(5) NOT NULL COLLATE NOCASE,
	"FTPmhDocNo"	varchar(20) NOT NULL COLLATE NOCASE,
	"FNPgtSeq"	integer NOT NULL,
	"FTPmdGrpName"	varchar(50) COLLATE NOCASE,
	"FTPgtStaGetEffect"	varchar(1) COLLATE NOCASE,
	"FTPgtStaGetType"	varchar(1) COLLATE NOCASE,
	"FTPgtStaGetPdt"	varchar(1) COLLATE NOCASE,
	"FTRolCode"	varchar(5) COLLATE NOCASE,
	"FCPgtGetvalue"	real,
	"FTPplCode"	varchar(20) COLLATE NOCASE,
	"FCPgtGetQty"	real,
	"FCPgtPerAvgDis"	real,
	"FTPgtStaPoint"	varchar(1) COLLATE NOCASE,
	"FTPgtStaPntCalType"	varchar(1) COLLATE NOCASE,
	"FTPgtStaPdtDT"	varchar(1) COLLATE NOCASE,
	"FNPgtPntGet"	integer,
	"FNPgtPntBuy"	integer,
	"FTPgtStaCoupon"	varchar(1) COLLATE NOCASE,
	"FTPgtCpnText"	varchar(50) COLLATE NOCASE,
	"FTCphDocNo"	varchar(20) COLLATE NOCASE,
	"FTSplCode"	varchar(5),
	"FDPgtPntStart"	datetime,
	"FDPgtPntExpired"	datetime,
	PRIMARY KEY("FTBchCode","FTPmhDocNo","FNPgtSeq")
);
CREATE TABLE IF NOT EXISTS TTKMBchLocation(
	[FTAgnCode] [varchar](5) NOT NULL,
	[FTBchCode] [varchar](5) NOT NULL,
	[FTLocCode] [varchar](5) NOT NULL,
	[FTLocStaAlwPet] [varchar](1) NULL,
	[FTLocStaAlwBook] [varchar](1) NULL,
	[FCLocCapacity] [numeric](18, 4) NULL,
	[FTLocStaUse] [varchar](1) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTAgnCode] ASC,
		[FTBchCode] ASC,
		[FTLocCode] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMBchLocation_L(
	[FTLocCode] [varchar](5) NOT NULL,
	[FNLngID] [bigint] NOT NULL,
	[FTLocName] [varchar](100) NULL,
	[FTLocRmk] [varchar](100) NULL,
	PRIMARY KEY
	(
		[FTLocCode] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocation(
	[FTLocCode] [varchar](5) NOT NULL,
	[FTBchCode] [varchar](5) NOT NULL,
	[FNLocLimit] [bigint] NULL,
	[FTLocTimeOpening] [varchar](8) NULL,
	[FTLocTimeClosing] [varchar](8) NULL,
	[FTLocStaActive] [varchar](1) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTLocCode] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocation_L(
	[FTLocCode] [varchar](5) NOT NULL,
	[FNLngID] [bigint] NOT NULL,
	[FTHalName] [varchar](100) NULL,
	[FTHalDesc] [varchar](200) NULL,
	[FTHalRmk] [varchar](4000) NULL,
	PRIMARY KEY
	(
		[FTLocCode] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZone(
	[FTZneCode] [varchar](5) NOT NULL,
	[FTLocCode] [varchar](5) NULL,
	[FTLevCode] [varchar](5) NULL,
	[FTGteCode] [varchar](5) NULL,
	[FNZneRow] [int] NULL,
	[FNZneRowStart] [int] NULL,
	[FNZneIncrement] [int] NULL,
	[FNZneCol] [int] NULL,
	[FNZneColStart] [int] NULL,
	[FTZneBookingType] [varchar](1) NULL,
	[FNZneQtyAvai] [int] NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTZneCode] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZone_L(
	[FTZneCode] [varchar](5) NOT NULL,
	[FNLngID] [bigint] NOT NULL,
	[FTZneName] [varchar](255) NULL,
	[FTZneRmk] [varchar](100) NULL,
	PRIMARY KEY
	(
		[FTZneCode] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZnePdt(
	[FTLzbRefPdt] [varchar](20) NOT NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	[FTLocCode] [varchar](5) NOT NULL,
	[FTZneCode] [varchar](5) NOT NULL,
	PRIMARY KEY
	(
		[FTLocCode] ASC,
		[FTZneCode] ASC,
		[FTLzbRefPdt] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZneBook(
	[FTZneChain] [varchar](20) NOT NULL,
	[FNLzbAddrNo] [bigint] NOT NULL,
	[FTLzbRefPdt] [varchar](5) NOT NULL,
	[FNLzbRow] [bigint] NULL,
	[FNLzbCol] [bigint] NULL,
	[FTLzbStaAlwUse] [varchar](1) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTZneChain] ASC,
		[FNLzbAddrNo] ASC,
		[FTLzbRefPdt] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZneBook_L(
	[FTZneChain] [varchar](20) NOT NULL,
	[FNLzbAddrNo] [bigint] NOT NULL,
	[FTLzbRefPdt] [varchar](5) NOT NULL,
	[FNLngID] [bigint] NOT NULL,
	[FTLzbName] [varchar](100) NULL,
	[FTLzbNameRmk] [varchar](100) NULL,
	PRIMARY KEY
	(
		[FTZneChain] ASC,
		[FNLzbAddrNo] ASC,
		[FTLzbRefPdt] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZne(
		[FTZneChain] [varchar](30) NOT NULL,
		[FTAgnCode] [varchar](5) NOT NULL,
		[FTBchCode] [varchar](5) NOT NULL,
		[FTLocCode] [varchar](5) NOT NULL,
		[FTZneCode] [varchar](5) NOT NULL,
		[FTLevCode] [varchar](5) NULL,
		[FTGteCode] [varchar](5) NULL,
		[FCZneCapacity] [numeric](18, 4) NULL,
		[FTZneStaUse] [varchar](1) NULL,
		[FDLastUpdOn] [datetime] NULL,
		[FTLastUpdBy] [varchar](20) NULL,
		[FDCreateOn] [datetime] NULL,
		[FTCreateBy] [varchar](20) NULL,
	 PRIMARY KEY
	(
		[FTZneCode] ASC,
		[FTZneChain] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMLocZne_L(
		[FTZneCode] [varchar](5) NOT NULL,
		[FNLngID] [bigint] NOT NULL,
		[FTZneName] [varchar](100) NULL,
		[FTZneRmk] [varchar](100) NULL,
	PRIMARY KEY
	(
		[FTZneCode] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMPdtPkgGrp(
	[FTPdtCode] [varchar](20) NOT NULL,
	[FNPkdGrpSeqNo] [int] NOT NULL,
	[FTPdtCodeRef] [varchar](20) NOT NULL,
	[FTPkdType] [varchar](1) NULL,
	[FTPdtQtySetPkg] [varchar](5) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	[FNPkdPdtQty] [bigint] NULL,
	[FTPdtCodeRef2] [varchar](20) NOT NULL,
	 PRIMARY KEY
	(
		[FTPdtCode] ASC,
		[FNPkdGrpSeqNo] ASC,
		[FTPdtCodeRef] ASC,
		[FTPdtCodeRef2] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMPdtPkgGrp_L(
	[FTPdtCode] [varchar](20) NOT NULL,
	[FNPkdGrpSeqNo] [int] NOT NULL,
	[FNLngID] [bigint] NOT NULL,
	[FTPkdGrpName] [varchar](100) NULL,
	[FTPkdGrpRmk] [varchar](100) NULL,
	PRIMARY KEY
	(
		[FTPdtCode] ASC,
		[FNPkdGrpSeqNo] ASC,
		[FNLngID] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMPdtPkgCard(
	[FTPdtCode] [varchar](20) NOT NULL,
	[FTCrdType] [varchar](1) NULL,
	[FNCrdDayActive] [bigint] NULL,
	[FDCrdDateStart] [datetime] NULL,
	[FDCrdDateExpired] [datetime] NULL,
	[FNCrdQtyLimit] [bigint] NULL,
	[FTCrdStaAlwDayDup] [varchar](1) NULL,
	[FTCrdStaChkID] [varchar](1) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTPdtCode] ASC
	)
);
CREATE TABLE IF NOT EXISTS TTKMPdtPkgCond(
	[FTPdtCode] [varchar](20) NOT NULL,
	[FNPkdGrpSeqNo] [int] NOT NULL,
	[FTPkdType] [varchar](1) NULL,
	[FNPkcQtyAlw] [bigint] NULL,
	[FTPkcStaAlwDup] [varchar](1) NULL,
	[FTPkcStaSelOrChk] [varchar](1) NULL,
	[FDLastUpdOn] [datetime] NULL,
	[FTLastUpdBy] [varchar](20) NULL,
	[FDCreateOn] [datetime] NULL,
	[FTCreateBy] [varchar](20) NULL,
	PRIMARY KEY
	(
		[FTPdtCode] ASC,
		[FNPkdGrpSeqNo] ASC
	)
);
CREATE TABLE IF NOT EXISTS "TCNMPdt" (
	"FTPdtCode"	varchar(20) NOT NULL COLLATE NOCASE,
	"FTPdtStkControl"	varchar(1) COLLATE NOCASE,
	"FTPdtGrpControl"	varchar(1) COLLATE NOCASE,
	"FTPdtForSystem"	varchar(30) COLLATE NOCASE,
	"FCPdtQtyOrdBuy"	real,
	"FCPdtCostDef"	real,
	"FCPdtCostOth"	real,
	"FCPdtCostStd"	real,
	"FCPdtMin"	real,
	"FCPdtMax"	real,
	"FTPdtPoint"	varchar(1) COLLATE NOCASE,
	"FCPdtPointTime"	real,
	"FTPdtType"	varchar(1) COLLATE NOCASE,
	"FTPdtSaleType"	varchar(1) COLLATE NOCASE,
	"FTPdtSetOrSN"	varchar(1) COLLATE NOCASE,
	"FTPdtStaSetPri"	varchar(1) COLLATE NOCASE,
	"FTPdtStaSetShwDT"	varchar(1) COLLATE NOCASE,
	"FTPdtStaAlwDis"	varchar(1) COLLATE NOCASE,
	"FTPdtStaAlwReturn"	varchar(1) COLLATE NOCASE,
	"FTPdtStaVatBuy"	varchar(1) COLLATE NOCASE,
	"FTPdtStaVat"	varchar(1) COLLATE NOCASE,
	"FTPdtStaActive"	varchar(1) COLLATE NOCASE,
	"FTPdtStaAlwReCalOpt"	varchar(1) COLLATE NOCASE,
	"FTPdtStaCsm"	varchar(1) COLLATE NOCASE,
	"FTTcgCode"	varchar(5) COLLATE NOCASE,
	"FTPgpChain"	varchar(30) COLLATE NOCASE,
	"FTPtyCode"	varchar(5) COLLATE NOCASE,
	"FTPbnCode"	varchar(5) COLLATE NOCASE,
	"FTPmoCode"	varchar(5) COLLATE NOCASE,
	"FTVatCode"	varchar(5) COLLATE NOCASE,
	"FTEvhCode"	varchar(5) COLLATE NOCASE,
	"FDPdtSaleStart"	datetime,
	"FDPdtSaleStop"	datetime,
	"FTPdtStaSetPrcStk"	varchar(1) COLLATE NOCASE,
	"FTPdtCtrlRole"	varchar(10) COLLATE NOCASE,
	"FDLastUpdOn"	datetime,
	"FTLastUpdBy"	varchar(20) COLLATE NOCASE,
	"FDCreateOn"	datetime,
	"FTCreateBy"	varchar(20) COLLATE NOCASE,
	PRIMARY KEY("FTPdtCode")
);
CREATE TABLE IF NOT EXISTS TFHMPdtFabric (
    FTFabCode VARCHAR(5) NOT NULL,
    FTAgnCode VARCHAR(10) NOT NULL,
    FDLastUpdOn DATETIME NULL,
    FTLastUpdBy VARCHAR(20) NULL,
    FDCreateOn DATETIME NULL,
    FTCreateBy VARCHAR(20) NULL,
    PRIMARY KEY (FTFabCode)
);
CREATE TABLE IF NOT EXISTS TFHMPdtFabric_L (
    FTFabCode VARCHAR(5) NOT NULL,
    FNLngID INT NOT NULL,
    FTFabName TEXT NULL,
    FTFabRmk TEXT NULL,
    PRIMARY KEY (FTFabCode, FNLngID)
);
CREATE TABLE IF NOT EXISTS TFHMPdtSeason (
    FTSeaCode VARCHAR(5) NOT NULL,
    FTSeaChain VARCHAR(30) NOT NULL,
    FNSeaLevel INT NOT NULL,
    FTSeaParent VARCHAR(5),
    FTAgnCode VARCHAR(10),
    FDLastUpdOn DATETIME,
    FTLastUpdBy VARCHAR(20),
    FDCreateOn DATETIME,
    FTCreateBy VARCHAR(20),
    PRIMARY KEY (FTSeaCode, FTSeaChain)
);
CREATE TABLE IF NOT EXISTS TFHMPdtSeason_L (
    FTSeaCode VARCHAR(5) NOT NULL,
    FTSeaChain VARCHAR(30) NOT NULL,
    FNLngID INT NOT NULL,
    FTSeaName TEXT NULL,
    FTSeaChainName TEXT NULL,
    FTSeaRmk TEXT NULL,
    PRIMARY KEY (FTSeaCode, FTSeaChain, FNLngID)
);
CREATE TABLE IF NOT EXISTS TFHMPdtFhn (
    FTPdtCode VARCHAR(20) NOT NULL,
    FTDepCode VARCHAR(10) NOT NULL,
    FTClsCode VARCHAR(10) NOT NULL,
    FTSclCode VARCHAR(10) NOT NULL,
    FTPgpCode VARCHAR(10) NOT NULL,
    FTCmlCode VARCHAR(10) NOT NULL,
    FTFhnModNo VARCHAR(30) NOT NULL,
    FTFhnGender VARCHAR(1),
    FDLastUpdOn DATETIME,
    FTLastUpdBy VARCHAR(20),
    FDCreateOn DATETIME,
    FTCreateBy VARCHAR(20),
    PRIMARY KEY (FTPdtCode)
);
CREATE TABLE IF NOT EXISTS TFHMPdtColorSize (
    FTPdtCode VARCHAR(20) NOT NULL,
    FTFhnRefCode VARCHAR(50) NOT NULL,
    FNFhnSeq INT NOT NULL,
    FTSeaCode VARCHAR(5) NOT NULL,
    FTFabCode VARCHAR(5) NOT NULL,
    FTClrCode VARCHAR(5) NOT NULL,
    FTPszCode VARCHAR(5) NOT NULL,
    FCFhnCostStd NUMERIC(18, 4),
    FDFhnStart DATETIME,
    FTFhnStaActive VARCHAR(1),
    FCFhnCostOth NUMERIC(18, 4),
    FDLastUpdOn DATETIME,
    FTLastUpdBy VARCHAR(20),
    FDCreateOn DATETIME,
    FTCreateBy VARCHAR(20),
    PRIMARY KEY (FTPdtCode, FTFhnRefCode, FNFhnSeq)
);
CREATE TABLE IF NOT EXISTS "TCNMPosSpcCat" (
    FTBchCode varchar(5) NOT NULL COLLATE NOCASE,
    FTShpCode varchar(5) NOT NULL COLLATE NOCASE,
    FTPosCode varchar(5) NOT NULL COLLATE NOCASE,
    FNCatSeq integer NOT NULL,
    FTPdtCat1 varchar(10) COLLATE NOCASE,
    FTPdtCat2 varchar(10) COLLATE NOCASE,
    FTPdtCat3 varchar(10) COLLATE NOCASE,
    FTPdtCat4 varchar(10) COLLATE NOCASE,
    FTPdtCat5 varchar(10) COLLATE NOCASE,
    FTPgpChain varchar(30) COLLATE NOCASE,
    FTPtyCode varchar(5) COLLATE NOCASE,
    FTPbnCode varchar(5) COLLATE NOCASE,
    FTPmoCode varchar(5) COLLATE NOCASE,
    FTTcgCode varchar(5) COLLATE NOCASE,
    FDLastUpdOn datetime,
    FTLastUpdBy varchar(20) COLLATE NOCASE,
    FDCreateOn datetime,
    FTCreateBy varchar(20) COLLATE NOCASE,
PRIMARY KEY("FTBchCode","FTShpCode","FTPosCode","FNCatSeq")
);
CREATE TABLE IF NOT EXISTS "TCNMPdtBar" (
	"FTPdtCode"	varchar(20) NOT NULL COLLATE NOCASE,
	"FTBarCode"	varchar(25) NOT NULL COLLATE NOCASE,
	"FTPunCode"	varchar(5) NOT NULL COLLATE NOCASE,
	"FTBarStaUse"	varchar(1) COLLATE NOCASE,
	"FTBarStaAlwSale"	varchar(1) COLLATE NOCASE,
	"FTBarStaByGen"	varchar(1) COLLATE NOCASE,
	"FTPlcCode"	varchar(5) COLLATE NOCASE,
	"FNPldSeq"	integer,
	"FDLastUpdOn"	datetime,
	"FTLastUpdBy"	varchar(20) COLLATE NOCASE,
	"FDCreateOn"	datetime,
	"FTCreateBy"	varchar(20) COLLATE NOCASE,
	"FTFhnRefCode"	varchar(20),
	"FNBarRefSeq"	INTEGER,
	PRIMARY KEY("FTPdtCode","FTBarCode","FNBarRefSeq")
);
CREATE TABLE IF NOT EXISTS "TCNMAddress_L" (
	"FNLngID"	integer NOT NULL,
	"FTAddGrpType"	varchar(1) NOT NULL COLLATE NOCASE,
	"FTAddRefCode"	varchar(30) NOT NULL COLLATE NOCASE,
	"FNAddSeqNo"	integer NOT NULL,
	"FTAddRefNo"	varchar(20) COLLATE NOCASE,
	"FTAddName"	varchar(200) COLLATE NOCASE,
	"FTAddTaxNo"	varchar(20) COLLATE NOCASE,
	"FTAddRmk"	varchar(200) COLLATE NOCASE,
	"FTAddCountry"	varchar(100) COLLATE NOCASE,
	"FTAddVersion"	varchar(1) COLLATE NOCASE,
	"FTAddV1No"	varchar(30) COLLATE NOCASE,
	"FTAddV1Soi"	varchar(30) COLLATE NOCASE,
	"FTAddV1Village"	varchar(70) COLLATE NOCASE,
	"FTAddV1Road"	varchar(30) COLLATE NOCASE,
	"FTAddV1SubDist"	varchar(30) COLLATE NOCASE,
	"FTAddV1DstCode"	varchar(5) COLLATE NOCASE,
	"FTAddV1PvnCode"	varchar(5) COLLATE NOCASE,
	"FTAddV1PostCode"	varchar(5) COLLATE NOCASE,
	"FTAddTel"	varchar(50) COLLATE NOCASE,
	"FTAddFax"	varchar(50) COLLATE NOCASE,
	"FTAddV2Desc1"	varchar(255) COLLATE NOCASE,
	"FTAddV2Desc2"	varchar(255) COLLATE NOCASE,
	"FTAddWebsite"	varchar(200) COLLATE NOCASE,
	"FTAddLongitude"	varchar(50) COLLATE NOCASE,
	"FTAddLatitude"	varchar(50) COLLATE NOCASE,
	"FDLastUpdOn"	datetime,
	"FTLastUpdBy"	varchar(20) COLLATE NOCASE,
	"FDCreateOn"	datetime,
	"FTCreateBy"	varchar(20) COLLATE NOCASE,
	"FTAgnCode"	varchar(10), FTAddType TEXT, FTAddStaActive TEXT, FTAddTaxNoHeader TEXT, FTAddFullText TEXT, FTAddV2Desc3 TEXT, FTAddV1Moo TEXT, FTAddV1Building TEXT, FTAddV1Floor TEXT, FTAddV1Room TEXT, FTAddV1DstName TEXT, FTAddV1PvnName TEXT, FTAddV1CtrName TEXT,
	PRIMARY KEY("FNLngID","FTAddGrpType","FTAddRefCode","FNAddSeqNo","FTAgnCode")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtCalAlwDT" (
 "FTAgnCode" varchar(10) NOT NULL COLLATE NOCASE,
 "FTBchCode" varchar(5) NOT NULL COLLATE NOCASE,
 "FTPahDocNo" varchar(20) NOT NULL COLLATE NOCASE,
 "FNPadSeqNo" integer NOT NULL,
 "FTPadType" varchar(1) COLLATE NOCASE,
 "FTPadDow" varchar(1) COLLATE NOCASE,
 "FDPadDateFrm" datetime,
 "FDPadDateTo" datetime,
 "FTPadStaAlw" varchar(1) COLLATE NOCASE,
 "FDLastUpdOn" datetime,
 "FTLastUpdBy" varchar(50) COLLATE NOCASE,
 "FDCreateOn" datetime,
 "FTCreateBy" varchar(50) COLLATE NOCASE, FTPadRefCodeOld NUMERIC, FTPadRefCodeNew NUMERIC, FTPadRefRmk TEXT,
 PRIMARY KEY("FTAgnCode", "FTBchCode", "FTPahDocNo", "FNPadSeqNo")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtCalAlwHD" (
 "FTAgnCode" varchar(10) NOT NULL COLLATE NOCASE,
 "FTBchCode" varchar(5) NOT NULL COLLATE NOCASE,
 "FTPahDocNo" varchar(20) NOT NULL COLLATE NOCASE,
 "FTPahTypeAlw" nvarchar(1) COLLATE NOCASE,
 "FDPahStart" datetime,
 "FDPahFinish" datetime,
 "FDLastUpdOn" datetime,
 "FTLastUpdBy" varchar(50) COLLATE NOCASE,
 "FDCreateOn" datetime,
 "FTCreateBy" varchar(50) COLLATE NOCASE,
 PRIMARY KEY("FTAgnCode", "FTBchCode", "FTPahDocNo")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtCalAlwHD_L" (
 "FTAgnCode" varchar(10) NOT NULL COLLATE NOCASE,
 "FTBchCode" varchar(5) NOT NULL COLLATE NOCASE,
 "FTPahDocNo" varchar(20) NOT NULL COLLATE NOCASE,
 "FNLngID" integer NOT NULL,
 "FTPahName" varchar(100) COLLATE NOCASE,
 "FTPmhRmk" varchar(200) COLLATE NOCASE,
 PRIMARY KEY("FTAgnCode", "FTBchCode", "FTPahDocNo", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNMBchAotConfig" (
  "FTBchCode" TEXT NOT NULL,
  "FTBchRCShopCode" TEXT,
  "FTBchRCCompanyCode" TEXT,
  "FTBchRCRcServerUrl" TEXT,
  "FTBchRCSenderMail" TEXT,
  "FTBchRCSenderPass" TEXT,
  "FTBchRCReceiverMail" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTBchRCBchCode" TEXT,
  PRIMARY KEY ("FTBchCode", "FTAgnCode")
);
CREATE TABLE IF NOT EXISTS "TCNMCstAddress" (
  "FTAgnCode" TEXT NOT NULL,
  "FTCstCode" TEXT NOT NULL,
  "FNAddSeqNo" INTEGER NOT NULL,
  "FTAddType" TEXT NOT NULL,
  "FTAddFmtType" TEXT NOT NULL,
  "FTAddTaxNo" TEXT,
  "FTAddStaActive" TEXT,
  "FTAddStaDefault" TEXT,
  "FTAddStaPrimary" TEXT,
  "FTCtyCode" TEXT,
  "FTPvnCode" TEXT,
  "FTDstCode" TEXT,
  "FTSudCode" TEXT,
  "FTPosCode" TEXT,
  "FTAddLongitude" TEXT,
  "FTAddLatitude" TEXT,
  "FTAddRefNo" TEXT,
  "FTAddMapUrl" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTCstCode", "FNAddSeqNo")
);
CREATE TABLE IF NOT EXISTS "TCNMSplAddress" (
  "FTAgnCode" TEXT NOT NULL,
  "FTSplCode" TEXT NOT NULL,
  "FNAddSeqNo" INTEGER NOT NULL,
  "FTAddType" TEXT NOT NULL,
  "FTAddFmtType" TEXT NOT NULL,
  "FTAddStaActive" TEXT,
  "FTAddLongitude" TEXT,
  "FTAddStaDefault" TEXT,
  "FTAddStaPrimary" TEXT,
  "FTCtyCode" TEXT,
  "FTPvnCode" TEXT,
  "FTDstCode" TEXT,
  "FTSudCode" TEXT,
  "FTPosCode" TEXT,
  "FTAddLatitude" TEXT,
  "FTAddRefNo" TEXT,
  "FTAddMapUrl" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTAddTaxNo" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTSplCode", "FNAddSeqNo")
);
CREATE TABLE IF NOT EXISTS "TPSMFuncHD" (
  "FTGhdCode" TEXT NOT NULL,
  "FTGhdApp" TEXT NOT NULL,
  "FTKbdScreen" TEXT NOT NULL,
  "FTKbdGrpName" TEXT NOT NULL,
  "FNGhdMaxPerPage" INTEGER,
  "FTGhdLayOut" TEXT,
  "FNGhdMaxLayOutX" INTEGER,
  "FNGhdMaxLayOutY" INTEGER,
  "FTGhdStaAlwChg" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTGhdCode")
);
CREATE TABLE IF NOT EXISTS "TPSMFuncDT" (
  "FTGhdCode" TEXT NOT NULL,
  "FTSysCode" TEXT NOT NULL,
  "FTLicPdtCode" TEXT,
  "FNGdtPage" INTEGER,
  "FNGdtDefSeq" INTEGER,
  "FNGdtUsrSeq" INTEGER,
  "FNGdtBtnSizeX" INTEGER,
  "FNGdtBtnSizeY" INTEGER,
  "FTGdtCallByName" TEXT,
  "FTGdtStaUse" TEXT,
  "FNGdtFuncLevel" INTEGER,
  "FTGdtSysUse" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTGhdCode", "FTSysCode")
);
CREATE TABLE IF NOT EXISTS "TPSMFuncDT_L" (
  "FTGhdCode" TEXT NOT NULL,
  "FTSysCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTGdtName" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTGhdCode", "FTSysCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNMCountry" (
  "FTCtyCode" TEXT NOT NULL,
  "FTVatCode" TEXT,
  "FNLngID" INTEGER,
  "FTCtyLongitude" TEXT,
  "FTCtyLatitude" TEXT,
  "FTCtyStaUse" TEXT,
  "FTRteIsoCode" TEXT,
  "FTCtyStaCtrlRate" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTCtyRefID" TEXT,
  "FTCtyCode2" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTCtyCode")
);
CREATE TABLE IF NOT EXISTS "TCNMCountry_L" (
  "FTCtyCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTCtyName" TEXT,
  "FTCtyRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTCtyCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNMNation" (
  "FTNatCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTNatCode")
);
CREATE TABLE IF NOT EXISTS "TCNMNation_L" (
  "FTNatCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTNatName" TEXT,
  "FTNatRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTNatCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNMPdtScale" (
  "FTAgnCode" TEXT NOT NULL,
  "FTPdsCode" TEXT NOT NULL,
  "FTPdsMatchStr" TEXT,
  "FNPdsLenBar" INTEGER,
  "FNPdsLenPdt" INTEGER,
  "FNPdsPdtStart" INTEGER,
  "FNPdsLenPri" INTEGER,
  "FNPdsPriStart" INTEGER,
  "FNPdsPriDec" INTEGER,
  "FNPdsLenWeight" INTEGER,
  "FNPdsWeightStart" INTEGER,
  "FNPdsWeightDec" INTEGER,
  "FTPdsStaChkDigit" TEXT,
  "FTPdsStaUse" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTPdsCode")
);
CREATE TABLE IF NOT EXISTS "TCNMPosAotConfig" (
  "FTPosCode" TEXT NOT NULL,
  "FTBchCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FTAotRcPosIp" TEXT,
  "FTAotRcClientId" TEXT,
  "FTAotRcClientSecret" TEXT,
  "FTAotMacAddress" TEXT,
  "FTAotCpuSerialNo" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTPosCode", "FTBchCode")
);
CREATE TABLE IF NOT EXISTS "TCNSAppConfig" (
  "FTAppCode" TEXT NOT NULL,
  "FTCNKey" TEXT NOT NULL,
  "FNCNSeq" INTEGER NOT NULL,
  "FTCNStaOnline" TEXT,
  "FTCNStaValue" TEXT,
  "FTCNStaUse" TEXT,
  PRIMARY KEY ("FTAppCode", "FTCNKey", "FNCNSeq")
);
CREATE TABLE IF NOT EXISTS "TCNTConfig" (
  "FTSysCode" TEXT NOT NULL,
  "FTSysApp" TEXT NOT NULL,
  "FTSysKey" TEXT NOT NULL,
  "FTSysSeq" TEXT NOT NULL,
  "FTGmnCode" TEXT,
  "FTSysStaAlwEdit" TEXT,
  "FTSysStaDataType" TEXT,
  "FNSysMaxLength" TEXT,
  "FTSysStaDefValue" TEXT,
  "FTSysStaDefRef" TEXT,
  "FTSysStaUsrValue" TEXT,
  "FTSysStaUsrRef" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTSysCode", "FTSysApp", "FTSysKey", "FTSysSeq")
);
CREATE TABLE IF NOT EXISTS "TCNTConfig_L" (
  "FTSysCode" TEXT NOT NULL,
  "FTSysApp" TEXT NOT NULL,
  "FTSysKey" TEXT NOT NULL,
  "FTSysSeq" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTSysName" TEXT,
  "FTSysDesc" TEXT,
  "FTSysRmk" TEXT,
  PRIMARY KEY ("FTSysCode", "FTSysApp", "FTSysKey", "FTSysSeq", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNSEdcRcv" (
  "FTFmtCode" TEXT NOT NULL,
  "FTSedCode" TEXT NOT NULL,
  "FTErcType" TEXT NOT NULL,
  "FTErcKey" TEXT NOT NULL,
  "FTErcTransCode" TEXT NOT NULL,
  "FNErcSeq" INTEGER NOT NULL,
  "FTErcFedType" TEXT,
  "FTErcValType" TEXT,
  "FNErcLenData" INTEGER,
  "FTErcValue" TEXT,
  PRIMARY KEY ("FTFmtCode", "FTSedCode", "FTErcType", "FTErcKey", "FTErcTransCode", "FNErcSeq")
);
CREATE TABLE IF NOT EXISTS "TCNSJobTask" (
  "FTAgnCode" TEXT NOT NULL,
  "FTJobRefTbl" TEXT NOT NULL,
  "FDJobDateCfm" TEXT,
  "FTJobStaUse" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTJobRefTbl")
);
CREATE TABLE IF NOT EXISTS "TCNSLanguage" (
  "FNLngID" INTEGER NOT NULL,
  "FTLngName" TEXT NOT NULL,
  "FTLngNameEng" TEXT NOT NULL,
  "FTLngShortName" TEXT,
  "FTLngStaLocal" TEXT,
  "FTLngStaUse" TEXT,
  PRIMARY KEY ("FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNSRate_L" (
  "FTRteIsoCode" TEXT NOT NULL,
  "FTRteIsoName" TEXT,
  "FNLngID" INTEGER NOT NULL,
  "FTFmtCode" TEXT,
  "FTRteIsoRmk" TEXT,
  "FTRteUnitName" TEXT,
  "FTRteSubUnitName" TEXT,
  PRIMARY KEY ("FTRteIsoCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNSTxnApiFunc" (
  "FTApiFmtCode" TEXT NOT NULL,
  "FNApiSeqNo" INTEGER NOT NULL,
  "FTApiKey" TEXT,
  "FTApiFuncPath" TEXT,
  PRIMARY KEY ("FTApiFmtCode", "FNApiSeqNo")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF1Depart" (
  "FTDepCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTDepCode")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF1Depart_L" (
  "FTDepCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTDepName" TEXT,
  "FTDepRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTDepCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF2Class" (
  "FTClsCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTClsCode")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF2Class_L" (
  "FTClsCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTClsName" TEXT,
  "FTClsRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTClsCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF3SubClass" (
  "FTSclCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTSclCode")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF3SubClass_L" (
  "FTSclCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTSclName" TEXT,
  "FTSclRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTSclCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF4Group" (
  "FTPgpCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTPgpCode")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF4Group_L" (
  "FTPgpCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTPgpName" TEXT,
  "FTPgpRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTPgpCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF5ComLines" (
  "FTCmlCode" TEXT NOT NULL,
  "FTAgnCode" TEXT NOT NULL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTCmlCode")
);
CREATE TABLE IF NOT EXISTS "TFHMPdtF5ComLines_L" (
  "FTCmlCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTCmlName" TEXT,
  "FTCmlRmk" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTAgnCode", "FTCmlCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TFNMInstallment" (
  "FTAgnCode" TEXT NOT NULL,
  "FTStmCode" TEXT NOT NULL,
  "FCStmLimit" NUMERIC,
  "FNStmQty" INTEGER,
  "FCStmRate" NUMERIC,
  "FTStmStaUnit" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTStmCode")
);
CREATE TABLE IF NOT EXISTS "TFNMInstallmentSub" (
  "FTAgnCode" TEXT NOT NULL,
  "FTStmCode" TEXT NOT NULL,
  "FTCrdCode" TEXT NOT NULL,
  "FTBnkCode" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTStmCode", "FTCrdCode")
);
CREATE TABLE IF NOT EXISTS "TFNMInstallment_L" (
  "FTAgnCode" TEXT NOT NULL,
  "FTStmCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTStmName" TEXT,
  "FTStmRmk" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTStmCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TLKMWaHouse" (
  "FTAgnCode" TEXT NOT NULL,
  "FTBchCode" TEXT NOT NULL,
  "FTWahCode" TEXT NOT NULL,
  "FTWahRefNo" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  "FTShpCode" TEXT,
  "FTWahStaChannel" TEXT,
  "FTWahRemark" TEXT,
  "FTWahStaOnline" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTWahCode")
);
CREATE TABLE IF NOT EXISTS "TSysPmt" (
  "FTSpmCode" TEXT NOT NULL,
  "FTSpmType" TEXT,
  "FTSpmStaGrpBuy" TEXT,
  "FTSpmStaBuy" TEXT,
  "FTSpmStaGrpRcv" TEXT,
  "FTSpmStaRcv" TEXT,
  "FTSpmStaGrpBoth" TEXT,
  "FTSpmStaGrpReject" TEXT,
  "FTSpmStaAllPdt" TEXT,
  "FTSpmStaExceptPmt" TEXT,
  "FTSpmStaGetNewPri" TEXT,
  "FTSpmStaGetDisAmt" TEXT,
  "FTSpmStaGetDisPer" TEXT,
  "FTSpmStaGetPoint" TEXT,
  "FTSpmStaRcvFree" TEXT,
  "FTSpmStaAlwOffline" TEXT,
  "FTSpmStaChkLimitGet" TEXT,
  "FTSpmStaChkCst" TEXT,
  "FTSpmStaChkCstDOB" TEXT,
  "FTSpmStaUseRange" TEXT,
  "FNSpmLimitGrpRcv" INTEGER,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTSpmCode")
);
CREATE TABLE IF NOT EXISTS "TSysPmt_L" (
  "FTSpmCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTSpmName" TEXT,
  "FTSpmRmk" TEXT,
  PRIMARY KEY ("FTSpmCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TSysPrinter" (
  "FTSppCode" TEXT NOT NULL,
  "FTSppValue" TEXT,
  "FTSppRef" TEXT,
  "FTSppType" TEXT,
  "FTSppStaUse" TEXT,
  "FDLastUpdOn" TEXT,
  "FDCreateOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTSppCode")
);
CREATE TABLE IF NOT EXISTS "TSysPrinter_L" (
  "FTSppCode" TEXT NOT NULL,
  "FNLngID" INTEGER NOT NULL,
  "FTSppName" TEXT,
  PRIMARY KEY ("FTSppCode", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TVDTPdtHotSale" (
  "FTBchCode" TEXT NOT NULL,
  "FTShpCode" TEXT NOT NULL,
  "FTPosCode" TEXT NOT NULL,
  "FTPdtCode" TEXT NOT NULL,
  "FCXsdQty" NUMERIC,
  "FDCreateOn" TEXT,
  "FTAgnCode" TEXT NOT NULL,
  PRIMARY KEY ("FTBchCode", "FTShpCode", "FTPosCode", "FTPdtCode", "FTAgnCode")
);
CREATE TABLE IF NOT EXISTS "TCNMPdtSpl" (
    "FTAgnCode" TEXT NOT NULL DEFAULT '',
    "FTPdtCode" TEXT NOT NULL,
    "FTBarCode" TEXT NOT NULL DEFAULT '',
    "FNBarRefSeq" INTEGER NOT NULL DEFAULT 1,
    "FTSplCode" TEXT NOT NULL,
    "FCSplLastPrice" REAL,
    "FDSplLastDate" TEXT,
    "FTUsrCode" TEXT,
    "FTSplStaAlwPO" TEXT,
    "FDPdtAlwOrdStart" TEXT,
    "FDPdtAlwOrdStop" TEXT,
    "FTPdtOrdDay" TEXT,
    "FTPdtStaAlwOrdSun" TEXT,
    "FTPdtStaAlwOrdMon" TEXT,
    "FTPdtStaAlwOrdTue" TEXT,
    "FTPdtStaAlwOrdWed" TEXT,
    "FTPdtStaAlwOrdThu" TEXT,
    "FTPdtStaAlwOrdFri" TEXT,
    "FTPdtStaAlwOrdSat" TEXT,
    "FCPdtLeadTime" REAL,
    PRIMARY KEY ("FTAgnCode", "FTPdtCode", "FTBarCode", "FNBarRefSeq", "FTSplCode")
  );
CREATE TABLE IF NOT EXISTS "TCNMPdtFhn" (
    "FTAgnCode" TEXT NOT NULL DEFAULT '',
    "FTPdtCode" TEXT NOT NULL,
    "FTPgpChain" TEXT,
    "FTDcsCode" TEXT,
    "FTClrCode" TEXT,
    "FTPszCode" TEXT,
    "FTPdtArticle" TEXT,
    "FTDepCode" TEXT,
    "FTClsCode" TEXT,
    "FTSclCode" TEXT,
    "FDLastUpdOn" TEXT,
    "FTLastUpdBy" TEXT,
    "FDCreateOn" TEXT,
    "FTCreateBy" TEXT,
    PRIMARY KEY ("FTAgnCode", "FTPdtCode")
  );
CREATE TABLE IF NOT EXISTS "TCNMPdtPolicy" (
      "FTAgnCode" TEXT NOT NULL,
      "FTPdtCode" TEXT NOT NULL,
      "FNPolSeq" INTEGER NOT NULL,
      "FTPolType" TEXT NOT NULL,
      "FTPolMode" TEXT NOT NULL,
      "FTPolRef1Code" TEXT NOT NULL,
      "FTPolRef2Code" TEXT,
      "FTPolStaUse" TEXT NOT NULL DEFAULT '1',
      "FTPolRmk" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTPdtCode", "FNPolSeq")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtFashionVariant" (
      "FTAgnCode" TEXT NOT NULL,
      "FTPdtCode" TEXT NOT NULL,
      "FTFhnVarCode" TEXT NOT NULL,
      "FTPdtVarSKU" TEXT NOT NULL,
      "FTFhnSeasonCode" TEXT,
      "FTFhnSeasonName" TEXT,
      "FTFhnFabricCode" TEXT,
      "FTFhnFabricName" TEXT,
      "FTFhnColorCode" TEXT,
      "FTFhnColorName" TEXT,
      "FTFhnSizeCode" TEXT,
      "FTFhnSizeName" TEXT,
      "FTFhnBarcode" TEXT,
      "FDFhnStart" TEXT,
      "FTFhnImageUrl" TEXT,
      "FTFhnStaActive" TEXT NOT NULL DEFAULT '1',
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTPdtCode", "FTFhnVarCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtProfRef" (
      "FTAgnCode" TEXT NOT NULL,
      "FTPdtCode" TEXT NOT NULL,
      "FTSpfCode" TEXT,
      "FTWpfCode" TEXT,
      "FTLepCode" TEXT,
      "FTPrfStaActive" TEXT NOT NULL DEFAULT '1',
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTPdtCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtSrnCtrlProf" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTSpfCode" TEXT NOT NULL,
      "FTSpfTrackType" TEXT NOT NULL DEFAULT '0',
      "FTSpfReqSerial" TEXT NOT NULL DEFAULT '0',
      "FTSpfUniqueSerial" TEXT NOT NULL DEFAULT '0',
      "FTSpfStaCtrl" TEXT NOT NULL DEFAULT '1',
      "FTSpfReqReceive" TEXT NOT NULL DEFAULT '0',
      "FTSpfReqTrOut" TEXT NOT NULL DEFAULT '0',
      "FTSpfReqTrIn" TEXT NOT NULL DEFAULT '0',
      "FTSpfReqSale" TEXT NOT NULL DEFAULT '0',
      "FTSpfAssignPick" TEXT NOT NULL DEFAULT '0',
      "FTSpfCostMethod" TEXT NOT NULL DEFAULT '0',
      "FTSpfCostTrace" TEXT NOT NULL DEFAULT '0',
      "FTSpfTraceSpl" TEXT NOT NULL DEFAULT '0',
      "FTSpfTraceCst" TEXT NOT NULL DEFAULT '0',
      "FTSpfTraceDoc" TEXT NOT NULL DEFAULT '0',
      "FTSpfRecallLevel" TEXT NOT NULL DEFAULT '0',
      "FTSpfBlockSale" TEXT NOT NULL DEFAULT '0',
      "FTSpfBlockTr" TEXT NOT NULL DEFAULT '0',
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTSpfCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtSrnCtrlProf_L" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTSpfCode" TEXT NOT NULL,
      "FNLngID" INTEGER NOT NULL,
      "FTSpfName" TEXT,
      "FTSpfRmk" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTSpfCode", "FNLngID")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtWtyProf" (
      "FTAgnCode" TEXT NOT NULL,
      "FTWpfCode" TEXT NOT NULL,
      "FTWpfStaActive" TEXT,
      "FTWpfStaWty" TEXT,
      "FTWpfStartRule" TEXT,
      "FNWpfCustVal" INTEGER,
      "FTWpfCustUnit" TEXT,
      "FTWpfVoidAllow" TEXT,
      "FTWpfTransfer" TEXT,
      "FTWpfReplaceMode" TEXT,
      "FTWpfReqOldSrn" TEXT,
      "FTWpfReqNewSrn" TEXT,
      "FTWpfDecisionLog" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTWpfCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtWtyProf_L" (
      "FTAgnCode" TEXT NOT NULL,
      "FTWpfCode" TEXT NOT NULL,
      "FNLngID" INTEGER NOT NULL,
      "FTWpfName" TEXT,
      "FTWpfRmk" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTWpfCode", "FNLngID")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtWtyProfRoute" (
      "FTAgnCode" TEXT NOT NULL,
      "FTWpfCode" TEXT NOT NULL,
      "FNWprSeq" INTEGER NOT NULL,
      "FTWprRouteType" TEXT,
      "FNWprL1Val" INTEGER,
      "FTWprL1Unit" TEXT,
      "FNWprCustVal" INTEGER,
      "FTWprCustUnit" TEXT,
      "FTWprStartEvent" TEXT,
      "FTWprL1Start" TEXT,
      "FTWprLiabilityRule" TEXT,
      "FTWprStaActive" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTWpfCode", "FNWprSeq")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtLotExpProf" (
      "FTAgnCode" TEXT NOT NULL,
      "FTLepCode" TEXT NOT NULL,
      "FTLepStaActive" TEXT NOT NULL DEFAULT '1',
      "FTLepStaLotReq" TEXT NOT NULL DEFAULT '0',
      "FTLepStaBatchReq" TEXT NOT NULL DEFAULT '0',
      "FTLepStaDupLot" TEXT NOT NULL DEFAULT '0',
      "FTLepStaMfgReq" TEXT NOT NULL DEFAULT '0',
      "FTLepStaExpReq" TEXT NOT NULL DEFAULT '0',
      "FNLepShelfLifeDays" INTEGER,
      "FTLepExpCalcMethod" TEXT NOT NULL DEFAULT 'EXP_DATE',
      "FNLepNearExpWarnDays" INTEGER,
      "FTLepStaBlockExpiredSale" TEXT NOT NULL DEFAULT '0',
      "FTLepIssueMethod" TEXT NOT NULL DEFAULT 'FIFO',
      "FTLepStaManualLotSelect" TEXT NOT NULL DEFAULT '0',
      "FTLepStaReqSupplierLot" TEXT NOT NULL DEFAULT '0',
      "FTLepStaQCReq" TEXT NOT NULL DEFAULT '0',
      "FTLepStaQuarantineReq" TEXT NOT NULL DEFAULT '0',
      "FTLepStaReturnSameLot" TEXT NOT NULL DEFAULT '0',
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTLepCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMPdtLotExpProf_L" (
      "FTAgnCode" TEXT NOT NULL,
      "FTLepCode" TEXT NOT NULL,
      "FNLngID" INTEGER NOT NULL,
      "FTLepName" TEXT,
      "FTLepRmk" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTLepCode", "FNLngID")
    );
CREATE TABLE IF NOT EXISTS TCNTShopContract (
    FTAgnCode TEXT NOT NULL,
    FTBchCode TEXT NOT NULL,
    FTShpCode TEXT NOT NULL,
    FTShpCtrNo TEXT NOT NULL,
    FTMerCode TEXT NOT NULL DEFAULT '',
    FTStcRefMerType TEXT NOT NULL DEFAULT '',
    FTStcRefCode TEXT NOT NULL DEFAULT '',
    FTStcCsmModel TEXT NOT NULL DEFAULT '',
    FTPplCode TEXT NOT NULL DEFAULT '',
    FTShpCtrSpn TEXT NOT NULL DEFAULT '',
    FDShpCtrStart TEXT,
    FDShpCtrEnd TEXT,
    FDShpCtrSignDate TEXT,
    FTShpCtrStaActive TEXT NOT NULL DEFAULT '1',
    FTShpCtrStaDoc TEXT NOT NULL DEFAULT '0',
    FTShpCtrSettleType TEXT NOT NULL DEFAULT '1',
    FCShpCtrMinGuarantee NUMERIC NOT NULL DEFAULT 0,
    FCShpCtrMinRent NUMERIC NOT NULL DEFAULT 0,
    FCShpCtrDeposit NUMERIC NOT NULL DEFAULT 0,
    FTShpCtrBillCycle TEXT NOT NULL DEFAULT '',
    FTShpCtrSettleCycle TEXT NOT NULL DEFAULT '',
    FTShpCtrRmk TEXT NOT NULL DEFAULT '',
    FDCreateOn TEXT,
    FTCreateBy TEXT,
    FDLastUpdOn TEXT,
    FTLastUpdBy TEXT,
    PRIMARY KEY (FTAgnCode, FTBchCode, FTShpCode, FTShpCtrNo)
  );
CREATE TABLE IF NOT EXISTS TCNTShopContractGP (
    FTAgnCode TEXT NOT NULL,
    FTBchCode TEXT NOT NULL,
    FTShpCode TEXT NOT NULL,
    FTShpCtrNo TEXT NOT NULL,
    FNSgpSeq INTEGER NOT NULL,
    FTSgpScopeType TEXT NOT NULL DEFAULT 'A',
    FTPdtCode TEXT NOT NULL DEFAULT '',
    FTPgpChain TEXT NOT NULL DEFAULT '',
    FTPbnCode TEXT NOT NULL DEFAULT '',
    FCSgpPerAvg NUMERIC NOT NULL DEFAULT 0,
    FCSgpPerSun NUMERIC,
    FCSgpPerMon NUMERIC,
    FCSgpPerTue NUMERIC,
    FCSgpPerWed NUMERIC,
    FCSgpPerThu NUMERIC,
    FCSgpPerFri NUMERIC,
    FCSgpPerSat NUMERIC,
    FTSgpSettleType TEXT,
    FTSgpStaActive TEXT NOT NULL DEFAULT '1',
    FDCreateOn TEXT,
    FTCreateBy TEXT,
    FDLastUpdOn TEXT,
    FTLastUpdBy TEXT,
    PRIMARY KEY (FTAgnCode, FTBchCode, FTShpCode, FTShpCtrNo, FNSgpSeq)
  );
CREATE TABLE IF NOT EXISTS TCNTShopContractArea (
    FTAgnCode TEXT NOT NULL,
    FTBchCode TEXT NOT NULL,
    FTShpCode TEXT NOT NULL,
    FTShpCtrNo TEXT NOT NULL,
    FNShpAreaSeq INTEGER NOT NULL,
    FTZneCode TEXT NOT NULL DEFAULT '',
    FTFloorCode TEXT NOT NULL DEFAULT '',
    FTShpAreaNo TEXT NOT NULL DEFAULT '',
    FCShpAreaSize NUMERIC NOT NULL DEFAULT 0,
    FTShpAreaUnit TEXT NOT NULL DEFAULT 'sqm',
    FCShpRentRate NUMERIC NOT NULL DEFAULT 0,
    FCShpRentAmt NUMERIC NOT NULL DEFAULT 0,
    FTShpAreaRmk TEXT NOT NULL DEFAULT '',
    FDCreateOn TEXT,
    FTCreateBy TEXT,
    FDLastUpdOn TEXT,
    FTLastUpdBy TEXT,
    PRIMARY KEY (FTAgnCode, FTBchCode, FTShpCode, FTShpCtrNo, FNShpAreaSeq)
  );
CREATE TABLE IF NOT EXISTS TCNTShopContractPdt (
    FTAgnCode TEXT NOT NULL,
    FTBchCode TEXT NOT NULL,
    FTShpCode TEXT NOT NULL,
    FTShpCtrNo TEXT NOT NULL,
    FTShpPdtScopeType TEXT NOT NULL,
    FTShpPdtRefCode TEXT NOT NULL,
    FDShpPdtStart TEXT,
    FDShpPdtEnd TEXT,
    FTShpPdtStaActive TEXT NOT NULL DEFAULT '1',
    FTShpPdtRmk TEXT NOT NULL DEFAULT '',
    FDCreateOn TEXT,
    FTCreateBy TEXT,
    FDLastUpdOn TEXT,
    FTLastUpdBy TEXT,
    PRIMARY KEY (
      FTAgnCode, FTBchCode, FTShpCode, FTShpCtrNo,
      FTShpPdtScopeType, FTShpPdtRefCode
    )
  );
CREATE TABLE IF NOT EXISTS "TCNTPmtHD" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FNPmhVersion" INTEGER NOT NULL DEFAULT 0,
  "FTPmhStaActive" TEXT COLLATE NOCASE NOT NULL DEFAULT '1',
  "FTPmhName" TEXT COLLATE NOCASE,
  "FTPmhRmk" TEXT COLLATE NOCASE,
  "FTPmhStaDoc" TEXT COLLATE NOCASE,
  "FTPmhStaPrcDoc" TEXT COLLATE NOCASE,
  "FDPmhDStart" TEXT,
  "FDPmhDStop" TEXT,
  "FNPmhPriority" INTEGER NOT NULL DEFAULT 100,
  "FTPmhStaStack" TEXT COLLATE NOCASE,
  "FTPmhFmtCode" TEXT COLLATE NOCASE,
  "FNPmhSchemaVer" INTEGER NOT NULL DEFAULT 1,
  "FTPmhRuleJson" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmhVersion")
);
CREATE TABLE IF NOT EXISTS "TCNTPmtHD_L" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FNPmhVersion" INTEGER NOT NULL DEFAULT 0,
  "FNLngID" INTEGER NOT NULL DEFAULT 0,
  "FTPmhName" TEXT COLLATE NOCASE,
  "FTPmhRmk" TEXT COLLATE NOCASE,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmhVersion", "FNLngID")
);
CREATE TABLE IF NOT EXISTS "TCNTPmtCond" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FNPmcSeqNo" INTEGER NOT NULL DEFAULT 0,
  "FTPmcCondType" TEXT COLLATE NOCASE,
  "FTPmcRefCode" TEXT COLLATE NOCASE,
  "FCPmcValue" REAL,
  "FTPmcJson" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmcSeqNo")
);
CREATE TABLE IF NOT EXISTS "TCNTPmtGrpDT" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmdGrpName" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FNPmdSeqNo" INTEGER NOT NULL DEFAULT 0,
  "FTPmdStaType" TEXT COLLATE NOCASE,
  "FTPmdRefType" TEXT COLLATE NOCASE,
  "FTPmdRefCode" TEXT COLLATE NOCASE,
  "FCPmdQty" REAL,
  "FTPmdJson" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FTPmdGrpName", "FNPmdSeqNo")
);
CREATE TABLE IF NOT EXISTS "TCNTPmtLmt" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmhDocNo" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FNPmlSeqNo" INTEGER NOT NULL DEFAULT 0,
  "FTPmlPer" TEXT COLLATE NOCASE,
  "FTPmlRefCode" TEXT COLLATE NOCASE,
  "FNPmlMaxUses" INTEGER,
  "FCPmlMaxValue" REAL,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmhDocNo", "FNPmlSeqNo")
);
CREATE TABLE IF NOT EXISTS "TCNMPmtFmt" (
  "FTAgnCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmfCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmfName" TEXT COLLATE NOCASE,
  "FTPmfCategory" TEXT COLLATE NOCASE,
  "FTPmfStaOrigin" TEXT COLLATE NOCASE NOT NULL DEFAULT '2',
  "FTPmfFmtJson" TEXT NOT NULL DEFAULT '{}',
  "FTPmfStaActive" TEXT COLLATE NOCASE NOT NULL DEFAULT '1',
  "FNPmfSortOrder" INTEGER,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTAgnCode", "FTPmfCode")
);
CREATE TABLE IF NOT EXISTS "TCNSPmtFmt" (
  "FTPmfCode" TEXT COLLATE NOCASE NOT NULL DEFAULT '',
  "FTPmfName" TEXT COLLATE NOCASE,
  "FTPmfCategory" TEXT COLLATE NOCASE,
  "FTPmfFmtJson" TEXT NOT NULL DEFAULT '{}',
  "FTPmfStaActive" TEXT COLLATE NOCASE NOT NULL DEFAULT '1',
  "FNPmfSortOrder" INTEGER,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT COLLATE NOCASE,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT COLLATE NOCASE,
  PRIMARY KEY ("FTPmfCode")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtAdjPriHDScope" (
  "FTAgnCode" TEXT NOT NULL,
  "FTBchCode" TEXT NOT NULL,
  "FTXphDocNo" TEXT NOT NULL,
  "FNXpsSeq" INTEGER NOT NULL,
  "FTXpsAgnTo" TEXT,
  "FTXpsBchTo" TEXT,
  "FTXpsMerTo" TEXT,
  "FTXpsShpTo" TEXT,
  "FTChnCode" TEXT,
  "FTXpsStaType" TEXT,
  "FTXpsRmk" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTXphDocNo", "FNXpsSeq")
);
CREATE TABLE IF NOT EXISTS "TSysProductSnapshotState" (
      "FTSnpToken" TEXT PRIMARY KEY,
      "FTAgnCode" TEXT NOT NULL,
      "FTBchCode" TEXT NOT NULL,
      "FDSnpCommittedAt" TEXT NOT NULL
    );
CREATE TABLE IF NOT EXISTS "TSysPdtSerialControlStatus" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTPdtCode" TEXT NOT NULL,
      "FTSpfCode" TEXT,
      "FTPrfStaActive" TEXT,
      "FTSerialProfileStatus" TEXT NOT NULL,
      PRIMARY KEY ("FTAgnCode", "FTPdtCode")
    );
CREATE TABLE IF NOT EXISTS "TCNMSalPolicy" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTSlpCode" TEXT NOT NULL,
      "FNSlpVersion" INTEGER NOT NULL DEFAULT 1,
      "FTSlpBusType" TEXT NOT NULL DEFAULT 'RETAIL',
      "FDSlpStartOn" TEXT NOT NULL,
      "FDSlpEndOn" TEXT,
      "FTSlpStatus" TEXT NOT NULL DEFAULT 'PUBLISHED',
      "FNSlpPriority" INTEGER NOT NULL DEFAULT 100,
      "FTSlpCombMode" TEXT NOT NULL DEFAULT 'ALL_MATCH',
      "FTSlpLogMode" TEXT NOT NULL DEFAULT 'EXCEPTION',
      "FTSlpTimeZone" TEXT NOT NULL DEFAULT 'Asia/Bangkok',
      "FTSlpStaActive" TEXT NOT NULL DEFAULT 'Y',
      "FTSlpSource" TEXT NOT NULL DEFAULT 'BACKOFFICE',
      "FTSlpCalBch" TEXT,
      "FTPahDocNo" TEXT,
      "FTSlpCalHash" TEXT,
      "FDSlpCalSync" TEXT,
      "FDSlpPublishOn" TEXT,
      "FTSlpPublishBy" TEXT,
      "FTSlpChecksum" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTSlpCode", "FNSlpVersion")
    );
CREATE TABLE IF NOT EXISTS "TCNMSalPolicy_L" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTSlpCode" TEXT NOT NULL,
      "FNSlpVersion" INTEGER NOT NULL,
      "FNLngID" INTEGER NOT NULL,
      "FTSlpName" TEXT NOT NULL,
      "FTSlpDesc" TEXT,
      "FTSlpPosMsg" TEXT,
      "FTSlpApvMsg" TEXT,
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTSlpCode", "FNSlpVersion", "FNLngID")
    );
CREATE TABLE IF NOT EXISTS "TCNMSalPolCond" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '',
      "FTSlpCode" TEXT NOT NULL,
      "FNSlpVersion" INTEGER NOT NULL,
      "FNSlcSeq" INTEGER NOT NULL,
      "FNSlcGroup" INTEGER NOT NULL DEFAULT 1,
      "FTSlcLogic" TEXT NOT NULL DEFAULT 'AND',
      "FTSlcType" TEXT NOT NULL,
      "FTSlcTarget" TEXT,
      "FTSlcRefAgn" TEXT,
      "FTSlcRefCode" TEXT,
      "FTSlcOperator" TEXT,
      "FCSlcValue1" TEXT,
      "FCSlcValue2" TEXT,
      "FTSlcValueText" TEXT,
      "FTSlcUnitCode" TEXT,
      "FTSlcAccum" TEXT,
      "FTSlcSubjType" TEXT,
      "FTSlcAction" TEXT NOT NULL DEFAULT 'BLOCK',
      "FTSlcPermission" TEXT,
      "FTSlcOffMode" TEXT NOT NULL DEFAULT 'BLOCK',
      "FTSlcInclude" TEXT NOT NULL DEFAULT 'Y',
      "FTSlcStaActive" TEXT NOT NULL DEFAULT 'Y',
      "FDLastUpdOn" TEXT,
      "FTLastUpdBy" TEXT,
      "FDCreateOn" TEXT,
      "FTCreateBy" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTSlpCode", "FNSlpVersion", "FNSlcSeq")
    );
CREATE TABLE IF NOT EXISTS TCNMRegion (
      FTAgnCode TEXT NOT NULL,
      FTRegCode TEXT NOT NULL,
      FTRegRefCode TEXT,
      FDCreateOn TEXT,
      FTCreateBy TEXT,
      FDLastUpdOn TEXT,
      FTLastUpdBy TEXT,
      PRIMARY KEY (FTAgnCode, FTRegCode)
    );
CREATE TABLE IF NOT EXISTS TCNMRegion_L (
      FTAgnCode TEXT NOT NULL,
      FTRegCode TEXT NOT NULL,
      FNLngID INTEGER NOT NULL,
      FTRegName TEXT,
      FTRegRmk TEXT,
      PRIMARY KEY (FTAgnCode, FTRegCode, FNLngID)
    );
CREATE TABLE IF NOT EXISTS TCNMSalPolPriceRule (
  FTBundleHash TEXT NOT NULL DEFAULT '',
  FTAgnCode TEXT NOT NULL,
  FTSlpCode TEXT NOT NULL,
  FNSlpVersion INTEGER NOT NULL,
  FTSlrRuleType TEXT NOT NULL,
  FTSlrStaActive TEXT NOT NULL,
  FTSlrSourceUnitMode TEXT NOT NULL,
  FTSlrAggregationMode TEXT NOT NULL,
  FTSlrSelectionMode TEXT NOT NULL,
  FTSlrPromotionBase TEXT NOT NULL,
  FTSlrMissingPriceMode TEXT NOT NULL,
  FNSlrResolverVersion INTEGER NOT NULL,
  PRIMARY KEY (FTBundleHash, FTAgnCode, FTSlpCode, FNSlpVersion, FTSlrRuleType)
);
CREATE TABLE IF NOT EXISTS TSysRepackUnitFactor (
  FTBundleHash TEXT NOT NULL DEFAULT '',
  FTAgnCode TEXT NOT NULL,
  FTPdtCode TEXT NOT NULL,
  FTPunCode TEXT NOT NULL,
  FCPdtUnitFact TEXT NOT NULL,
  FCPdtUnitFactSource TEXT NOT NULL,
  FTRepackPriceStatus TEXT NOT NULL,
  FTPdtStaActive TEXT NOT NULL,
  FDLastUpdOn TEXT,
  PRIMARY KEY (FTBundleHash, FTAgnCode, FTPdtCode, FTPunCode)
);
CREATE TABLE IF NOT EXISTS TSysRepackUnitPrice (
  FTBundleHash TEXT NOT NULL DEFAULT '',
  FTPdtAgnCode TEXT NOT NULL,
  FTPdtCode TEXT NOT NULL,
  FTPunCode TEXT NOT NULL,
  FTPplCode TEXT NOT NULL,
  FTPricingScopeId TEXT NOT NULL,
  FTVatCode TEXT NOT NULL,
  FCVatRate TEXT NOT NULL,
  FTCurrency TEXT NOT NULL,
  FTVatMode TEXT NOT NULL,
  FCPgdPrice TEXT,
  FTPriceSource TEXT NOT NULL,
  FTRepackPriceStatus TEXT NOT NULL,
  PRIMARY KEY (
    FTBundleHash, FTPdtAgnCode, FTPdtCode, FTPunCode, FTPplCode,
    FTPricingScopeId, FTVatCode, FCVatRate, FTCurrency, FTVatMode
  )
);
CREATE TABLE IF NOT EXISTS TSysRepackPriceScope (
  FTBundleHash TEXT NOT NULL DEFAULT '',
  FTAgnCode TEXT NOT NULL,
  FTBchCode TEXT NOT NULL,
  FTPghDocType TEXT NOT NULL,
  FTPghDocNo TEXT NOT NULL,
  FNPghPublishSeq INTEGER NOT NULL,
  FNPgsSrcSeq INTEGER NOT NULL,
  FTPgsAgnTo TEXT,
  FTPgsBchTo TEXT,
  FTPgsMerTo TEXT,
  FTPgsShpTo TEXT,
  FTChnCode TEXT,
  FTPgsStaType TEXT,
  FTPgsRmk TEXT,
  FDPghPublishOn TEXT,
  PRIMARY KEY (
    FTBundleHash, FTAgnCode, FTBchCode, FTPghDocType, FTPghDocNo,
    FNPghPublishSeq, FNPgsSrcSeq
  )
);
CREATE TABLE IF NOT EXISTS TSysRepackBundleHD (
  FTBundleHash TEXT NOT NULL PRIMARY KEY,
  FTContractVersion TEXT NOT NULL,
  FNResolverVersion INTEGER NOT NULL,
  FDGeneratedOn TEXT NOT NULL,
  FTBundleMode TEXT NOT NULL,
  FTPreviousBundleHash TEXT,
  FTPolicyChecksum TEXT NOT NULL,
  FTComplete TEXT NOT NULL,
  FTReasonCode TEXT NOT NULL,
  FTPageHash TEXT NOT NULL,
  FTExpectedRowCountsJson TEXT NOT NULL,
  FTObservedRowCountsJson TEXT NOT NULL,
  FTScopeKey TEXT NOT NULL,
  FTActivationStatus TEXT NOT NULL,
  FDStoredOn TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS TSysRepackManifest (
  FTBundleHash TEXT NOT NULL DEFAULT '',
  FTEntityType TEXT NOT NULL,
  FTEntityKey TEXT NOT NULL,
  FTRowHash TEXT,
  FTCanonicalPayload TEXT,
  FTPreviousRowHash TEXT,
  FTReasonCode TEXT,
  PRIMARY KEY (FTBundleHash, FTEntityType, FTEntityKey)
);
CREATE TABLE IF NOT EXISTS TSysRepackActiveBundle (
  FTScopeKey TEXT NOT NULL PRIMARY KEY,
  FTBundleHash TEXT NOT NULL,
  FDActivatedOn TEXT NOT NULL,
  FOREIGN KEY (FTBundleHash) REFERENCES TSysRepackBundleHD (FTBundleHash)
);
CREATE TABLE IF NOT EXISTS "TCNMCstPriList" (
    "FTAgnCode" TEXT NOT NULL DEFAULT '',
    "FTCstCode" TEXT NOT NULL DEFAULT '',
    "FNCplSeqNo" INTEGER NOT NULL,
    "FTCplType" TEXT,
    "FTPplCode" TEXT,
    "FTCplStaDefault" TEXT,
    "FTCplVatInOrEx" TEXT,
    "FTCplStaTypeDef" TEXT,
    "FDCplStart" TEXT,
    "FDCplEnd" TEXT,
    "FTCplDisAuto" TEXT,
    "FTPplStaUse" TEXT,
    "FTPplRmk" TEXT,
    "FDLastUpdOn" TEXT,
    "FTLastUpdBy" TEXT,
    "FDCreateOn" TEXT,
    "FTCreateBy" TEXT,
    PRIMARY KEY ("FTAgnCode", "FTCstCode", "FNCplSeqNo")
  );
CREATE TABLE IF NOT EXISTS "TCNMPdtPriList" (
        "FTAgnCode" TEXT NOT NULL DEFAULT '', "FTPplCode" TEXT NOT NULL,
        "FTPplStaUse" TEXT, "FTPplRmk" TEXT, "FDLastUpdOn" TEXT,
        "FTLastUpdBy" TEXT, "FDCreateOn" TEXT, "FTCreateBy" TEXT,
        PRIMARY KEY ("FTAgnCode", "FTPplCode"));
CREATE TABLE IF NOT EXISTS "TCNMPdtPriList_L" (
        "FTAgnCode" TEXT NOT NULL DEFAULT '', "FTPplCode" TEXT NOT NULL,
        "FNLngID" INTEGER NOT NULL, "FTPplName" TEXT, "FTPplRmk" TEXT,
        PRIMARY KEY ("FTAgnCode", "FTPplCode", "FNLngID"));
CREATE TABLE IF NOT EXISTS "TSysStandardPriceBundleState" (
      "FTContractVersion" TEXT NOT NULL PRIMARY KEY,
      "FTSyncMode" TEXT NOT NULL,
      "FDGeneratedOn" TEXT NOT NULL,
      "FTReasonCode" TEXT NOT NULL,
      "FNComplete" INTEGER NOT NULL
    );
CREATE TABLE IF NOT EXISTS "TCNTPdtPrice4PDT" (
    "FTAgnCode" TEXT NOT NULL DEFAULT '',
    "FTBchCode" TEXT NOT NULL DEFAULT '',
    "FTPghDocType" TEXT NOT NULL DEFAULT '',
    "FTPghDocNo" TEXT NOT NULL DEFAULT '',
    "FNPghSrcSeq" INTEGER NOT NULL,
    "FTPdtAgnCode" TEXT NOT NULL DEFAULT '',
    "FTPdtCode" TEXT NOT NULL,
    "FTPunCode" TEXT NOT NULL,
    "FTPplCode" TEXT NOT NULL DEFAULT '',
    "FDPghEffectiveStart" TEXT NOT NULL,
    "FDPghEffectiveStop" TEXT,
    "FTPghStaAdj" TEXT,
    "FCPgdPrice" NUMERIC,
    "FNPghPublishSeq" INTEGER NOT NULL DEFAULT 0,
    "FDPghPublishOn" TEXT,
    "FDPghDStart" TEXT,
    "FTPghTStart" TEXT,
    "FDPghDStop" TEXT,
    "FTPghTStop" TEXT,
    "FCPgdPriceRet" NUMERIC,
    "FCPgdPriceWhs" NUMERIC,
    "FCPgdPriceNet" NUMERIC,
    "FTPgdRmk" TEXT,
    "FDLastUpdOn" TEXT,
    "FTLastUpdBy" TEXT,
    "FDCreateOn" TEXT,
    "FTCreateBy" TEXT,
    PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTPghDocType", "FTPghDocNo", "FNPghSrcSeq")
  );
CREATE TABLE IF NOT EXISTS "TCNTPdtPrice4PDTStep" (
      "FTAgnCode" TEXT NOT NULL DEFAULT '', "FTBchCode" TEXT NOT NULL DEFAULT '',
      "FTPghDocType" TEXT NOT NULL DEFAULT '5', "FTPghDocNo" TEXT NOT NULL DEFAULT '',
      "FNPghSrcSeq" INTEGER NOT NULL, "FTPdtAgnCode" TEXT NOT NULL DEFAULT '',
      "FTPdtCode" TEXT NOT NULL, "FTPunCode" TEXT NOT NULL, "FTPplCode" TEXT NOT NULL DEFAULT '',
      "FDPghEffectiveStart" TEXT NOT NULL, "FDPghEffectiveStop" TEXT,
      "FCPgsMinQty" NUMERIC NOT NULL, "FCPgsPrice" NUMERIC NOT NULL,
      "FTPghStaAdj" TEXT, "FNPghPublishSeq" INTEGER NOT NULL DEFAULT 0,
      "FNPgsTierSeq" INTEGER NOT NULL DEFAULT 0, "FDLastUpdOn" TEXT,
      PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTPghDocType", "FTPghDocNo", "FNPghSrcSeq", "FNPgsTierSeq")
    );
CREATE TABLE IF NOT EXISTS "TCNTPdtPrice4PDTScope" (
    "FTAgnCode" TEXT NOT NULL DEFAULT '',
    "FTBchCode" TEXT NOT NULL DEFAULT '',
    "FTPghDocType" TEXT NOT NULL DEFAULT '',
    "FTPghDocNo" TEXT NOT NULL DEFAULT '',
    "FNPghPublishSeq" INTEGER NOT NULL DEFAULT 0,
    "FNPgsSrcSeq" INTEGER NOT NULL,
    "FTPgsAgnTo" TEXT,
    "FTPgsBchTo" TEXT,
    "FTPgsMerTo" TEXT,
    "FTPgsShpTo" TEXT,
    "FTChnCode" TEXT,
    "FTPgsStaType" TEXT,
    "FTPgsRmk" TEXT,
    "FDPghPublishOn" TEXT,
    "FDLastUpdOn" TEXT,
    "FTLastUpdBy" TEXT,
    "FDCreateOn" TEXT,
    "FTCreateBy" TEXT,
    PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTPghDocType", "FTPghDocNo", "FNPghPublishSeq", "FNPgsSrcSeq")
  );
CREATE TABLE IF NOT EXISTS "TCNTPdtAdjPriHD" (
  "FTAgnCode" TEXT NOT NULL DEFAULT '',
  "FTBchCode" TEXT NOT NULL,
  "FTXphDocNo" TEXT NOT NULL,
  "FTXphDocType" TEXT,
  "FTXphStaAdj" TEXT,
  "FDXphDocDate" TEXT,
  "FTXphDocTime" TEXT,
  "FTXphRefInt" TEXT,
  "FDXphRefIntDate" TEXT,
  "FTXphName" TEXT,
  "FTPplCode" TEXT,
  "FDXphDStart" TEXT,
  "FTXphTStart" TEXT,
  "FDXphDStop" TEXT,
  "FTXphTStop" TEXT,
  "FTXphPriType" TEXT,
  "FTXphStaDoc" TEXT,
  "FTXphStaApv" TEXT,
  "FTXphStaPrcDoc" TEXT,
  "FNXphStaDocAct" INTEGER,
  "FTXphStaDelMQ" TEXT,
  "FTUsrCode" TEXT,
  "FTXphUsrApv" TEXT,
  "FTXphRmk" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTXphDocNo")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtAdjPriDT" (
  "FTAgnCode" TEXT NOT NULL DEFAULT '',
  "FTBchCode" TEXT NOT NULL,
  "FTXphDocNo" TEXT NOT NULL,
  "FNXpdSeq" INTEGER NOT NULL,
  "FTPdtCode" TEXT,
  "FTPunCode" TEXT,
  "FCXpdPriceRet" NUMERIC,
  "FCXpdPriceWhs" NUMERIC,
  "FCXpdPriceNet" NUMERIC,
  "FTXpdShpTo" TEXT,
  "FTXpdBchTo" TEXT,
  "FDLastUpdOn" TEXT,
  "FTLastUpdBy" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTXphDocNo", "FNXpdSeq")
);
CREATE TABLE IF NOT EXISTS "TCNTPdtAdjPriHDDocRef" (
  "FTAgnCode" TEXT NOT NULL DEFAULT '',
  "FTBchCode" TEXT NOT NULL,
  "FTXphDocNo" TEXT NOT NULL,
  "FNXphSeq" INTEGER NOT NULL,
  "FTXphRefType" TEXT NOT NULL DEFAULT '',
  "FTXphRefKey" TEXT NOT NULL DEFAULT '',
  "FTXphRefDocNo" TEXT NOT NULL DEFAULT '',
  "FDXphRefDate" TEXT,
  "FTXphRefRemark" TEXT,
  "FDCreateOn" TEXT,
  "FTCreateBy" TEXT,
  PRIMARY KEY ("FTAgnCode", "FTBchCode", "FTXphDocNo", "FNXphSeq")
);
