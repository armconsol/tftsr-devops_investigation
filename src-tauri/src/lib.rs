pub mod ai;
pub mod audit;
pub mod commands;
pub mod db;
pub mod docs;
pub mod integrations;
pub mod kube;
pub mod mcp;
pub mod ollama;
pub mod pii;
pub mod shell;
pub mod state;

use sha2::{Digest, Sha256};
use state::AppState;
use std::sync::{Arc, Mutex};

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
        mcp_connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        pending_approvals: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        clusters: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        port_forwards: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        refresh_registry: Arc::new(tokio::sync::Mutex::new(crate::kube::RefreshRegistry::new())),
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
            let handle = app.handle().clone();

            // Initialize MCP servers
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::mcp::discovery::init_all_servers(&handle).await {
                    tracing::warn!("MCP startup discovery error: {e}");
                }
            });

            // Auto-detect kubeconfig
            // Note: Kubeconfig auto-detection is implemented in shell::kubeconfig::auto_detect_kubeconfig
            // but not called at startup because it requires database access which may not be initialized yet.
            // Users can manually upload kubeconfig files via the frontend UI.

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
            commands::db::get_timeline_events,
            // Analysis / PII
            commands::analysis::upload_log_file,
            commands::analysis::upload_log_file_by_content,
            commands::analysis::detect_pii,
            commands::analysis::scan_text_for_pii,
            commands::analysis::apply_redactions,
            commands::analysis::get_log_file_content,
            commands::analysis::list_all_log_files,
            commands::image::upload_image_attachment,
            commands::image::upload_image_attachment_by_content,
            commands::image::list_image_attachments,
            commands::image::delete_image_attachment,
            commands::image::upload_paste_image,
            commands::image::get_image_attachment_data,
            commands::image::list_all_image_attachments,
            commands::image::upload_file_to_datastore,
            commands::image::upload_file_to_datastore_any,
            // AI
            commands::ai::analyze_logs,
            commands::ai::chat_message,
            commands::ai::test_provider_connection,
            commands::ai::detect_tool_calling_support,
            commands::ai::list_providers,
            commands::system::save_ai_provider,
            commands::system::load_ai_providers,
            commands::system::delete_ai_provider,
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
            commands::system::get_app_version,
            commands::system::set_sudo_password,
            commands::system::get_sudo_config_status,
            commands::system::test_sudo_password,
            commands::system::clear_sudo_password,
            // MCP Servers
            mcp::commands::list_mcp_servers,
            mcp::commands::create_mcp_server,
            mcp::commands::update_mcp_server,
            mcp::commands::delete_mcp_server,
            mcp::commands::toggle_mcp_server,
            mcp::commands::discover_mcp_server,
            mcp::commands::get_mcp_server_status,
            mcp::commands::initiate_mcp_oauth,
            // Shell Execution
            commands::shell::upload_kubeconfig,
            commands::shell::list_kubeconfigs,
            commands::shell::activate_kubeconfig,
            commands::shell::delete_kubeconfig,
            commands::shell::respond_to_shell_approval,
            commands::shell::list_command_executions,
            commands::shell::check_kubectl_installed,
            // Kubernetes Management
            commands::kube::add_cluster,
            commands::kube::remove_cluster,
            commands::kube::list_clusters,
            commands::kube::start_port_forward,
            commands::kube::stop_port_forward,
            commands::kube::list_port_forwards,
            commands::kube::delete_port_forward,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Troubleshooting and RCA Assistant application");
}

/// Determine the application data directory.
fn dirs_data_dir() -> std::path::PathBuf {
    // Support both TRCAA_DATA_DIR (new) and TFTSR_DATA_DIR (legacy) for backwards compatibility
    if let Ok(dir) = std::env::var("TRCAA_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("TFTSR_DATA_DIR") {
        tracing::warn!("TFTSR_DATA_DIR is deprecated, use TRCAA_DATA_DIR instead");
        return std::path::PathBuf::from(dir);
    }

    // Use platform-appropriate data directory
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return std::path::PathBuf::from(xdg).join("tftsr");
        }
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("tftsr");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("tftsr");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return std::path::PathBuf::from(appdata).join("tftsr");
        }
    }

    // Fallback
    std::path::PathBuf::from("./tftsr-data")
}
