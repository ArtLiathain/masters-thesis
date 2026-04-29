use git2::{Delta, DiffFindOptions, Repository};
use log::{debug, info};
use std::collections::HashSet;

use crate::file_graph::ChangedFile;
use crate::storage::Neo4jClient;

pub struct GitAnalyzer {
    repo_path: String,
    repo_url: String,
    extensions: Vec<String>,
}

impl GitAnalyzer {
    pub fn new(repo_path: String, repo_url: String, extensions: String) -> Self {
        let extensions: Vec<String> = if extensions.is_empty() {
            vec![]
        } else {
            extensions.split(',').map(|s| s.to_string()).collect()
        };
        Self {
            repo_path,
            repo_url,
            extensions,
        }
    }

    pub async fn analyze(
        &self,
        client: &Neo4jClient,
        repo_name: &str,
        max_files_per_commit: usize,
        max_renames_per_commit: usize,
    ) -> Result<i64, String> {
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
        let mut skipped_commits: Vec<String> = Vec::new();
        let mut total_files_saved = 0i64;
        let mut total_deleted_files_saved = 0i64;
        let mut deleted_files_from_large_commits = 0i64;

        for rev in revwalk {
            let current_commit = rev.map_err(|err| format!("Error unwrapping revwalk:{}", err))?;
            let commit = repo
                .find_commit(current_commit)
                .map_err(|e| format!("Failed to find commit: {}", e))?;
            debug!("Getting changed files");
            let (changed_files, renames) = self.get_changed_files(&repo, &commit)?;

            let commit_hash = commit.id().to_string();

            if changed_files.len() > max_files_per_commit {
                debug!(
                    "Large commit detected: {} ({} files)",
                    commit_hash,
                    changed_files.len()
                );

                // Still process renames if under threshold
                if !renames.is_empty() && renames.len() <= max_renames_per_commit {
                    debug!(
                        "Processing {} renames for large commit {}",
                        renames.len(),
                        commit_hash
                    );
                    for (old_path, new_path) in &renames {
                        match client.rename_file_node(repo_name, old_path, new_path).await {
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
                }

                // Always process deleted files even in large commits
                let deleted_files: Vec<&ChangedFile> =
                    changed_files.iter().filter(|f| f.is_deleted).collect();
                if !deleted_files.is_empty() {
                    debug!(
                        "Processing {} deleted files for large commit {}",
                        deleted_files.len(),
                        commit_hash
                    );
                    deleted_files_from_large_commits += deleted_files.len() as i64;
                    for file in deleted_files {
                        if let Err(e) = client
                            .save_file_node(
                                repo_name,
                                &file.path,
                                file.additions as i64,
                                file.deletions as i64,
                                Some(&commit_hash),
                            )
                            .await
                        {
                            log::warn!("Failed to save deleted file node {}: {}", file.path, e);
                        }
                    }
                }

                skipped_commits.push(commit_hash.clone());
                info!(
                    "Skipped large commit {} ({} files, {} renames)",
                    commit_hash,
                    changed_files.len(),
                    renames.len()
                );
            } else if !changed_files.is_empty() || !renames.is_empty() {
                debug!("Save to Neo4j");
                let (files, deleted) = self
                    .save_to_neo4j(
                        client,
                        repo_name,
                        &changed_files,
                        &renames,
                        Some(&commit_hash),
                    )
                    .await?;
                total_files_saved += files;
                total_deleted_files_saved += deleted;
                debug!("Saved to Neo4j");
            }

            commit_count += 1;
            if commit_count % 1 == 0 {
                info!("Processed {} commits", commit_count);
                info!("Processed {} commit", commit.id());
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

        if !skipped_commits.is_empty() {
            info!(
                "Skipped {} large commits out of {} total ({}%)",
                skipped_commits.len(),
                commit_count,
                (skipped_commits.len() as f64 / commit_count as f64 * 100.0) as i64
            );
        }

        info!(
            "Git analysis summary for {}: {} commits analyzed, {} skipped (large), {} files saved ({} deleted from small commits, {} deleted from large commits)",
            repo_name,
            commit_count,
            skipped_commits.len(),
            total_files_saved,
            total_deleted_files_saved,
            deleted_files_from_large_commits
        );

        Ok(commit_count)
    }

    async fn save_to_neo4j(
        &self,
        client: &Neo4jClient,
        repo_name: &str,
        changed_files: &[ChangedFile],
        renames: &[(String, String)],
        commit_hash: Option<&str>,
    ) -> Result<(i64, i64), String> {
        let mut files_saved = 0i64;
        let mut deleted_files_saved = 0i64;

        debug!("Renaming files Started");
        for (old_path, new_path) in renames {
            match client.rename_file_node(repo_name, old_path, new_path).await {
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
        debug!("Renaming files Ended");

        let mut unique_files: Vec<&str> = changed_files.iter().map(|f| f.path.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        debug!("Saving file nodes");
        for file in changed_files {
            let deleted_at_commit = if file.is_deleted { commit_hash } else { None };
            files_saved += 1;
            if file.is_deleted {
                deleted_files_saved += 1;
            }
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
        debug!("file nodes saved");

        debug!("Relationships saving");

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
        debug!("Relationships saved");

        Ok((files_saved, deleted_files_saved))
    }

    fn get_changed_files(
        &self,
        repo: &Repository,
        commit: &git2::Commit,
    ) -> Result<(Vec<ChangedFile>, Vec<(String, String)>), String> {
        let tree = commit
            .tree()
            .map_err(|e| format!("Failed to get commit tree: {}", e))?;

        let mut renames: Vec<(String, String)> = Vec::new();
        let mut deleted_paths: HashSet<String> = HashSet::new();
        let mut all_paths: Vec<String> = Vec::new();

        if commit.parent_count() == 0 {
            let mut diff = repo
                .diff_tree_to_tree(None, Some(&tree), None)
                .map_err(|e| format!("Failed to get diff: {}", e))?;

            diff.find_similar(Some(&mut DiffFindOptions::new().renames(true)))
                .map_err(|e| format!("Failed to find similar: {}", e))?;

            let mut file_stats: std::collections::HashMap<String, (u32, u32)> =
                std::collections::HashMap::new();
            Self::collect_diff_entries(
                &diff,
                &mut renames,
                &mut deleted_paths,
                &mut all_paths,
                &mut file_stats,
                &self.extensions,
            )?;

            let mut result: Vec<ChangedFile> = all_paths
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .filter_map(|path| {
                    let (additions, deletions) = file_stats.remove(path).unwrap_or((0, 0));
                    Some(ChangedFile {
                        path: path.clone(),
                        additions,
                        deletions,
                        is_deleted: false,
                        renamed_to: None,
                    })
                })
                .collect();

            for path in deleted_paths {
                result.push(ChangedFile {
                    path,
                    additions: 0,
                    deletions: 0,
                    is_deleted: true,
                    renamed_to: None,
                });
            }

            return Ok((result, renames));
        } else if commit.parent_count() == 1 {
            let parent = commit
                .parent(0)
                .map_err(|e| format!("Failed to get parent commit: {}", e))?;
            let parent_tree = parent
                .tree()
                .map_err(|e| format!("Failed to get parent tree: {}", e))?;

            let mut diff = repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
                .map_err(|e| format!("Failed to get diff: {}", e))?;

            diff.find_similar(Some(&mut DiffFindOptions::new().renames(true)))
                .map_err(|e| format!("Failed to find similar: {}", e))?;

            let mut file_stats: std::collections::HashMap<String, (u32, u32)> =
                std::collections::HashMap::new();
            Self::collect_diff_entries(
                &diff,
                &mut renames,
                &mut deleted_paths,
                &mut all_paths,
                &mut file_stats,
                &self.extensions,
            )?;

            let mut result: Vec<ChangedFile> = all_paths
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .filter_map(|path| {
                    let (additions, deletions) = file_stats.remove(path).unwrap_or((0, 0));
                    Some(ChangedFile {
                        path: path.clone(),
                        additions,
                        deletions,
                        is_deleted: false,
                        renamed_to: None,
                    })
                })
                .collect();

            for path in deleted_paths {
                result.push(ChangedFile {
                    path,
                    additions: 0,
                    deletions: 0,
                    is_deleted: true,
                    renamed_to: None,
                });
            }

            return Ok((result, renames));
        } else {
            let mut file_stats: std::collections::HashMap<String, (u32, u32)> =
                std::collections::HashMap::new();

            for i in 0..commit.parent_count() {
                let parent = commit
                    .parent(i)
                    .map_err(|e| format!("Failed to get parent commit: {}", e))?;
                let parent_tree = parent
                    .tree()
                    .map_err(|e| format!("Failed to get parent tree: {}", e))?;

                let mut diff = repo
                    .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
                    .map_err(|e| format!("Failed to get diff: {}", e))?;

                diff.find_similar(Some(&mut DiffFindOptions::new().renames(true)))
                    .map_err(|e| format!("Failed to find similar: {}", e))?;

                Self::collect_diff_entries(
                    &diff,
                    &mut renames,
                    &mut deleted_paths,
                    &mut all_paths,
                    &mut file_stats,
                    &self.extensions,
                )?;
            }

            let mut result: Vec<ChangedFile> = all_paths
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .filter_map(|path| {
                    let (additions, deletions) = file_stats.remove(path).unwrap_or((0, 0));
                    Some(ChangedFile {
                        path: path.clone(),
                        additions,
                        deletions,
                        is_deleted: false,
                        renamed_to: None,
                    })
                })
                .collect();

            for path in deleted_paths {
                result.push(ChangedFile {
                    path,
                    additions: 0,
                    deletions: 0,
                    is_deleted: true,
                    renamed_to: None,
                });
            }

            return Ok((result, renames));
        }
    }

    fn collect_diff_entries(
        diff: &git2::Diff,
        renames: &mut Vec<(String, String)>,
        deleted_paths: &mut HashSet<String>,
        all_paths: &mut Vec<String>,
        file_stats: &mut std::collections::HashMap<String, (u32, u32)>,
        extensions: &[String],
    ) -> Result<(), String> {
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
                            deleted_paths.remove(&old_path.to_string_lossy().to_string());
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
                    let path_str = path.to_string_lossy().to_string();
                    if extensions.is_empty() || extensions.iter().any(|ext| path_str.ends_with(ext))
                    {
                        all_paths.push(path_str);
                    }
                }
                true
            }),
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path() {
                    let path_str = path.to_string_lossy().to_string();
                    if extensions.is_empty() || extensions.iter().any(|ext| path_str.ends_with(ext))
                    {
                        let stats = file_stats.entry(path_str).or_insert((0, 0));
                        if line.origin() == '+' {
                            stats.0 += 1;
                        } else if line.origin() == '-' {
                            stats.1 += 1;
                        }
                    }
                }
                true
            }),
        )
        .map_err(|e| format!("Failed to iterate diff: {}", e))?;

        Ok(())
    }
}
