use serde::{Deserialize, Serialize};

/// A single node in the DAG — represents a skill or processing stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub node_id: String,
    pub node_type: String,
    pub upstream: Vec<String>,
    pub downstream: Vec<String>,
    pub flow_capacity: f64,
}

/// A dataset-to-node binding from the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetBinding {
    pub id: String,
    pub domain: String,
    pub nodes: Vec<String>,
}

/// Top-level schema container — loaded from schema.json at compile time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub datasets: Vec<DatasetBinding>,
}

impl Schema {
    /// Build a DAG from the dataset bindings.
    /// Each dataset contributes its two nodes; edges connect them bidirectionally
    /// with default capacity 1.0.
    pub fn to_dag(&self) -> Vec<DagNode> {
        use std::collections::HashMap;

        let mut node_map: HashMap<String, DagNode> = HashMap::new();

        for ds in &self.datasets {
            for w in ds.nodes.windows(2) {
                let from = &w[0];
                let to = &w[1];
                // Ensure nodes exist
                node_map
                    .entry(from.clone())
                    .or_insert_with(|| DagNode {
                        node_id: from.clone(),
                        node_type: "skill".into(),
                        upstream: vec![],
                        downstream: vec![],
                        flow_capacity: 1.0,
                    });
                node_map
                    .entry(to.clone())
                    .or_insert_with(|| DagNode {
                        node_id: to.clone(),
                        node_type: "skill".into(),
                        upstream: vec![],
                        downstream: vec![],
                        flow_capacity: 1.0,
                    });

                // Add edges
                if let Some(n) = node_map.get_mut(from) {
                    if !n.downstream.contains(to) {
                        n.downstream.push(to.clone());
                    }
                }
                if let Some(n) = node_map.get_mut(to) {
                    if !n.upstream.contains(from) {
                        n.upstream.push(from.clone());
                    }
                }
            }
        }

        node_map.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_loading() {
        let schema_json = include_str!("../schema.json");
        let schema: Schema = serde_json::from_str(schema_json).expect("schema.json must be valid");
        assert_eq!(schema.datasets.len(), 14);
    }

    #[test]
    fn test_to_dag() {
        let schema_json = include_str!("../schema.json");
        let schema: Schema = serde_json::from_str(schema_json).unwrap();
        let dag = schema.to_dag();
        // We expect 28 unique node entries (2 per dataset, may overlap)
        assert!(dag.len() >= 14);
        assert!(dag.len() <= 28);
    }
}
