pub mod ai;
pub mod audit;
pub mod cli;
pub mod commands;
pub mod db;
pub mod docs;
pub mod integrations;
pub mod kube;
pub mod mcp;
pub mod metrics;
pub mod ollama;
pub mod pii;
pub mod proxmox;
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
    tracing::info!("Database initialized at {data_dir:?}");

    let app_state = AppState {
        db: Arc::new(Mutex::new(conn)),
        settings: Arc::new(Mutex::new(state::AppSettings::default())),
        app_data_dir: data_dir.clone(),
        integration_webviews: Arc::new(Mutex::new(std::collections::HashMap::new())),
        mcp_connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        pending_approvals: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        clusters: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        proxmox_clusters: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        port_forwards: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        refresh_registry: Arc::new(tokio::sync::Mutex::new(crate::kube::RefreshRegistry::new())),
        watchers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        log_streams: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        pty_sessions: Arc::new(crate::shell::SessionManager::new()),
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
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
            // Proxmox - Core Management (Phase 1)
            commands::proxmox::list_auth_realms,
            commands::proxmox::add_ldap_realm,
            commands::proxmox::add_ad_realm,
            commands::proxmox::add_openid_realm,
            commands::proxmox::list_acme_accounts,
            commands::proxmox::register_acme_account,
            commands::proxmox::get_acme_challenges,
            commands::proxmox::list_apt_updates,
            commands::proxmox::update_apt_repos,
            commands::proxmox::list_apt_repositories,
            commands::proxmox::get_shell_ticket,
            commands::proxmox::list_certificates,
            commands::proxmox::upload_certificate,
            commands::proxmox::get_certificate,
            // Proxmox - Advanced Management (Phase 2)
            commands::proxmox::list_firewall_rules,
            commands::proxmox::add_firewall_rule,
            commands::proxmox::delete_firewall_rule,
            commands::proxmox::list_sdn_controllers,
            commands::proxmox::list_sdn_vnets,
            commands::proxmox::list_sdn_zones,
            // Proxmox - Network Management (Phase 3)
            commands::proxmox::list_ceph_clusters,
            commands::proxmox::get_ceph_cluster_status,
            // Proxmox - Advanced Operations (Phase 4)
            commands::proxmox::migrate_vm,
            commands::proxmox::list_migration_status,
            commands::proxmox::list_updates,
            commands::proxmox::refresh_updates,
            commands::proxmox::install_updates,
            commands::proxmox::list_tasks,
            commands::proxmox::get_task_status,
            commands::proxmox::stop_task,
            // Proxmox - Infrastructure (Phase 5)
            commands::proxmox::get_metrics_summary,
            commands::proxmox::list_metric_collections,
            // Proxmox - HA Management (Phase 6)
            commands::proxmox::list_ha_groups,
            commands::proxmox::create_ha_group,
            commands::proxmox::update_ha_group,
            commands::proxmox::delete_ha_group,
            commands::proxmox::list_ha_resources,
            commands::proxmox::enable_ha_resource,
            // Proxmox - ACL / Users / Realms (Phase 7)
            commands::proxmox::list_acls,
            commands::proxmox::list_users,
            commands::proxmox::list_realms,
            // Proxmox - Cluster Notes (Phase 8)
            commands::proxmox::get_cluster_notes,
            commands::proxmox::update_cluster_notes,
            // Proxmox - Resource Search (Phase 9)
            commands::proxmox::search_proxmox_resources,
            // Proxmox - Node Status (Phase 10)
            commands::proxmox::get_node_status,
            // Proxmox - Syslog (Phase 11)
            commands::proxmox::get_syslog,
            // Proxmox - Network Interfaces (Phase 12)
            commands::proxmox::list_network_interfaces,
            commands::proxmox::create_network_interface,
            commands::proxmox::update_network_interface,
            commands::proxmox::delete_network_interface,
            // Proxmox - VM Snapshots (Phase 12b)
            commands::proxmox::list_proxmox_snapshots,
            commands::proxmox::create_proxmox_snapshot,
            commands::proxmox::delete_proxmox_snapshot,
            commands::proxmox::rollback_proxmox_snapshot,
            commands::proxmox::list_iso_images,
            commands::proxmox::upload_iso_image,
            // Proxmox - Subscription (Phase 14)
            commands::proxmox::get_subscription_status,
            // Proxmox - Cluster Tasks (Phase 15)
            commands::proxmox::list_cluster_tasks,
            // Proxmox - Existing
            commands::proxmox::add_proxmox_cluster,
            commands::proxmox::remove_proxmox_cluster,
            commands::proxmox::update_proxmox_cluster,
            commands::proxmox::ping_proxmox_cluster,
            commands::proxmox::connect_proxmox_cluster,
            commands::proxmox::disconnect_proxmox_cluster,
            commands::proxmox::list_proxmox_clusters,
            commands::proxmox::get_proxmox_cluster,
            commands::proxmox::list_proxmox_vms,
            commands::proxmox::list_proxmox_containers,
            commands::proxmox::get_proxmox_vm,
            commands::proxmox::start_proxmox_vm,
            commands::proxmox::stop_proxmox_vm,
            commands::proxmox::reboot_proxmox_vm,
            commands::proxmox::shutdown_proxmox_vm,
            commands::proxmox::resume_proxmox_vm,
            commands::proxmox::suspend_proxmox_vm,
            commands::proxmox::clone_vm,
            commands::proxmox::delete_vm,
            commands::proxmox::list_proxmox_nodes,
            commands::proxmox::create_proxmox_vm,
            commands::proxmox::list_proxmox_backup_jobs,
            commands::proxmox::list_proxmox_datastores,
            commands::proxmox::get_proxmox_storage_config,
            commands::proxmox::update_proxmox_storage,
            commands::proxmox::delete_proxmox_storage,
            commands::proxmox::trigger_proxmox_backup_job,
            commands::proxmox::list_ceph_pools,
            commands::proxmox::list_ceph_osd,
            commands::proxmox::get_ceph_health,
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
            commands::system::check_app_updates,
            commands::system::install_app_updates,
            commands::system::get_update_channel,
            commands::system::set_update_channel,
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
            // PTY Sessions
            commands::shell::start_pty_exec_session,
            commands::shell::start_pty_attach_session,
            commands::shell::send_pty_stdin,
            commands::shell::resize_pty_session,
            commands::shell::terminate_pty_session,
            commands::shell::list_pty_sessions,
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
            // Kubernetes Metrics
            commands::metrics::get_pod_metrics,
            commands::metrics::get_node_metrics,
            // Proxmox HA resource management
            commands::proxmox::disable_ha_resource,
            commands::proxmox::delete_ha_resource,
            commands::proxmox::update_ha_resource,
            // Proxmox Firewall Rule Update
            commands::proxmox::update_proxmox_firewall_rule,
            // Proxmox SDN CRUD
            commands::proxmox::create_sdn_zone,
            commands::proxmox::update_sdn_zone,
            commands::proxmox::delete_sdn_zone,
            commands::proxmox::create_sdn_vnet,
            commands::proxmox::update_sdn_vnet,
            commands::proxmox::delete_sdn_vnet,
            // Proxmox Backup Job CRUD
            commands::proxmox::create_proxmox_backup_job,
            commands::proxmox::update_proxmox_backup_job,
            commands::proxmox::delete_proxmox_backup_job,
            // Proxmox LXC Container Power
            commands::proxmox::start_proxmox_container,
            commands::proxmox::stop_proxmox_container,
            commands::proxmox::reboot_proxmox_container,
            commands::proxmox::shutdown_proxmox_container,
            commands::proxmox::suspend_proxmox_container,
            commands::proxmox::resume_proxmox_container,
            // Proxmox ACL CRUD
            commands::proxmox::create_proxmox_acl,
            commands::proxmox::delete_proxmox_acl,
            // Proxmox User CRUD
            commands::proxmox::create_proxmox_user,
            commands::proxmox::update_proxmox_user,
            commands::proxmox::delete_proxmox_user,
            // Proxmox Realm CRUD
            commands::proxmox::create_proxmox_realm,
            commands::proxmox::update_proxmox_realm,
            commands::proxmox::delete_proxmox_realm,
            // Proxmox Node Administration
            commands::proxmox::get_node_dns,
            commands::proxmox::update_node_dns,
            commands::proxmox::get_node_time,
            commands::proxmox::update_node_time,
            commands::proxmox::reboot_node,
            commands::proxmox::shutdown_node,
            commands::proxmox::get_node_journal,
            commands::proxmox::get_node_report,
            commands::proxmox::reload_network_config,
            // Proxmox VM/Container Config
            commands::proxmox::get_vm_config,
            commands::proxmox::get_vm_pending_config,
            commands::proxmox::remote_migrate_vm,
            commands::proxmox::start_remote_migration,
            commands::proxmox::open_vnc_console,
            commands::proxmox::open_lxc_console,
            commands::proxmox::open_node_shell,
            commands::proxmox::get_container_config,
            commands::proxmox::create_proxmox_container,
            // Proxmox RRD Metrics
            commands::proxmox::get_node_rrd_data,
            commands::proxmox::get_vm_rrd_data,
            commands::proxmox::get_storage_rrd_data,
            // Proxmox Ceph Advanced
            commands::proxmox::list_ceph_monitors,
            commands::proxmox::list_ceph_managers,
            commands::proxmox::list_cephfs,
            commands::proxmox::get_ceph_flags,
            // Proxmox Firewall (cluster + guest level)
            commands::proxmox::list_cluster_firewall_rules,
            commands::proxmox::get_cluster_firewall_status,
            commands::proxmox::list_guest_firewall_rules,
            commands::proxmox::add_guest_firewall_rule,
            commands::proxmox::delete_guest_firewall_rule,
            // Proxmox TFA Management
            commands::proxmox::list_tfa_entries,
            commands::proxmox::add_tfa_entry,
            commands::proxmox::delete_tfa_entry,
            // Proxmox User API Tokens
            commands::proxmox::list_user_tokens,
            commands::proxmox::create_user_token,
            commands::proxmox::delete_user_token,
            // Proxmox PBS Management
            commands::proxmox::list_pbs_datastores,
            commands::proxmox::get_pbs_datastore_status,
            commands::proxmox::list_pbs_namespaces,
            commands::proxmox::list_pbs_snapshots,
            commands::proxmox::list_pbs_tasks,
            commands::proxmox::get_pbs_node_status,
            // Proxmox Subscription Update
            commands::proxmox::update_subscription,
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
