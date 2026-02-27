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
pub struct Era {
    pub era_index: u32,
    pub commits_in_era: u32,
    pub reset_commit: Option<String>,
    pub nodes: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGraph {
    pub repo: String,
    pub total_commits_analyzed: u32,
    pub eras: Vec<Era>,
}

impl FileGraph {
    pub fn new(repo: String) -> Self {
        Self {
            repo,
            total_commits_analyzed: 0,
            eras: Vec::new(),
        }
    }
}

pub struct FileGraphBuilder {
    repo: String,
    eras: Vec<EraBuilder>,
    current_era_commits: u32,
    total_commits: u32,
}

struct EraBuilder {
    era_index: u32,
    nodes_map: HashMap<String, NodeBuilder>,
    commits_in_era: u32,
    reset_commit: Option<String>,
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
        let first_era = EraBuilder {
            era_index: 0,
            nodes_map: HashMap::new(),
            commits_in_era: 0,
            reset_commit: None,
        };
        Self {
            repo,
            eras: vec![first_era],
            current_era_commits: 0,
            total_commits: 0,
        }
    }

    pub fn add_commit(&mut self, changed_files: &[ChangedFile], commit_hash: Option<&str>) {
        self.total_commits += 1;
        self.current_era_commits += 1;

        let should_reset =
            self.current_era_commits > 10 && self.should_start_new_era(changed_files);

        if should_reset {
            self.start_new_era(commit_hash);
        }

        let era = self.eras.last_mut().unwrap();
        era.commits_in_era += 1;
        FileGraphBuilder::add_commit_to_era(era, changed_files);
    }

    fn get_deleted_files(&self, changed_files: &[ChangedFile]) -> Vec<String> {
        changed_files
            .iter()
            .filter(|f| f.is_deleted)
            .map(|f| f.path.clone())
            .collect()
    }

    fn get_top_10_files(&self) -> Vec<String> {
        let current_era = self.eras.last().unwrap();
        let mut file_weights: Vec<(String, u32)> = current_era
            .nodes_map
            .iter()
            .map(|(path, node)| {
                let total_weight: u32 = node.edges.values().map(|e| e.weight).sum();
                (path.clone(), total_weight)
            })
            .collect();

        file_weights.sort_by(|a, b| b.1.cmp(&a.1));
        file_weights.into_iter().take(10).map(|(p, _)| p).collect()
    }

    fn should_start_new_era(&self, changed_files: &[ChangedFile]) -> bool {
        if self.eras.len() > 10 {
            return false;
        }

        let top_10 = self.get_top_10_files();
        if top_10.len() < 10 {
            return false;
        }

        let deleted_files = self.get_deleted_files(changed_files);
        let deleted_count = top_10.iter().filter(|f| deleted_files.contains(*f)).count();

        let threshold = (top_10.len() as f32 * 0.4).ceil() as usize;
        deleted_count >= threshold
    }

    fn start_new_era(&mut self, commit_hash: Option<&str>) {
        let new_index = self.eras.last().map(|e| e.era_index + 1).unwrap_or(0);

        let new_era = EraBuilder {
            era_index: new_index,
            nodes_map: HashMap::new(),
            commits_in_era: 0,
            reset_commit: commit_hash.map(|s| s.to_string()),
        };

        self.eras.push(new_era);
        self.current_era_commits = 0;
    }

    fn add_commit_to_era(era: &mut EraBuilder, changed_files: &[ChangedFile]) {
        let mut unique_files: Vec<&str> = changed_files.iter().map(|f| f.path.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        for file in changed_files {
            let node = era
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

                let source_node = era.nodes_map.get_mut(source).unwrap();

                let edge = source_node
                    .edges
                    .entry(target.to_string())
                    .or_insert_with(|| EdgeBuilder {
                        target: target.to_string(),
                        weight: 0,
                    });
                edge.weight += 1;

                let target_node = era.nodes_map.get_mut(target).unwrap();
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
        let eras: Vec<Era> = self
            .eras
            .into_iter()
            .map(|era_builder| {
                let commit_counts: HashMap<String, u32> = era_builder
                    .nodes_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.commit_count))
                    .collect();

                let nodes_map = era_builder.nodes_map;

                let mut nodes: Vec<FileNode> = nodes_map
                    .into_values()
                    .map(|nb| {
                        let edges: Vec<Edge> = nb
                            .edges
                            .into_values()
                            .map(|eb| {
                                let target_commits =
                                    commit_counts.get(&eb.target).copied().unwrap_or(0);
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

                Era {
                    era_index: era_builder.era_index,
                    commits_in_era: era_builder.commits_in_era,
                    reset_commit: era_builder.reset_commit,
                    nodes,
                }
            })
            .collect();

        FileGraph {
            repo: self.repo,
            total_commits_analyzed: self.total_commits,
            eras,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub is_deleted: bool,
}
