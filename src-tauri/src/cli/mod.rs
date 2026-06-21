// CLI tools for TFTSR Proxmox Management
// Provides command-line interface for Proxmox operations

#![allow(dead_code, clippy::too_many_arguments)]

use anyhow::Result;
use std::process;

/// TFTSR Proxmox CLI - Command-line interface for Proxmox VE/PBS management
/// Note: This module provides CLI functionality using environment variables and arguments
struct Cli {
    url: String,
    username: String,
    password: String,
    insecure: bool,
    command: String,
    args: Vec<String>,
}

impl Cli {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let url = std::env::var("PVE_URL").unwrap_or_else(|_| "https://localhost:8006".to_string());
        let username = std::env::var("PVE_USERNAME").unwrap_or_else(|_| "root@pam".to_string());
        let password = std::env::var("PVE_PASSWORD").unwrap_or_default();
        let insecure = std::env::var("PVE_INSECURE").is_ok();

        let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
        let args: Vec<String> = args.iter().skip(2).map(|s| s.to_string()).collect();

        Self {
            url,
            username,
            password,
            insecure,
            command: command.to_string(),
            args,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut client = crate::proxmox::client::ProxmoxClient::new(&cli.url, 8006, &cli.username);

    let ticket = match client.authenticate(&cli.password).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            process::exit(1);
        }
    };

    let result = match cli.command.as_str() {
        "list-clusters" => list_clusters(&client).await,
        "list-vms" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_vms(&client, &cluster, &ticket).await
        }
        "list-pools" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_pools(&client, &cluster, &ticket).await
        }
        "list-osds" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_osds(&client, &cluster, &ticket).await
        }
        "ceph-health" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            get_ceph_health(&client, &cluster, &ticket).await
        }
        "list-realms" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_realms(&client, &cluster, &ticket).await
        }
        "list-updates" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            let node = cli.args.get(1).cloned().unwrap_or_default();
            list_updates(&client, &cluster, &node, &ticket).await
        }
        "shell-ticket" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            let remote = cli.args.get(1).cloned().unwrap_or_default();
            get_shell_ticket(&client, &cluster, &remote, &ticket).await
        }
        "list-views" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_views(&client, &cluster, &ticket).await
        }
        "list-certificates" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_certificates(&client, &cluster, &ticket).await
        }
        "list-firewall-rules" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            let node = cli.args.get(1).cloned().unwrap_or_default();
            list_firewall_rules(&client, &cluster, &node, &ticket).await
        }
        "list-sdn-controllers" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_sdn_controllers(&client, &cluster, &ticket).await
        }
        "list-sdn-vnets" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_sdn_vnets(&client, &cluster, &ticket).await
        }
        "list-sdn-zones" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_sdn_zones(&client, &cluster, &ticket).await
        }
        "list-ceph-clusters" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            list_ceph_clusters(&client, &cluster, &ticket).await
        }
        "list-migrations" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            let node = cli.args.get(1).cloned().unwrap_or_default();
            list_migrations(&client, &cluster, &node, &ticket).await
        }
        "list-tasks" => {
            let cluster = cli.args.first().cloned().unwrap_or_default();
            let node = cli.args.get(1).cloned().unwrap_or_default();
            list_tasks(&client, &cluster, &node, &ticket).await
        }
        "help" => {
            print_help();
            return;
        }
        _ => {
            print_help();
            return;
        }
    };

    match result {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("TFTSR Proxmox CLI - Command-line interface for Proxmox VE/PBS management");
    println!();
    println!("Usage: tftsr-proxmox <command> [args...]");
    println!();
    println!("Environment Variables:");
    println!("  PVE_URL       Proxmox base URL (default: https://localhost:8006)");
    println!("  PVE_USERNAME  Username (default: root@pam)");
    println!("  PVE_PASSWORD  Password or API token (required)");
    println!("  PVE_INSECURE  Skip SSL verification (optional)");
    println!();
    println!("Commands:");
    println!("  list-clusters                    List Proxmox clusters");
    println!("  list-vms [cluster-id]            List VMs on a cluster");
    println!("  list-pools [cluster-id]          List Ceph pools");
    println!("  list-osds [cluster-id]           List Ceph OSDs");
    println!("  ceph-health [cluster-id]         Get Ceph health");
    println!("  list-realms [cluster-id]         List authentication realms");
    println!("  list-updates [cluster-id] [node] List APT updates");
    println!("  shell-ticket [cluster-id] [remote] Get shell ticket for remote access");
    println!("  list-views [cluster-id]          List dashboard views");
    println!("  list-certificates [cluster-id]   List certificates");
    println!("  list-firewall-rules [cluster-id] [node] List firewall rules");
    println!("  list-sdn-controllers [cluster-id]       List SDN controllers");
    println!("  list-sdn-vnets [cluster-id]             List SDN virtual networks");
    println!("  list-sdn-zones [cluster-id]             List SDN zones");
    println!("  list-ceph-clusters [cluster-id]         List Ceph clusters");
    println!("  list-migrations [cluster-id] [node]     List migration tasks");
    println!("  list-tasks [cluster-id] [node]          List tasks");
}

async fn list_clusters(_client: &crate::proxmox::client::ProxmoxClient) -> Result<String, String> {
    Err("list-clusters not implemented in CLI mode".to_string())
}

async fn list_vms(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let vms = crate::proxmox::vm::list_vms(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list VMs: {}", e))?;

    serde_json::to_string_pretty(&vms).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_pools(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let pools = crate::proxmox::ceph::list_pools(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list pools: {}", e))?;

    serde_json::to_string_pretty(&pools).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_osds(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let osds = crate::proxmox::ceph::list_osds(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list OSDs: {}", e))?;

    serde_json::to_string_pretty(&osds).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn get_ceph_health(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let health = crate::proxmox::ceph::get_ceph_health(_client, ticket)
        .await
        .map_err(|e| format!("Failed to get Ceph health: {}", e))?;

    serde_json::to_string_pretty(&health).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_realms(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let realms = crate::proxmox::auth_realm::list_auth_realms(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list realms: {}", e))?;

    serde_json::to_string_pretty(&realms).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_updates(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let updates = crate::proxmox::apt::list_apt_updates(_client, node, ticket)
        .await
        .map_err(|e| format!("Failed to list updates: {}", e))?;

    serde_json::to_string_pretty(&updates).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn get_shell_ticket(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    remote: &str,
    ticket: &str,
) -> Result<String, String> {
    let shell_ticket = crate::proxmox::shell::get_shell_ticket(_client, remote, ticket)
        .await
        .map_err(|e| format!("Failed to get shell ticket: {}", e))?;

    serde_json::to_string_pretty(&shell_ticket).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_views(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let views = crate::proxmox::views::list_views(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list views: {}", e))?;

    serde_json::to_string_pretty(&views).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_certificates(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let certs = crate::proxmox::certificates::list_certificates(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list certificates: {}", e))?;

    serde_json::to_string_pretty(&certs).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_firewall_rules(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let rules = crate::proxmox::firewall::list_firewall_rules(_client, node, ticket)
        .await
        .map_err(|e| format!("Failed to list firewall rules: {}", e))?;

    serde_json::to_string_pretty(&rules).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_sdn_controllers(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let controllers = crate::proxmox::sdn::list_evpn_zones(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list SDN controllers: {}", e))?;

    serde_json::to_string_pretty(&controllers).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_sdn_vnets(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let vnets = crate::proxmox::sdn::list_vnets(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list SDN virtual networks: {}", e))?;

    serde_json::to_string_pretty(&vnets).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_sdn_zones(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let zones = crate::proxmox::sdn::list_evpn_zones(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list SDN zones: {}", e))?;

    serde_json::to_string_pretty(&zones).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_ceph_clusters(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    ticket: &str,
) -> Result<String, String> {
    let clusters = crate::proxmox::ceph_cluster::list_ceph_clusters(_client, ticket)
        .await
        .map_err(|e| format!("Failed to list Ceph clusters: {}", e))?;

    serde_json::to_string_pretty(&clusters).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_migrations(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let tasks = crate::proxmox::migration::list_migration_status(_client, node, ticket)
        .await
        .map_err(|e| format!("Failed to list migrations: {}", e))?;

    serde_json::to_string_pretty(&tasks).map_err(|e| format!("Failed to serialize: {}", e))
}

async fn list_tasks(
    _client: &crate::proxmox::client::ProxmoxClient,
    _cluster_id: &str,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let tasks = crate::proxmox::tasks::list_tasks(_client, node, ticket)
        .await
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    serde_json::to_string_pretty(&tasks).map_err(|e| format!("Failed to serialize: {}", e))
}
