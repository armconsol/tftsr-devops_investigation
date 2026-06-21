// Dashboard Views module
// Provides operations for managing custom dashboard views

use serde::{Deserialize, Serialize};

/// Dashboard view configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardView {
    pub view_id: String,
    pub name: String,
    pub description: String,
    pub layout: String,
    pub widgets: Vec<Widget>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub widget_id: String,
    pub type_: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position: WidgetPosition,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// List dashboard views
pub async fn list_views(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<DashboardView>, String> {
    let path = "config/views";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list dashboard views: {}", e))?;

    if let Some(views) = response.as_array() {
        let view_list: Vec<DashboardView> = views
            .iter()
            .filter_map(|view| {
                let id = view.get("id")?.as_str()?.to_string();
                let name = view.get("name")?.as_str().unwrap_or("").to_string();
                let description = view
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let layout = view
                    .get("layout")
                    .and_then(|l| l.as_str())
                    .unwrap_or("grid")
                    .to_string();
                let enabled = view
                    .get("enabled")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(true);
                let created_at = view
                    .get("created")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let updated_at = view
                    .get("updated")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();

                let widgets: Vec<Widget> = view
                    .get("widgets")
                    .and_then(|w| w.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|widget| {
                                let wid = widget.get("id")?.as_str()?.to_string();
                                let wtype = widget.get("type")?.as_str().unwrap_or("").to_string();
                                let title = widget
                                    .get("title")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let config = widget
                                    .get("config")
                                    .cloned()
                                    .unwrap_or(serde_json::json!({}));

                                let position = widget
                                    .get("position")
                                    .and_then(|p| {
                                        let x = p.get("x")?.as_u64()?;
                                        let y = p.get("y")?.as_u64()?;
                                        let w = p.get("width")?.as_u64()?;
                                        let h = p.get("height")?.as_u64()?;
                                        Some(WidgetPosition {
                                            x: x as u32,
                                            y: y as u32,
                                            width: w as u32,
                                            height: h as u32,
                                        })
                                    })
                                    .unwrap_or(WidgetPosition {
                                        x: 0,
                                        y: 0,
                                        width: 1,
                                        height: 1,
                                    });

                                Some(Widget {
                                    widget_id: wid,
                                    type_: wtype,
                                    title,
                                    config,
                                    position,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Some(DashboardView {
                    view_id: id,
                    name,
                    description,
                    layout,
                    widgets,
                    enabled,
                    created_at,
                    updated_at,
                })
            })
            .collect();

        Ok(view_list)
    } else {
        Ok(vec![])
    }
}

/// Add dashboard view
pub async fn add_view(
    client: &crate::proxmox::client::ProxmoxClient,
    view: &DashboardView,
    ticket: &str,
) -> Result<(), String> {
    let path = "config/views";
    let config = serde_json::json!({
        "id": view.view_id,
        "name": view.name,
        "description": view.description,
        "layout": view.layout,
        "widgets": view.widgets.iter().map(|w| {
            serde_json::json!({
                "id": w.widget_id,
                "type": w.type_,
                "title": w.title,
                "config": w.config,
                "position": {
                    "x": w.position.x,
                    "y": w.position.y,
                    "width": w.position.width,
                    "height": w.position.height
                }
            })
        }).collect::<Vec<_>>(),
        "enabled": view.enabled
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add dashboard view {}: {}", view.view_id, e))?;
    Ok(())
}

/// Update dashboard view
pub async fn update_view(
    client: &crate::proxmox::client::ProxmoxClient,
    view_id: &str,
    view: &DashboardView,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("config/views/{}", view_id);
    let config = serde_json::json!({
        "name": view.name,
        "description": view.description,
        "layout": view.layout,
        "widgets": view.widgets.iter().map(|w| {
            serde_json::json!({
                "id": w.widget_id,
                "type": w.type_,
                "title": w.title,
                "config": w.config,
                "position": {
                    "x": w.position.x,
                    "y": w.position.y,
                    "width": w.position.width,
                    "height": w.position.height
                }
            })
        }).collect::<Vec<_>>(),
        "enabled": view.enabled
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update dashboard view {}: {}", view_id, e))?;
    Ok(())
}

/// Delete dashboard view
pub async fn delete_view(
    client: &crate::proxmox::client::ProxmoxClient,
    view_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("config/views/{}", view_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete dashboard view {}: {}", view_id, e))?;
    Ok(())
}

/// Get dashboard view
pub async fn get_view(
    client: &crate::proxmox::client::ProxmoxClient,
    view_id: &str,
    ticket: &str,
) -> Result<DashboardView, String> {
    let path = format!("config/views/{}", view_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get dashboard view {}: {}", view_id, e))?;

    {
        let data = &response;
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let name = data
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let description = data
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();
        let layout = data
            .get("layout")
            .and_then(|l| l.as_str())
            .unwrap_or("grid")
            .to_string();
        let enabled = data
            .get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(true);
        let created_at = data
            .get("created")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let updated_at = data
            .get("updated")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        let widgets: Vec<Widget> = data
            .get("widgets")
            .and_then(|w| w.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|widget| {
                        let wid = widget.get("id")?.as_str()?.to_string();
                        let wtype = widget.get("type")?.as_str().unwrap_or("").to_string();
                        let title = widget
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let config = widget
                            .get("config")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));

                        let position = widget
                            .get("position")
                            .and_then(|p| {
                                let x = p.get("x")?.as_u64()?;
                                let y = p.get("y")?.as_u64()?;
                                let w = p.get("width")?.as_u64()?;
                                let h = p.get("height")?.as_u64()?;
                                Some(WidgetPosition {
                                    x: x as u32,
                                    y: y as u32,
                                    width: w as u32,
                                    height: h as u32,
                                })
                            })
                            .unwrap_or(WidgetPosition {
                                x: 0,
                                y: 0,
                                width: 1,
                                height: 1,
                            });

                        Some(Widget {
                            widget_id: wid,
                            type_: wtype,
                            title,
                            config,
                            position,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(DashboardView {
            view_id: id,
            name,
            description,
            layout,
            widgets,
            enabled,
            created_at,
            updated_at,
        })
    }
}
