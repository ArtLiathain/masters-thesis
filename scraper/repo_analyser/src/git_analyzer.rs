use git2::{DiffOptions, Repository};
use log::info;
use std::collections::HashMap;
use std::path::Path;

use crate::file_graph::{ChangedFile, FileGraph, FileGraphBuilder};

pub struct GitAnalyzer {
    repo_path: String,
    rename_map: HashMap<String, String>,
}

impl GitAnalyzer {
    pub fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            rename_map: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<FileGraph, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        let repo_name = Path::new(&self.repo_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        info!("Analyzing repository: {}", repo_name);

        let head = repo
            .head()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let head_oid = head.target().ok_or("HEAD has no target")?;

        let mut revwalk = repo
            .revwalk()
            .map_err(|e| format!("Failed to create revwalk: {}", e))?;
        revwalk
            .push(head_oid)
            .map_err(|e| format!("Failed to push HEAD: {}", e))?;

        let mut builder = FileGraphBuilder::new(repo_name);
        let mut commit_count = 0;

        for oid in revwalk {
            let oid = oid.map_err(|e| format!("Failed to get commit oid: {}", e))?;
            let commit = repo
                .find_commit(oid)
                .map_err(|e| format!("Failed to find commit: {}", e))?;

            let changed_files = self.get_changed_files(&repo, &commit)?;

            if !changed_files.is_empty() {
                builder.add_commit(&changed_files);
            }

            commit_count += 1;
            if commit_count % 100 == 0 {
                info!("Processed {} commits", commit_count);
            }
        }

        info!("Total commits analyzed: {}", commit_count);

        let mut graph = builder.finalize();
        graph.total_commits_analyzed = commit_count;

        Ok(graph)
    }

    fn get_changed_files(
        &mut self,
        repo: &Repository,
        commit: &git2::Commit,
    ) -> Result<Vec<ChangedFile>, String> {
        let tree = commit
            .tree()
            .map_err(|e| format!("Failed to get commit tree: {}", e))?;

        let parent_tree = if commit.parent_count() > 0 {
            let parent = commit
                .parent(0)
                .map_err(|e| format!("Failed to get parent: {}", e))?;
            Some(
                parent
                    .tree()
                    .map_err(|e| format!("Failed to get parent tree: {}", e))?,
            )
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts
            .include_untracked(true)
            .recurse_untracked_dirs(true);

        let diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))
            .map_err(|e| format!("Failed to get diff: {}", e))?;

        let mut changed_files = Vec::new();
        let mut new_renames = Vec::new();

        diff.foreach(
            &mut |_, _| true,
            None,
            Some(&mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    let path_str = path.to_string_lossy().to_string();

                    if let Some(old_path) = delta.old_file().path() {
                        let old_path_str = old_path.to_string_lossy().to_string();
                        if old_path_str != path_str {
                            new_renames.push((old_path_str, path_str.clone()));
                        }
                    }

                    changed_files.push(path_str);
                }
                true
            }),
            None,
        )
        .map_err(|e| format!("Failed to iterate diff: {}", e))?;

        for (old_path, new_path) in new_renames {
            self.rename_map.insert(old_path, new_path);
        }

        let mut file_stats: HashMap<String, (u32, u32)> = HashMap::new();

        diff.foreach(
            &mut |_, _| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path() {
                    let path_str = path.to_string_lossy().to_string();
                    let stats = file_stats.entry(path_str).or_insert((0, 0));
                    if line.origin() == '+' {
                        stats.0 += 1;
                    } else if line.origin() == '-' {
                        stats.1 += 1;
                    }
                }
                true
            }),
        )
        .map_err(|e| format!("Failed to count lines: {}", e))?;

        let result: Vec<ChangedFile> = changed_files
            .into_iter()
            .filter_map(|path| {
                let (additions, deletions) = file_stats.remove(&path).unwrap_or((0, 0));
                Some(ChangedFile {
                    path,
                    additions,
                    deletions,
                })
            })
            .collect();

        Ok(result)
    }

    pub fn get_rename_map(&self) -> &HashMap<String, String> {
        &self.rename_map
    }
}
