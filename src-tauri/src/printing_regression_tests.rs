use crate::{storage,standalone,db,auth,print_center,delta_engine,routes,models};
use axum::{extract::State,Json};
use serde_json::json;
use sqlx::{Connection,SqliteConnection,SqlitePool};

async fn fixture()->(SqlitePool,SqliteConnection){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tmp/printing-regression").join(uuid::Uuid::new_v4().to_string());
 std::fs::create_dir_all(&root).unwrap();
 let master=root.join("AdaAccD.db");
 standalone::create_master(&master).await.unwrap();
 let pool=storage::connect_path(&root.join("AdaBarprint.db")).await.unwrap();
 db::ensure_label_schema(&pool).await.unwrap();
 db::seed_defaults(&pool).await.unwrap();
 print_center::ensure_schema(&pool).await.unwrap();
 auth::ensure_auth_schema(&pool).await.unwrap();
 sqlx::query("INSERT INTO ABStorage VALUES(1,?,'B1',1440)").bind(master.to_str().unwrap()).execute(&pool).await.unwrap();
 sqlx::query("INSERT INTO ABSource(id,mode,last_success) VALUES(1,'STANDALONE','2026-11-23 00:00:00')").execute(&pool).await.unwrap();
 let conn=SqliteConnection::connect_with(&sqlx::sqlite::SqliteConnectOptions::new().filename(master)).await.unwrap();
 (pool,conn)
}
fn schedule(event:&str)->models::TCNTLabelSchedule{
 serde_json::from_value(json!({"FTLscCode":"AUDIT","FTLscName":"Audit","FTLscStaUse":"1","FTLscRunTime":"00:00","FNLscPreSyncMin":0,"FNLscJitterSec":0,"FTLscDaysMask":"1111111","FTLscTriggerRules":event,"FTLscBchCode":"B1","FTLscPplCode":"","FTLscLbsCode":"ADA-100x38","FNLscQtyMode":1,"FTLscSyncMode":"LOCAL_ONLY","FNLscSyncRetry":0,"FNLscRetryWaitSec":0,"FTLscOnSyncFail":"ABORT","FTLscOnPrinterBack":"WAIT_CONFIRM","FNLscMisfireMin":0,"FTLscStaCoverSheet":"0"})).unwrap()
}

/// Exercise manual preparation against a read-only installed master, isolated from the live print queue.
#[tokio::test]
#[ignore]
async fn installed_master_checks_all_active_barcodes() {
    let master = std::env::var("ADABARPRINT_VERIFY_MASTER").unwrap();
    let branch = std::env::var("ADABARPRINT_VERIFY_BRANCH").unwrap();
    let (pool, conn) = fixture().await;
    sqlx::query("UPDATE ABStorage SET path=?, branch=? WHERE id=1")
        .bind(master).bind(&branch).execute(&pool).await.unwrap();
    let products: Vec<(String,String)> = sqlx::query_as(
        "SELECT DISTINCT FTPdtCode, FTBarCode FROM master.TCNMPdtBar
         WHERE COALESCE(FTBarStaUse,'1')='1' AND TRIM(FTBarCode)<>''
         ORDER BY FTPdtCode,FTBarCode"
    ).fetch_all(&pool).await.unwrap();
    let total = products.len();
    assert!(total > 0);
    let mut failures = std::collections::BTreeMap::new();
    let mut no_price = 0;
    for (code, barcode) in products {
        let request = json!({"barcode":barcode,"pdtCode":code,"qty":1,"bchCode":branch,"priType":"1","verify":true});
        let response = routes::post_build_line(State(routes::AppState { pool:pool.clone() }),
            Json(serde_json::from_value(request).unwrap())).await;
        if !response.status().is_success() {
            let body = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
            let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let reason = error["error"].as_str().unwrap();
            if reason.starts_with("ไม่พบราคาขายที่ใช้ได้") {
                no_price += 1;
            } else {
                *failures.entry(reason.to_owned()).or_insert(0) += 1;
            }
        }
    }
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM ABPrintJob").fetch_one(&pool).await.unwrap(), 0);
    let blocked: usize = failures.values().copied().sum();
    println!("Total {total}, prepared {}, missing price {no_price}, other blocks {failures:?}", total-no_price-blocked);
    assert!(total > no_price+blocked, "No printable labels");
    conn.close().await.unwrap();
    pool.close().await;
}
async fn seed_product(c:&mut SqliteConnection){
 sqlx::raw_sql("INSERT INTO TCNMBranch(FTBchCode,FTPplCode,FTWahCode) VALUES('B1','','');
 INSERT INTO TCNMPdt(FTPdtCode,FDCreateOn) VALUES('P1','2026-01-01');
 INSERT INTO TCNMPdt_L(FTPdtCode,FNLngID,FTPdtName) VALUES('P1',1,'Audit product');
 INSERT INTO TCNMPdtBar(FTPdtCode,FTBarCode,FTPunCode,FTBarStaUse) VALUES('P1','8850000000016','PCS','1');
 INSERT INTO TCNMPdtPackSize(FTPdtCode,FTPunCode,FCPdtPriceRET) VALUES('P1','PCS',100);").execute(&mut *c).await.unwrap();
}
async fn price(c:&mut SqliteConnection,seq:i64,date:&str,unit:&str,group:&str,amount:f64){
 sqlx::query("INSERT INTO TCNTPdtPrice4PDT(FNPghSrcSeq,FTPdtCode,FTPunCode,FDPghEffectiveStart,FDPghDStart,FTPghDocType,FTPplCode,FCPgdPrice,FCPgdPriceRet) VALUES(?,'P1',?,?,?,'1',?,?,?)")
 .bind(seq).bind(unit).bind(date).bind(date).bind(group).bind(amount).bind(amount).execute(c).await.unwrap();
}

#[tokio::test]
async fn price_change_respects_unit_and_price_group_and_reports_schema_errors() {
 let (pool,mut c)=fixture().await; seed_product(&mut c).await;
 price(&mut c,1,"2026-01-01","PCS","",100.0).await;
 price(&mut c,2,"2026-11-24","BOX","MEMBER",900.0).await;
 assert!(delta_engine::compute_profile_candidates_exact(&pool,&schedule("PRICE_CHG"),"2026-11-24").await.unwrap().items.is_empty());
 price(&mut c,3,"2026-11-24","PCS","",110.0).await;
 let result=delta_engine::compute_profile_candidates_exact(&pool,&schedule("PRICE_CHG"),"2026-11-24").await.unwrap();
 assert_eq!(result.items.len(),1); assert_eq!(result.items[0].sale_price,110.0);
 price(&mut c,4,"2026-11-25","PCS","",110.0).await;
 assert!(delta_engine::compute_profile_candidates_exact(&pool,&schedule("PRICE_CHG"),"2026-11-25").await.unwrap().items.is_empty());
 sqlx::query("DROP TABLE TCNTPdtAdjPriDT").execute(&mut c).await.unwrap();
 assert!(delta_engine::compute_profile_candidates_exact(&pool,&schedule("PRICE_CHG"),"2026-11-24").await.is_err());
 c.close().await.unwrap(); pool.close().await;
}

async fn canonical(c:&mut SqliteConnection) {
 sqlx::query("UPDATE TCNMPdt SET FTTcgCode='G1' WHERE FTPdtCode='P1'").execute(&mut *c).await.unwrap();
 let rule=json!({"groups":{"A":{"include":[{"type":"GRP","code":"G1"}]}},"buy":[{"group":"A","basis":"qty","min":1}],"get":[{"type":"NP","target":"A","value":79}]});
 sqlx::query("INSERT INTO TCNTPmtHD(FTAgnCode,FTPmhDocNo,FNPmhVersion,FTPmhName,FDPmhDStart,FDPmhDStop,FTPmhStaActive,FTPmhStaDoc,FTPmhStaPrcDoc,FTPmhRuleJson) VALUES('','PROMO-GROUP',1,'Group promo','2026-11-23','2026-11-24','1','1','1',?)")
 .bind(rule.to_string()).execute(&mut *c).await.unwrap();
}

#[tokio::test]
async fn four_manual_events_resolve_group_promotion_and_revert_to_base() {
 let (pool,mut c)=fixture().await; seed_product(&mut c).await;
 price(&mut c,1,"2026-11-23","PCS","",100.0).await; canonical(&mut c).await;
 sqlx::query("UPDATE TCNMBranch SET FTAgnCode='LOCAL' WHERE FTBchCode='B1'").execute(&mut c).await.unwrap();
 sqlx::query("INSERT INTO TCNTPmtHD(FTAgnCode,FTPmhDocNo,FNPmhVersion,FTPmhName,FDPmhDStart,FDPmhDStop,FTPmhStaActive,FTPmhStaDoc,FTPmhStaPrcDoc,FTPmhRuleJson) SELECT 'OTHER','FOREIGN',FNPmhVersion,FTPmhName,FDPmhDStart,FDPmhDStop,FTPmhStaActive,FTPmhStaDoc,FTPmhStaPrcDoc,FTPmhRuleJson FROM TCNTPmtHD WHERE FTPmhDocNo='PROMO-GROUP'").execute(&mut c).await.unwrap();
 sqlx::query("UPDATE TCNMPdt SET FDCreateOn='2026-11-23' WHERE FTPdtCode='P1'").execute(&mut c).await.unwrap();
 for (event,day,expected) in [("PRICE_CHG","2026-11-23",100.0),("NEW_PDT","2026-11-23",79.0),("PROMO_START","2026-11-23",79.0),("PROMO_END_REVERT","2026-11-24",100.0)] {
  let response=print_center::manual_candidates(State(routes::AppState{pool:pool.clone()}),Json(print_center::ManualCandidates{source:event.into(),date:day.into(),size:"SH-100x38".into(),quantity:1})).await;
  let status=response.status();let bytes=axum::body::to_bytes(response.into_body(),1048576).await.unwrap();
  assert!(status.is_success(),"{event}: {}",String::from_utf8_lossy(&bytes));
  let result:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
  assert_eq!(result["lines"].as_array().unwrap().len(),1,"{event}: {result}");
  assert_eq!(result["lines"][0]["setPrice"],expected,"{event}");
 }
 // Exact exclusions must also suppress the expiry event, not just active price.
 sqlx::query("UPDATE TCNTPmtHD SET FTPmhRuleJson=json_set(FTPmhRuleJson,'$.groups.A.exclude',json('[{\"type\":\"BAR\",\"code\":\"8850000000016\"}]'))").execute(&mut c).await.unwrap();
 assert!(crate::promo_v6::canonical_event_barcodes(&pool,"2026-11-24","2026-11-24",true).await.unwrap().is_empty());
 c.close().await.unwrap(); pool.close().await;
}

#[tokio::test]
async fn four_scheduler_profiles_enqueue_snapshots_without_sending() {
 let (pool,mut c)=fixture().await; seed_product(&mut c).await;
 let today=chrono::Local::now().format("%Y-%m-%d").to_string();
 price(&mut c,1,&today,"PCS","",100.0).await; canonical(&mut c).await;
 sqlx::query("UPDATE TCNMPdt SET FDCreateOn=?").bind(&today).execute(&mut c).await.unwrap();
 sqlx::query("UPDATE TCNTPmtHD SET FDPmhDStart=?,FDPmhDStop=?").bind(&today).bind(&today).execute(&mut c).await.unwrap();
 let layout=json!({"elements":[{"type":"price","field":"price.value","x":1,"y":1,"width":30,"height":12,"fontSize":18}]});
 sqlx::query("INSERT OR REPLACE INTO TCNMLabelTmpl(FTLtmCode,FTLbsCode,FTLtmName,FTLtmPriType,FTLtmStaUse,FTLtmLayout) VALUES('REGRESSION','SH-100x38','Regression','1','1',?)")
  .bind(layout.to_string()).execute(&pool).await.unwrap();
 for event in ["PRICE_CHG","NEW_PDT","PROMO_START","PROMO_END_REVERT"] {
  let rule=json!({"id":event,"code":event,"name":event,"enabled":true,"branch":"B1","source":event,"time":"","days":"","size":"SH-100x38","template":"REGRESSION","printer":"SIMULATION ONLY","tray":"","protocol":"ZPL","ip":"127.0.0.1","port":1,"dpi":203,"quantity":1,"catchUp":false,"offsetDays":0});
  sqlx::query("INSERT INTO ABPrintRule(id,config) VALUES(?,?)").bind(event).bind(rule.to_string()).execute(&pool).await.unwrap();
  let result=print_center::enqueue_profile(&pool,event).await.unwrap();
  assert_eq!(result["status"],"QUEUED","{event}: {result}");
  assert_eq!(result["itemCount"],1,"{event}: {result}");
  assert_eq!(result["printedUnits"],0);
  assert!(print_center::enqueue_profile(&pool,event).await.unwrap_err().contains("มีอยู่ในคิวแล้ว"));
 }
 let states:Vec<String>=sqlx::query_scalar("SELECT status FROM ABPrintUnit").fetch_all(&pool).await.unwrap();
 assert_eq!(states,vec!["READY";4]);
 // A fifth event is rejected, including configurations left by older builds.
 sqlx::query("INSERT INTO ABPrintRule(id,config) SELECT 'old',json_set(config,'$.id','old','$.code','OLD','$.source','ALL_PRODUCTS') FROM ABPrintRule LIMIT 1").execute(&pool).await.unwrap();
 assert!(print_center::enqueue_profile(&pool,"OLD").await.unwrap_err().contains("ประเภทข้อมูล"));
 c.close().await.unwrap(); pool.close().await;
}

#[tokio::test]
async fn import_master_twice_keeps_product_identity_and_resolves_prices() {
 let (pool,mut conn)=fixture().await;
 for price in [100,120] {
    let body=serde_json::from_value(json!({"branch":"B1","items":[{"pdtCode":"P1","pdtName":"Coffee","barCode":"00123","basePrice":price,"promoPrice":79,"effectiveStart":"2026-11-23","effectiveStop":"2026-11-30"}]})).unwrap();
    let response=routes::post_import_master(State(routes::AppState{pool:pool.clone()}),Json(body)).await;
    let status=response.status();
    let bytes=axum::body::to_bytes(response.into_body(),10240).await.unwrap();
    assert!(status.is_success(),"{}",String::from_utf8_lossy(&bytes));
    if price==100 {sqlx::query("UPDATE TCNMPdt SET FTPbnCode='KEEP',FDCreateOn='2025-01-01'").execute(&mut conn).await.unwrap();}
 }
 assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNMPdt WHERE FTPbnCode='KEEP' AND FDCreateOn='2025-01-01'").fetch_one(&mut conn).await.unwrap(),1);
 assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM TCNTPdtPrice4PDT").fetch_one(&mut conn).await.unwrap(),1);
 let result=crate::promo_v6::resolve_effective_price(&pool,"P1","00123",None,"2026-11-24",true).await.unwrap();
 assert_eq!((result.normal_price,result.sale_price),(120.0,79.0));
}

#[tokio::test]
async fn branch_only_sync_accepts_one_agency_and_rejects_mixed_prices(){
 let (pool,c)=fixture().await;
 let path:String=sqlx::query_scalar("SELECT path FROM ABStorage WHERE id=1").fetch_one(&pool).await.unwrap();
 let tasks:serde_json::Value=serde_json::from_str(include_str!("../resources/master-tasks.json")).unwrap();
 let mut bundles=vec![];
 for task in tasks.as_array().unwrap(){
   let mut data=serde_json::Map::new();
   for (key,table) in task["mapping"].as_object().unwrap(){
     let rows=if table=="TCNMBranch" {json!([{"FTBchCode":"B1","FTPplCode":"","FTWahCode":""}])}
       else if table=="TCNTPdtPrice4PDT" {json!([{"FTAgnCode":"A1","FTPdtCode":"P1","FTPunCode":"PCS","FNPghSrcSeq":1,"FDPghEffectiveStart":"2026-11-23"}])}
       else {json!([])};
     data.insert(key.clone(),rows);
   }
   bundles.push((task["table"].as_str().unwrap().to_string(),serde_json::Value::Object(data)));
 }
 let cfg=standalone::SyncConfig{base_url:"https://example.invalid".into(),header_name:"X-API-Key".into(),branch:"B1".into(),api_key:"audit".into(),..Default::default()};
 assert!(standalone::validate_config(&cfg).is_ok());
 let result=standalone::apply_snapshots(std::path::Path::new(&path),&cfg,&bundles).await;
 println!("AUDIT optional empty agency config accepted but valid A1 price snapshot={result:?}");
 result.unwrap();
 let agency:String=sqlx::query_scalar("SELECT agency FROM ABMasterIdentity WHERE id=1").fetch_one(&pool).await.unwrap();
 assert_eq!(agency,"A1");
 for (_,data) in &mut bundles {
    if let Some(rows)=data.get_mut("raPdtPrice4PDT").and_then(|v|v.as_array_mut()) {
        if let Some(first)=rows.first().cloned() { let mut other=first; other["FTAgnCode"]=json!("A2"); rows.push(other); }
    }
 }
 assert!(standalone::apply_snapshots(std::path::Path::new(&path),&cfg,&bundles).await.unwrap_err().contains("Agency"));
 c.close().await.unwrap();pool.close().await;
}
