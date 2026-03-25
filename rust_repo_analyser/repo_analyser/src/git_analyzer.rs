use git2::{Delta, DiffFindOptions, DiffOptions, Repository};
use log::info;
use std::collections::HashMap;

use crate::file_graph::ChangedFile;
use crate::storage::Neo4jClient;

pub struct GitAnalyzer {
    repo_path: String,
    rename_map: HashMap<String, String>,
    current_names: HashMap<String, String>,
}

impl GitAnalyzer {
    pub fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            rename_map: HashMap::new(),
            current_names: HashMap::new(),
        }
    }

    pub async fn analyze(&mut self, client: &Neo4jClient, repo_name: &str) -> Result<i64, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        info!("Analyzing repository: {}", repo_name);

        client
            .save_repository(repo_name, None)
            .await
            .map_err(|e| format!("Failed to save repository: {}", e))?;

        let mut revwalk = repo
            .revwalk()
            .map_err(|err| format!("Error creating revwalk: {}", err))?;

        revwalk
            .push_head()
            .map_err(|err| format!("Error pushing revwalk {}", err))?;
        revwalk
            .set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)
            .map_err(|err| format!("Sorting failed {}", err))?;

        let mut commit_count = 0i64;

        for rev in revwalk {
            let current_commit = rev.map_err(|err| format!("Error unwrapping revwalk:{}", err))?;
            let commit = repo
                .find_commit(current_commit)
                .map_err(|e| format!("Failed to find commit: {}", e))?;
            let commit_id = commit.id().to_string();
            let changed_files = self.get_changed_files(&repo, &commit)?;
            if !changed_files.is_empty() {
                self.save_to_neo4j(client, repo_name, &changed_files, Some(&commit_id))
                    .await?;
            }

            commit_count += 1;
            if commit_count % 100 == 0 {
                info!("Processed {} commits", commit_count);
            }
        }

        client
            .update_commit_count(repo_name, commit_count)
            .await
            .map_err(|e| format!("Failed to update commit count: {}", e))?;

        client
            .link_all_files_to_repo(repo_name)
            .await
            .map_err(|e| format!("Failed to link files to repo: {}", e))?;

        info!("Total commits analyzed: {}", commit_count);

        Ok(commit_count)
    }

    async fn save_to_neo4j(
        &mut self,
        client: &Neo4jClient,
        repo_name: &str,
        changed_files: &[ChangedFile],
        commit_id: Option<&str>,
    ) -> Result<(), String> {
        let mut unique_files: Vec<&str> = changed_files.iter().map(|f| f.path.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        for file in changed_files {
            let deleted_at_commit = if file.is_deleted { commit_id } else { None };
            client
                .save_file_node(
                    repo_name,
                    &file.path,
                    file.additions as i64,
                    file.deletions as i64,
                    deleted_at_commit,
                )
                .await
                .map_err(|e| format!("Failed to save file node: {}", e))?;
        }

        for i in 0..unique_files.len() {
            for j in (i + 1)..unique_files.len() {
                let source = unique_files[i];
                let target = unique_files[j];

                client
                    .save_cochange_relationship(repo_name, source, target)
                    .await
                    .map_err(|e| format!("Failed to save co-change relationship: {}", e))?;
            }
        }

        Ok(())
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

        let mut diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))
            .map_err(|e| format!("Failed to get diff: {}", e))?;

        let mut find_opts = DiffFindOptions::new();
        find_opts.renames(true);
        diff.find_similar(Some(&mut find_opts))
            .map_err(|e| format!("Failed to find similar: {}", e))?;

        let mut commit_path_to_canonical: HashMap<String, String> = HashMap::new();
        let mut new_renames = Vec::new();
        let mut commit_paths = Vec::new();
        let mut deleted_paths = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if delta.status() == Delta::Deleted {
                    if let Some(path) = delta.old_file().path() {
                        deleted_paths.push(path.to_string_lossy().to_string());
                    }
                }
                true
            },
            None,
            Some(&mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    let path_str = path.to_string_lossy().to_string();
                    commit_paths.push(path_str.clone());

                    if delta.status() == Delta::Renamed {
                        if let Some(old_path) = delta.old_file().path() {
                            let old_path_str = old_path.to_string_lossy().to_string();
                            new_renames.push((old_path_str, path_str));
                        }
                    }
                }
                true
            }),
            None,
        )
        .map_err(|e| format!("Failed to iterate diff: {}", e))?;

        for (old_path, new_path) in new_renames {
            self.rename_map.insert(old_path.clone(), new_path.clone());
            self.current_names.insert(old_path, new_path.clone());
            self.current_names
                .insert(new_path.clone(), new_path.clone());
            commit_path_to_canonical.insert(new_path.clone(), new_path);
        }

        for path_str in &commit_paths {
            if commit_path_to_canonical.contains_key(path_str) {
                continue;
            }
            let canonical = self
                .current_names
                .get(path_str)
                .cloned()
                .unwrap_or_else(|| path_str.clone());
            commit_path_to_canonical.insert(path_str.clone(), canonical);
        }

        let mut file_stats: HashMap<String, (u32, u32, bool)> = HashMap::new();

        for deleted in &deleted_paths {
            file_stats.entry(deleted.clone()).or_insert((0, 0, true));
        }

        diff.foreach(
            &mut |_, _| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path() {
                    let path_str = path.to_string_lossy().to_string();
                    let stats = file_stats.entry(path_str).or_insert((0, 0, false));
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

        let result: Vec<ChangedFile> = commit_path_to_canonical
            .into_iter()
            .map(|(commit_path, canonical_path)| {
                let (additions, deletions, is_deleted) =
                    file_stats.remove(&commit_path).unwrap_or((0, 0, false));
                ChangedFile {
                    path: canonical_path,
                    additions,
                    deletions,
                    is_deleted,
                }
            })
            .collect();

        Ok(result)
    }

    pub fn get_rename_map(&self) -> &HashMap<String, String> {
        &self.rename_map
    }
}
