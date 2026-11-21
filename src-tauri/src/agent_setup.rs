use axum::{http::StatusCode,response::{IntoResponse,Response},Json};
use serde_json::json;

/// Register for the current interactive Windows user. Do not start printing as
/// a side effect of registration; the scheduled task begins on next sign-in.
pub async fn register() -> Response {
    if cfg!(debug_assertions) {
        return (StatusCode::BAD_REQUEST,Json(json!({"error":"การตั้งให้เริ่มพร้อม Windows ใช้ได้จากชุดติดตั้ง AdaBarprint กรุณาติดตั้งรุ่นที่รวมบริการพิมพ์ก่อน"}))).into_response();
    }
    #[cfg(target_os="windows")]
    {
        let result:Result<(),String>=async {
            let exe=std::env::current_exe().map_err(|e|e.to_string())?;
            let folder=exe.parent().ok_or("ไม่พบโฟลเดอร์ติดตั้ง")?.join("agent");
            let script=folder.join("Register-AutoPrintAgent.ps1");
            let agent=folder.join("autoprint_agent.exe");
            if !script.is_file() || !agent.is_file() {return Err("ชุดติดตั้งไม่มีบริการพิมพ์เบื้องหลัง กรุณาติดตั้งชุดที่รวม Agent".into());}
            let database=crate::storage::print_db_path();
            let directory=database.parent().ok_or("ไม่พบโฟลเดอร์ฐานข้อมูลการพิมพ์")?;
            let output=tokio::process::Command::new("powershell.exe")
                .creation_flags(0x08000000)
                .args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"])
                .arg(script).arg("-AgentPath").arg(agent).arg("-DatabaseDirectory").arg(directory)
                .output().await.map_err(|e|e.to_string())?;
            if !output.status.success() {return Err("Windows ไม่อนุญาตให้ตั้งงานเริ่มอัตโนมัติ กรุณาให้ผู้ดูแลตรวจสิทธิ์ Task Scheduler ของบัญชีนี้".into());}
            Ok(())
        }.await;
        match result {
            Ok(())=>Json(json!({"ok":true,"message":"ตั้งค่าแล้ว บริการจะเริ่มเมื่อบัญชี Windows นี้เข้าสู่ระบบครั้งถัดไป"})).into_response(),
            Err(error)=>(StatusCode::BAD_REQUEST,Json(json!({"error":error}))).into_response(),
        }
    }
    #[cfg(not(target_os="windows"))]
    (StatusCode::BAD_REQUEST,Json(json!({"error":"รองรับการเริ่มอัตโนมัติบน Windows เท่านั้น"}))).into_response()
}
