// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/graph.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Entity relationship graph for cross-entity correlation

use std::collections::{HashMap, HashSet};

/// Graph node (entity)
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub entity_id: String,
    pub neighbors: HashSet<String>,
    pub metadata: HashMap<String, String>,
}

/// Entity relationship graph
pub struct EntityGraph {
    /// Nodes by entity ID
    nodes: HashMap<String, GraphNode>,
    /// Maximum nodes (bounded)
    max_nodes: usize,
}

impl EntityGraph {
    /// Create new graph
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            max_nodes,
        }
    }

    /// Add or update node
    pub fn add_node(&mut self, entity_id: String, metadata: HashMap<String, String>) {
        // Evict if at capacity (simple: remove oldest)
        if self.nodes.len() >= self.max_nodes && !self.nodes.contains_key(&entity_id) {
            // Remove first node (simple eviction)
            if let Some(first_key) = self.nodes.keys().next().cloned() {
                self.remove_node(&first_key);
            }
        }

        let node = GraphNode {
            entity_id: entity_id.clone(),
            neighbors: HashSet::new(),
            metadata,
        };
        self.nodes.insert(entity_id, node);
    }

    /// Add edge (relationship) between entities
    pub fn add_edge(&mut self, from: &str, to: &str) {
        if let Some(node) = self.nodes.get_mut(from) {
            node.neighbors.insert(to.to_string());
        }
        if let Some(node) = self.nodes.get_mut(to) {
            node.neighbors.insert(from.to_string());
        }
    }

    /// Remove node and all its edges
    pub fn remove_node(&mut self, entity_id: &str) {
        if let Some(node) = self.nodes.remove(entity_id) {
            // Remove edges from neighbors
            for neighbor_id in &node.neighbors {
                if let Some(neighbor) = self.nodes.get_mut(neighbor_id) {
                    neighbor.neighbors.remove(entity_id);
                }
            }
        }
    }

    /// Get node
    pub fn get_node(&self, entity_id: &str) -> Option<&GraphNode> {
        self.nodes.get(entity_id)
    }

    /// Get neighbors of entity
    pub fn get_neighbors(&self, entity_id: &str) -> Vec<String> {
        self.nodes
            .get(entity_id)
            .map(|n| n.neighbors.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Find path between entities (BFS)
    pub fn find_path(&self, from: &str, to: &str, max_depth: usize) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to.to_string()];
                let mut node = to;
                while let Some(p) = parent.get(node) {
                    path.push(p.clone());
                    if p == from {
                        break;
                    }
                    node = p;
                }
                path.reverse();
                return Some(path);
            }

            if let Some(node) = self.nodes.get(&current) {
                for neighbor in &node.neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        None
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_operations() {
        let mut graph = EntityGraph::new(100);
        
        graph.add_node("entity1".to_string(), HashMap::new());
        graph.add_node("entity2".to_string(), HashMap::new());
        graph.add_edge("entity1", "entity2");
        
        assert_eq!(graph.get_neighbors("entity1").len(), 1);
        assert!(graph.find_path("entity1", "entity2", 10).is_some());
    }
}

