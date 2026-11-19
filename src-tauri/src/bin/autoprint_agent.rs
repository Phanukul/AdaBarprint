#[tokio::main]
async fn main() {
    adabarprint_tauri_lib::db::load_env();
    let pool = adabarprint_tauri_lib::db::connect_db()
        .await
        .expect("Failed to connect db/AdaAccD.db in 24/7 autoprint_agent");

    adabarprint_tauri_lib::db::ensure_label_schema(&pool)
        .await
        .expect("Failed to ensure AdaAccD.db schema in autoprint_agent");

    let _ = adabarprint_tauri_lib::db::seed_defaults(&pool).await;

    println!("[AdaBarprint 24/7 Auto-Print Agent] Started. Monitoring TCNTLabelSchedule (WAL Mode)...");

    adabarprint_tauri_lib::scheduler_worker::spawn_scheduler_loop(pool.clone());

    let _ = tokio::signal::ctrl_c().await;
    println!("\n[AdaBarprint 24/7 Auto-Print Agent] Shutting down gracefully...");
    pool.close().await;
}
