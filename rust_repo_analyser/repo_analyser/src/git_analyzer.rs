use git2::{Delta, DiffFindOptions, Repository};
use log::info;
use std::collections::HashSet;

use crate::file_graph::ChangedFile;
use crate::storage::Neo4jClient;

pub struct GitAnalyzer {
    repo_path: String,
    repo_url: String,
}

impl GitAnalyzer {
    pub fn new(repo_path: String, repo_url: String) -> Self {
        Self {
            repo_path,
            repo_url,
        }
    }

    pub async fn analyze(&self, client: &Neo4jClient, repo_name: &str) -> Result<i64, String> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        info!("Analyzing repository: {}", repo_name);

        client
            .save_repository(repo_name, Some(&self.repo_url))
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
            let (changed_files, renames) = self.get_changed_files(&repo, &commit)?;
            if !changed_files.is_empty() || !renames.is_empty() {
                self.save_to_neo4j(client, repo_name, &changed_files, &renames)
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
        &self,
        client: &Neo4jClient,
        repo_name: &str,
        changed_files: &[ChangedFile],
        renames: &[(String, String)],
    ) -> Result<(), String> {
        for (old_path, new_path) in renames {
            match client
                .rename_file_node(repo_name, old_path, new_path)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::warn!(
                        "Failed to rename file node {} -> {}: {}",
                        old_path,
                        new_path,
                        e
                    );
                }
            }
        }

        let mut unique_files: Vec<&str> = changed_files.iter().map(|f| f.path.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        for file in changed_files {
            client
                .save_file_node(
                    repo_name,
                    &file.path,
                    file.additions as i64,
                    file.deletions as i64,
                    None,
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
        &self,
        repo: &Repository,
        commit: &git2::Commit,
    ) -> Result<(Vec<ChangedFile>, Vec<(String, String)>), String> {
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
                    .map_err(|e| format!("Failed to get commit tree: {}", e))?,
            )
        } else {
            None
        };

        let mut diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
            .map_err(|e| format!("Failed to get diff: {}", e))?;

        diff.find_similar(Some(&mut DiffFindOptions::new().renames(true)))
            .map_err(|e| format!("Failed to find similar: {}", e))?;

        let mut renames: Vec<(String, String)> = Vec::new();
        let mut deleted_paths: HashSet<String> = HashSet::new();
        let mut all_paths: Vec<String> = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                match delta.status() {
                    Delta::Deleted => {
                        if let Some(path) = delta.old_file().path() {
                            deleted_paths.insert(path.to_string_lossy().to_string());
                        }
                    }
                    Delta::Renamed => {
                        if let Some(old_path) = delta.old_file().path() {
                            let old_path_str = old_path.to_string_lossy().to_string();
                            deleted_paths.remove(&old_path_str);
                        }
                        if let (Some(old_path), Some(new_path)) =
                            (delta.old_file().path(), delta.new_file().path())
                        {
                            renames.push((
                                old_path.to_string_lossy().to_string(),
                                new_path.to_string_lossy().to_string(),
                            ));
                        }
                    }
                    _ => {}
                }
                true
            },
            None,
            Some(&mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    all_paths.push(path.to_string_lossy().to_string());
                }
                true
            }),
            None,
        )
        .map_err(|e| format!("Failed to iterate diff: {}", e))?;

        let mut file_stats: std::collections::HashMap<String, (u32, u32)> =
            std::collections::HashMap::new();

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

        let result: Vec<ChangedFile> = all_paths
            .into_iter()
            .filter_map(|path| {
                if deleted_paths.contains(&path) {
                    return None;
                }
                let (additions, deletions) = file_stats.remove(&path).unwrap_or((0, 0));
                Some(ChangedFile {
                    path,
                    additions,
                    deletions,
                    is_deleted: false,
                    renamed_to: None,
                })
            })
            .collect();

        Ok((result, renames))
    }
}
