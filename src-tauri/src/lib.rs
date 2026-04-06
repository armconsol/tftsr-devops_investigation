pub mod ai;
pub mod audit;
pub mod commands;
pub mod db;
pub mod docs;
pub mod integrations;
pub mod ollama;
pub mod pii;
pub mod state;

use sha2::{Digest, Sha256};
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Troubleshooting and RCA Assistant application");

    // Determine data directory
    let data_dir = dirs_data_dir();

    // Initialize database
    let conn = db::connection::init_db(&data_dir).expect("Failed to initialize database");
    tracing::info!("Database initialized at {:?}", data_dir);

    let app_state = AppState {
        db: Arc::new(Mutex::new(conn)),
        settings: Arc::new(Mutex::new(state::AppSettings::default())),
        app_data_dir: data_dir.clone(),
        integration_webviews: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };
    let stronghold_salt = format!(
        "tftsr-stronghold-salt-v1-{:x}",
        Sha256::digest(data_dir.to_string_lossy().as_bytes())
    );

    tauri::Builder::default()
        .plugin(
            tauri_plugin_stronghold::Builder::new(move |password| {
                let mut hasher = Sha256::new();
                hasher.update(password);
                hasher.update(stronghold_salt.as_bytes());
                hasher.finalize().to_vec()
            })
            .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .setup(|app| {
            // Restore persistent browser windows from previous session
            let app_handle = app.handle().clone();
            let state: tauri::State<AppState> = app.state();

            // Clone Arc fields for 'static lifetime
            let db = state.db.clone();
            let settings = state.settings.clone();
            let app_data_dir = state.app_data_dir.clone();
            let integration_webviews = state.integration_webviews.clone();

            tauri::async_runtime::spawn(async move {
                let app_state = AppState {
                    db,
                    settings,
                    app_data_dir,
                    integration_webviews,
                };

                if let Err(e) =
                    commands::integrations::restore_persistent_webviews(&app_handle, &app_state)
                        .await
                {
                    tracing::warn!("Failed to restore persistent webviews: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // DB / Issue CRUD
            commands::db::create_issue,
            commands::db::get_issue,
            commands::db::update_issue,
            commands::db::delete_issue,
            commands::db::list_issues,
            commands::db::search_issues,
            commands::db::get_issue_messages,
            commands::db::add_five_why,
            commands::db::update_five_why,
            commands::db::add_timeline_event,
            // Analysis / PII
            commands::analysis::upload_log_file,
            commands::analysis::detect_pii,
            commands::analysis::apply_redactions,
            // AI
            commands::ai::analyze_logs,
            commands::ai::chat_message,
            commands::ai::test_provider_connection,
            commands::ai::list_providers,
            // Docs
            commands::docs::generate_rca,
            commands::docs::generate_postmortem,
            commands::docs::update_document,
            commands::docs::export_document,
            // Integrations
            commands::integrations::test_confluence_connection,
            commands::integrations::publish_to_confluence,
            commands::integrations::test_servicenow_connection,
            commands::integrations::create_servicenow_incident,
            commands::integrations::test_azuredevops_connection,
            commands::integrations::create_azuredevops_workitem,
            commands::integrations::initiate_oauth,
            commands::integrations::handle_oauth_callback,
            commands::integrations::authenticate_with_webview,
            commands::integrations::extract_cookies_from_webview,
            commands::integrations::save_manual_token,
            commands::integrations::save_integration_config,
            commands::integrations::get_integration_config,
            commands::integrations::get_all_integration_configs,
            commands::integrations::add_ado_comment,
            // System / Settings
            commands::system::check_ollama_installed,
            commands::system::get_ollama_install_guide,
            commands::system::list_ollama_models,
            commands::system::pull_ollama_model,
            commands::system::delete_ollama_model,
            commands::system::detect_hardware,
            commands::system::recommend_models,
            commands::system::get_settings,
            commands::system::update_settings,
            commands::system::get_audit_log,
            commands::system::save_ai_provider,
            commands::system::load_ai_providers,
            commands::system::delete_ai_provider,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Troubleshooting and RCA Assistant application");
}

/// Determine the application data directory.
fn dirs_data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("TFTSR_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }

    // Use platform-appropriate data directory
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return std::path::PathBuf::from(xdg).join("trcaa");
        }
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("trcaa");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("trcaa");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return std::path::PathBuf::from(appdata).join("trcaa");
        }
    }

    // Fallback
    std::path::PathBuf::from("./trcaa-data")
}
