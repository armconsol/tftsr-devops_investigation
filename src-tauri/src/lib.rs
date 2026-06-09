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
        watchers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        log_streams: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
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
            commands::db::load_clusters,
            commands::db::load_port_forwards,
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
            commands::shell::get_classifier_rules,
            // Kubernetes Management
            commands::kube::add_cluster,
            commands::kube::connect_cluster_from_kubeconfig,
            commands::kube::test_kubectl_connection,
            commands::kube::remove_cluster,
            commands::kube::list_clusters,
            commands::kube::start_port_forward,
            commands::kube::stop_port_forward,
            commands::kube::list_port_forwards,
            commands::kube::delete_port_forward,
            commands::kube::shutdown_port_forwards,
            commands::kube::test_cluster_connection,
            commands::kube::discover_pods,
            // Kubernetes Resource Discovery
            commands::kube::list_namespaces,
            commands::kube::list_pods,
            commands::kube::list_services,
            commands::kube::list_deployments,
            commands::kube::list_statefulsets,
            commands::kube::list_daemonsets,
            // Additional Kubernetes Resource Discovery
            commands::kube::list_replicasets,
            commands::kube::list_jobs,
            commands::kube::list_cronjobs,
            commands::kube::list_configmaps,
            commands::kube::list_secrets,
            commands::kube::list_nodes,
            commands::kube::list_events,
            commands::kube::list_ingresses,
            commands::kube::list_persistentvolumeclaims,
            commands::kube::list_persistentvolumes,
            commands::kube::list_serviceaccounts,
            commands::kube::list_roles,
            commands::kube::list_clusterroles,
            commands::kube::list_rolebindings,
            commands::kube::list_clusterrolebindings,
            commands::kube::list_horizontalpodautoscalers,
            commands::kube::list_storageclasses,
            commands::kube::list_networkpolicies,
            commands::kube::list_resourcequotas,
            commands::kube::list_limitranges,
            // Kubernetes Resource Management
            commands::kube::get_pod_logs,
            commands::kube::scale_deployment,
            commands::kube::restart_deployment,
            commands::kube::delete_resource,
            commands::kube::exec_pod,
            // Additional Kubernetes Resource Management
            commands::kube::cordon_node,
            commands::kube::uncordon_node,
            commands::kube::drain_node,
            commands::kube::rollback_deployment,
            commands::kube::create_resource,
            commands::kube::edit_resource,
            // Phase 4: Additional Resource Discovery
            commands::kube::list_replicationcontrollers,
            commands::kube::list_poddisruptionbudgets,
            commands::kube::list_priorityclasses,
            commands::kube::list_runtimeclasses,
            commands::kube::list_leases,
            commands::kube::list_mutatingwebhookconfigurations,
            commands::kube::list_validatingwebhookconfigurations,
            commands::kube::list_endpoints,
            commands::kube::list_endpointslices,
            commands::kube::list_ingressclasses,
            commands::kube::list_namespaces_resource,
            commands::kube::list_crds,
            commands::kube::list_custom_resources,
            // Phase 5: Action Commands
            commands::kube::force_delete_resource,
            commands::kube::describe_resource,
            commands::kube::get_resource_yaml,
            commands::kube::attach_pod,
            commands::kube::restart_statefulset,
            commands::kube::restart_daemonset,
            commands::kube::scale_statefulset,
            commands::kube::scale_replicaset,
            commands::kube::scale_replicationcontroller,
            commands::kube::suspend_cronjob,
            commands::kube::resume_cronjob,
            commands::kube::trigger_cronjob,
            commands::kube::create_namespace,
            commands::kube::delete_namespace,
            // Phase 6: Log Streaming
            commands::kube::stream_pod_logs,
            commands::kube::stop_log_stream,
            // Phase 7: Helm Commands
            commands::kube::helm_list_repos,
            commands::kube::helm_add_repo,
            commands::kube::helm_update_repos,
            commands::kube::helm_search_repo,
            commands::kube::helm_list_releases,
            commands::kube::helm_uninstall,
            commands::kube::helm_rollback,
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
