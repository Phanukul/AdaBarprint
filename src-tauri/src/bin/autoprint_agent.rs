#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    let args:Vec<String>=std::env::args().skip(1).collect();
    if !args.is_empty() {
        if args.len()!=2 || args[0]!="--database-directory" {eprintln!("Invalid Agent arguments");return;}
        let directory=std::path::Path::new(&args[1]);
        if !directory.is_absolute() || !directory.is_dir() {eprintln!("Invalid database directory");return;}
        std::env::set_var("ADABARPRINT_DB_DIR",directory);
    }
    // Same initialization and local settings as the desktop application.
    let (_router, pool) = adabarprint_tauri_lib::init_db_and_router()
        .await.expect("Failed to initialize AdaBarprint storage");
    if std::env::var("AUTOPRINT_DISABLE_EMBEDDED_SCHEDULER").as_deref() == Ok("1") {
        adabarprint_tauri_lib::scheduler_worker::spawn_scheduler_loop(pool.clone());
    }
    if tokio::signal::ctrl_c().await.is_err() {
        // A Windows background task may have no console signal source.
        std::future::pending::<()>().await;
    }
    println!("\n[AdaBarprint 24/7 Auto-Print Agent] Shutting down gracefully...");
    pool.close().await;
}
