use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub target: String,
    pub weight: u32,
    pub target_commits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub commit_count: u32,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGraph {
    pub repo: String,
    pub total_commits_analyzed: u32,
    pub nodes: Vec<FileNode>,
}

impl FileGraph {
    pub fn new(repo: String) -> Self {
        Self {
            repo,
            total_commits_analyzed: 0,
            nodes: Vec::new(),
        }
    }
}

pub struct FileGraphBuilder {
    repo: String,
    nodes_map: HashMap<String, NodeBuilder>,
    total_commits: u32,
}

struct NodeBuilder {
    path: String,
    additions: u32,
    deletions: u32,
    commit_count: u32,
    edges: HashMap<String, EdgeBuilder>,
}

struct EdgeBuilder {
    target: String,
    weight: u32,
}

impl FileGraphBuilder {
    pub fn new(repo: String) -> Self {
        Self {
            repo,
            nodes_map: HashMap::new(),
            total_commits: 0,
        }
    }

    pub fn add_commit(&mut self, changed_files: &[ChangedFile]) {
        self.total_commits += 1;

        let mut unique_files: Vec<&str> = changed_files.iter().map(|f| f.path.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        for file in changed_files {
            let node = self
                .nodes_map
                .entry(file.path.clone())
                .or_insert_with(|| NodeBuilder {
                    path: file.path.clone(),
                    additions: 0,
                    deletions: 0,
                    commit_count: 0,
                    edges: HashMap::new(),
                });

            node.additions += file.additions;
            node.deletions += file.deletions;
            node.commit_count += 1;
        }

        for i in 0..unique_files.len() {
            for j in (i + 1)..unique_files.len() {
                let source = unique_files[i];
                let target = unique_files[j];

                let source_node = self.nodes_map.get_mut(source).unwrap();

                let edge = source_node
                    .edges
                    .entry(target.to_string())
                    .or_insert_with(|| EdgeBuilder {
                        target: target.to_string(),
                        weight: 0,
                    });
                edge.weight += 1;

                let target_node = self.nodes_map.get_mut(target).unwrap();
                let edge_to_source =
                    target_node
                        .edges
                        .entry(source.to_string())
                        .or_insert_with(|| EdgeBuilder {
                            target: source.to_string(),
                            weight: 0,
                        });
                edge_to_source.weight += 1;
            }
        }
    }

    pub fn finalize(self) -> FileGraph {
        let commit_counts: HashMap<String, u32> = self
            .nodes_map
            .iter()
            .map(|(k, v)| (k.clone(), v.commit_count))
            .collect();

        let nodes_map = self.nodes_map;

        let mut nodes: Vec<FileNode> = nodes_map
            .into_values()
            .map(|nb| {
                let edges: Vec<Edge> = nb
                    .edges
                    .into_values()
                    .map(|eb| {
                        let target_commits = commit_counts.get(&eb.target).copied().unwrap_or(0);
                        Edge {
                            target: eb.target,
                            weight: eb.weight,
                            target_commits,
                        }
                    })
                    .collect();

                FileNode {
                    path: nb.path,
                    additions: nb.additions,
                    deletions: nb.deletions,
                    commit_count: nb.commit_count,
                    edges,
                }
            })
            .collect();

        nodes.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

        FileGraph {
            repo: self.repo,
            total_commits_analyzed: self.total_commits,
            nodes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
}
