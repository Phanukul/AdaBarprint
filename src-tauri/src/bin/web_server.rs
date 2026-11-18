use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let (router, pool) = adabarprint_tauri_lib::init_db_and_router()
        .await
        .expect("Failed to connect and initialize db/AdaAccD.db");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind 127.0.0.1:3000 (port already in use)");

    println!(
        "[AdaBarprint Web Dev] API listening on http://{} (open http://localhost:5173/AdaBarprint)",
        addr
    );

    let close_pool = pool.clone();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n[AdaBarprint Web Dev] Shutting down...");
            close_pool.close().await;
        })
        .await
        .expect("Axum server error");
}
