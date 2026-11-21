pub mod standalone;
pub mod agent_setup;
pub mod auth;
pub mod barcode;
pub mod db;
pub mod delta_engine;
pub mod label_renderer;
pub mod models;
pub mod print_drivers;
pub mod promo_v6;
pub mod promo_rule;
pub mod routes;
pub mod scheduler_worker;
pub mod storage;
pub mod templates;
pub mod print_center;

/// Enables Auto-Print routes and the embedded scheduler for testing.
/// Use AUTOPRINT_DISABLE_EMBEDDED_SCHEDULER=1 when running the standalone agent.
pub const ENABLE_AUTO_PRINT: bool = true;

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use routes::*;

#[derive(RustEmbed)]
#[folder = "../dist"]
struct Assets;

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    // Unknown API routes must not return the SPA HTML document as a successful response.
    if path == "api" || path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
            "error": "ไม่พบบริการที่เรียกใช้งาน กรุณาตรวจสอบรุ่นของบริการพิมพ์"
        }))).into_response();
    }
    let target = if path.is_empty() { "index.html" } else { path };

    if let Some(content) = Assets::get(target) {
        let mime = mime_guess::from_path(target).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    if let Some(index) = Assets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(index.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    (
        StatusCode::NOT_FOUND,
        "Frontend assets not embedded. Run `npm run build` first.",
    )
        .into_response()
}

pub async fn init_db_and_router() -> Result<(Router, SqlitePool), sqlx::Error> {
    db::load_env();
    let pool = db::connect_db().await?;
    db::ensure_label_schema(&pool).await?;
    print_center::ensure_schema(&pool).await?;
    auth::ensure_auth_schema(&pool).await?;
    sqlx::query(&format!("PRAGMA user_version={}",storage::PRINT_SCHEMA_REVISION)).execute(&pool).await?;
    let _ = db::seed_defaults(&pool).await;
    let _ = db::ensure_ufm_label_sizes(&pool).await;
    let _ = db::ensure_ada_label_sizes(&pool).await;
    if ENABLE_AUTO_PRINT
        && std::env::var("AUTOPRINT_DISABLE_EMBEDDED_SCHEDULER").as_deref() != Ok("1")
    {
        scheduler_worker::spawn_scheduler_loop(pool.clone());
    }
    if std::env::var("ADABARPRINT_DISABLE_SYNC_WORKER").as_deref()!=Ok("1"){standalone::spawn(pool.clone());}
    Ok((build_router(pool.clone()), pool))
}

pub fn build_router(pool: SqlitePool) -> Router {
    let state = AppState { pool: pool.clone() };

    let mut router = Router::new()
        .route("/api/storage", get(storage::get_status).post(storage::save_config))
        .route("/api/storage/sync", post(standalone::trigger))
        .route("/api/storage/check", post(storage::check_config))
        .route("/api/agent/register", post(agent_setup::register))
        .route("/api/print-center", get(print_center::get_center))
        .route("/api/print-center/rule", post(print_center::save_rule))
        .route("/api/print-center/rules", post(print_center::save_rules))
        .route("/api/print-center/preview", post(print_center::preview_rule))
        .route("/api/print-center/manual-candidates", post(print_center::manual_candidates))
        .route("/api/print-center/enqueue", post(print_center::enqueue_rule))
        .route("/api/print-center/manual", post(print_center::enqueue_manual))
        .route("/api/print-center/action", post(print_center::queue_action))
        .route("/api/print-center/sync", post(print_center::save_sync))
        .route("/api/print-center/sync-now", post(print_center::request_sync))
        .route("/api/auth/login-options", get(auth::get_login_options))
        .route("/api/auth/users", get(auth::get_auth_users))
        .route("/api/auth/login", post(auth::post_auth_login))
        .route("/api/auth/me", get(auth::get_auth_me))
        .route("/api/auth/lock", post(auth::post_auth_lock))
        .route("/api/auth/unlock", post(auth::post_auth_unlock))
        .route("/api/auth/logout", post(auth::post_auth_logout))
        .route("/api/seed", get(get_seed).post(post_seed))
        .route("/api/branches", get(get_branches))
        .route("/api/filters/master", get(get_filter_master))
        // Keep read-only run history endpoint available alongside /api/history
        .route("/api/schedules/runs", get(get_schedule_runs))
        .route("/api/paper-sizes", get(get_paper_sizes))
        .route("/api/paper-sizes/save", post(post_save_paper_size))
        .route("/api/paper-sizes/delete", post(post_delete_paper_size))
        .route("/api/preprints", get(get_preprints))
        .route("/api/preprints/save", post(post_save_preprint))
        .route("/api/preprints/delete", post(post_delete_preprint))
        .route("/api/label-fields", get(get_label_fields))
        .route("/api/lan-printers", get(get_lan_printers))
        .route("/api/lan-printers/save", post(post_save_lan_printer))
        .route("/api/lan-printers/delete", post(post_delete_lan_printer))
        .route("/api/label-sizes", get(get_label_sizes))
        .route("/api/label-sizes/save", post(post_save_label_size))
        .route("/api/label-sizes/toggle", post(post_toggle_label_size))
        .route("/api/label-sizes/delete", post(post_delete_custom_format))
        .route("/api/config/all", get(get_config_all))
        .route("/api/config/options", post(post_save_config_options))
        .route(
            "/api/label-printers",
            get(get_label_printers).post(post_label_printers),
        )
        .route("/api/templates", get(get_templates_route))
        .route("/api/templates/save", post(post_save_template))
        .route("/api/templates/save-as", post(post_save_as_template))
        .route("/api/templates/delete-custom", post(post_delete_custom_format))
        .route("/api/templates/reset", post(post_reset_template))
        .route("/api/printers", get(get_printers).post(post_printers))
        .route("/api/printers/windows", get(get_windows_printers))
        .route("/api/promotions", get(get_promotions))
        .route("/api/products", get(get_products))
        .route("/api/image-proxy", get(get_image_proxy))
        .route("/api/history", get(get_history))
        .route("/api/history/:jobId", get(get_history_job))
        .route("/api/import", post(post_import))
        .route("/api/print/build-line", post(post_build_line))
        .route("/api/print/create-job", post(post_create_job))
        .route("/api/print/send", post(print_center::enqueue_lan))
        .route("/api/print/windows-direct", post(print_center::enqueue_windows))
        .route("/api/print/check", post(post_print_check))
        .route("/api/print/ezpl", get(get_print_ezpl).post(post_print_ezpl))
        .route("/api/godex/print", post(post_godex_print))
        .route("/api/godex/status", post(post_godex_status));

    if ENABLE_AUTO_PRINT {
        router = router
            .route("/api/schedules", get(get_schedules))
            .route("/api/schedules/save", post(post_save_schedule))
            .route("/api/schedules/:code/trigger", post(print_center::legacy_disabled))
            .route("/api/schedules/runs/:runId/reprint-failed", post(print_center::legacy_disabled))
            .route("/api/schedules/runs/:runId/load-to-manual", post(post_load_failed_to_manual))
            .route("/api/sync/status", get(get_sync_status))
            .route("/api/sync/trigger", post(print_center::request_sync))
            .route("/api/config/admin-verify", post(post_verify_admin_pin));
    }

    router
        .layer(axum::extract::DefaultBodyLimit::max(128 * 1024 * 1024))
        .fallback(static_handler)
        .layer(axum::middleware::from_fn_with_state(pool, auth::require_api_session))
        .with_state(state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (router, db_pool) = tauri::async_runtime::block_on(async { init_db_and_router().await })
        .expect("Failed to initialize AdaBarprint storage");

    let mut router_opt = Some(router);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![storage::choose_master_database])
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.unminimize();
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .setup(move |app| {
            let router = router_opt.take().expect("Router already initialized");

            let listener = tauri::async_runtime::block_on(async {
                if cfg!(debug_assertions) {
                    match tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000)))
                        .await
                    {
                        Ok(l) => l,
                        Err(_) => {
                            tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
                                .await
                                .expect("Failed to bind local backend port")
                        }
                    }
                } else {
                    tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
                        .await
                        .expect("Failed to bind local backend port")
                }
            });

            let port = listener
                .local_addr()
                .expect("Missing local address")
                .port();
            println!(
                "[AdaBarprint Tauri] Internal API listening on http://127.0.0.1:{}",
                port
            );

            tauri::async_runtime::spawn(async move {
                if let Err(e) = axum::serve(listener, router).await {
                    eprintln!("[AdaBarprint Tauri] Axum server error: {}", e);
                }
            });

            let initial_url: tauri::Url = if cfg!(debug_assertions) {
                "http://localhost:5173/AdaBarprint"
                    .parse()
                    .expect("Invalid devUrl")
            } else {
                format!("http://127.0.0.1:{}/AdaBarprint", port)
                    .parse()
                    .expect("Invalid local URL")
            };

            let win = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(initial_url))
                .title("AdaPos 6.0 PrintBar — ระบบพิมพ์ป้ายราคาและบาร์โค้ด")
                .inner_size(1366.0, 860.0)
                .min_inner_size(1024.0, 680.0)
                .maximized(true)
                .build()?;

            let _ = win.set_focus();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building AdaBarprint Tauri application");

    let exit_pool = db_pool.clone();
    app.run(move |_app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let pool = exit_pool.clone();
            tauri::async_runtime::block_on(async move {
                pool.close().await;
            });
        }
    });
}
