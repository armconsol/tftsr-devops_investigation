// ER Diagram generation for database schema visualization

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    pool::DatabasePoolManager,
};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete ER diagram data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERDiagramData {
    pub nodes: Vec<TableNode>,
    pub edges: Vec<ForeignKeyEdge>,
}

/// Table node in ER diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNode {
    pub id: String,
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub x: f64,
    pub y: f64,
    pub row_count: Option<usize>,
}

/// Column information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub foreign_key: bool,
}

/// Foreign key edge in ER diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyEdge {
    pub id: String,
    pub name: String,
    pub from_table: String,
    pub to_table: String,
    pub from_column: String,
    pub to_column: String,
    pub on_delete: String,
    pub on_update: String,
}

/// Generate ER diagram for a database
///
/// # Arguments
/// * `connection_id` - Database connection ID
/// * `database` - Database name (optional, uses current database if None)
/// * `pool` - Database pool manager
///
/// # Returns
/// ERDiagramData with nodes positioned using force-directed layout
pub async fn generate_er_diagram(
    connection_id: &str,
    database: Option<&str>,
    pool: &DatabasePoolManager,
) -> DriverResult<ERDiagramData> {
    // Get driver from pool
    let driver = pool.get_driver(connection_id).await?;
    let driver_lock = driver.read().await;

    // Get schema for database
    let schema = if let Some(db_name) = database {
        driver_lock.get_schema(db_name).await?
    } else {
        return Err(DriverError::ValidationError(
            "Database name must be specified".to_string(),
        ));
    };

    // Build nodes
    let mut nodes = Vec::new();
    let mut node_indices: HashMap<String, usize> = HashMap::new();

    for (idx, table) in schema.tables.iter().enumerate() {
        // Identify foreign key columns
        let fk_columns: std::collections::HashSet<String> = table
            .foreign_keys
            .iter()
            .flat_map(|fk| fk.from_columns.iter().cloned())
            .collect();

        let columns: Vec<ColumnInfo> = table
            .columns
            .iter()
            .map(|col| ColumnInfo {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                primary_key: col.primary_key,
                foreign_key: fk_columns.contains(&col.name),
            })
            .collect();

        let node = TableNode {
            id: table.name.clone(),
            name: table.name.clone(),
            schema: table.schema.clone(),
            columns,
            x: 0.0, // Will be computed by layout algorithm
            y: 0.0,
            row_count: table.row_count,
        };

        node_indices.insert(table.name.clone(), idx);
        nodes.push(node);
    }

    // Build edges from foreign keys
    let mut edges = Vec::new();
    let mut graph_edges = Vec::new();

    for table in &schema.tables {
        if let Some(&from_idx) = node_indices.get(&table.name) {
            for fk in &table.foreign_keys {
                if let Some(&to_idx) = node_indices.get(&fk.to_table) {
                    let edge = ForeignKeyEdge {
                        id: format!("{}_{}", fk.from_table, fk.to_table),
                        name: fk.name.clone(),
                        from_table: fk.from_table.clone(),
                        to_table: fk.to_table.clone(),
                        from_column: fk.from_columns.join(", "),
                        to_column: fk.to_columns.join(", "),
                        on_delete: fk.on_delete.clone(),
                        on_update: fk.on_update.clone(),
                    };
                    edges.push(edge);
                    graph_edges.push((from_idx, to_idx));
                }
            }
        }
    }

    // Apply layout algorithm if we have nodes
    if !nodes.is_empty() {
        apply_layout(&mut nodes, &graph_edges);
    }

    Ok(ERDiagramData { nodes, edges })
}

/// Apply force-directed layout algorithm to position nodes
///
/// Uses a simple force-directed algorithm with:
/// - Spring forces between connected nodes
/// - Repulsive forces between all nodes
/// - Center gravity to keep diagram compact
fn apply_layout(nodes: &mut [TableNode], edges: &[(usize, usize)]) {
    if nodes.is_empty() {
        return;
    }

    // Build graph for analysis
    let mut graph: DiGraph<(), ()> = DiGraph::new();
    let mut node_map = Vec::new();

    for _ in 0..nodes.len() {
        node_map.push(graph.add_node(()));
    }

    for (from, to) in edges {
        graph.add_edge(node_map[*from], node_map[*to], ());
    }

    // Find strongly connected components for hierarchical layout
    let sccs = kosaraju_scc(&graph);

    // Initialize positions in a hierarchical layout based on SCCs
    let mut layer_y = 0.0;
    let layer_spacing = 300.0;
    let node_spacing = 250.0;

    for scc in &sccs {
        let mut layer_x = -(scc.len() as f64 * node_spacing) / 2.0;

        for &node_idx in scc {
            let idx = node_map.iter().position(|&n| n == node_idx).unwrap();
            nodes[idx].x = layer_x;
            nodes[idx].y = layer_y;
            layer_x += node_spacing;
        }

        layer_y += layer_spacing;
    }

    // Apply force-directed refinement (simplified Fruchterman-Reingold)
    let iterations = 50;
    let k = 150.0; // Optimal distance between nodes
    let area = (nodes.len() as f64) * 100000.0;
    let mut temperature = area.sqrt() * 0.1;
    let cooling_factor = 0.95;

    for _ in 0..iterations {
        // Calculate repulsive forces between all nodes
        let mut forces = vec![(0.0, 0.0); nodes.len()];

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let dx = nodes[i].x - nodes[j].x;
                let dy = nodes[i].y - nodes[j].y;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                let force = k * k / distance;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;

                forces[i].0 += fx;
                forces[i].1 += fy;
                forces[j].0 -= fx;
                forces[j].1 -= fy;
            }
        }

        // Calculate attractive forces for connected nodes
        for (from, to) in edges {
            let dx = nodes[*from].x - nodes[*to].x;
            let dy = nodes[*from].y - nodes[*to].y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);

            let force = distance * distance / k;
            let fx = (dx / distance) * force;
            let fy = (dy / distance) * force;

            forces[*from].0 -= fx;
            forces[*from].1 -= fy;
            forces[*to].0 += fx;
            forces[*to].1 += fy;
        }

        // Apply forces with temperature cooling
        for (i, node) in nodes.iter_mut().enumerate() {
            let force_magnitude = (forces[i].0 * forces[i].0 + forces[i].1 * forces[i].1).sqrt();
            if force_magnitude > 0.0 {
                let displacement = temperature.min(force_magnitude);
                node.x += (forces[i].0 / force_magnitude) * displacement;
                node.y += (forces[i].1 / force_magnitude) * displacement;
            }
        }

        temperature *= cooling_factor;
    }

    // Center the diagram
    if !nodes.is_empty() {
        let center_x = nodes.iter().map(|n| n.x).sum::<f64>() / nodes.len() as f64;
        let center_y = nodes.iter().map(|n| n.y).sum::<f64>() / nodes.len() as f64;

        for node in nodes.iter_mut() {
            node.x -= center_x;
            node.y -= center_y;
        }
    }
}

/// Generate simplified ER diagram for tables without foreign keys
///
/// Uses a simple grid layout when no relationships exist
pub async fn generate_simple_er_diagram(
    connection_id: &str,
    database: Option<&str>,
    pool: &DatabasePoolManager,
) -> DriverResult<ERDiagramData> {
    let mut diagram = generate_er_diagram(connection_id, database, pool).await?;

    // If no edges, use simple grid layout
    if diagram.edges.is_empty() && !diagram.nodes.is_empty() {
        let cols = (diagram.nodes.len() as f64).sqrt().ceil() as usize;
        let spacing = 300.0;

        for (idx, node) in diagram.nodes.iter_mut().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            node.x = col as f64 * spacing - (cols as f64 * spacing / 2.0);
            node.y = row as f64 * spacing;
        }
    }

    Ok(diagram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_layout_empty() {
        let mut nodes = vec![];
        let edges = vec![];
        apply_layout(&mut nodes, &edges);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_apply_layout_single_node() {
        let mut nodes = vec![TableNode {
            id: "users".to_string(),
            name: "users".to_string(),
            schema: None,
            columns: vec![],
            x: 0.0,
            y: 0.0,
            row_count: None,
        }];
        let edges = vec![];
        apply_layout(&mut nodes, &edges);
        // Single node should be centered
        assert_eq!(nodes[0].x, 0.0);
        assert_eq!(nodes[0].y, 0.0);
    }

    #[test]
    fn test_apply_layout_two_connected_nodes() {
        let mut nodes = vec![
            TableNode {
                id: "users".to_string(),
                name: "users".to_string(),
                schema: None,
                columns: vec![],
                x: 0.0,
                y: 0.0,
                row_count: None,
            },
            TableNode {
                id: "posts".to_string(),
                name: "posts".to_string(),
                schema: None,
                columns: vec![],
                x: 0.0,
                y: 0.0,
                row_count: None,
            },
        ];
        let edges = vec![(1, 0)]; // posts -> users

        apply_layout(&mut nodes, &edges);

        // After layout, nodes should be separated
        let distance =
            ((nodes[0].x - nodes[1].x).powi(2) + (nodes[0].y - nodes[1].y).powi(2)).sqrt();
        assert!(distance > 0.0, "Nodes should be separated after layout");
    }
}
