use git2::{Cred, RemoteCallbacks};
use std::{
    env,
    fs::{self, File},
    path::Path,
};

use crate::git_analyzer::GitAnalyzer;
use crate::storage::Neo4jClient;

fn extract_repo_name(url: &str) -> Option<String> {
    let mut name_url = String::from(url);
    if name_url.chars().last() == Some('/') {
        name_url.pop();
    }
    name_url
        .split("github.com/")
        .nth(1)?
        .split('/')
        .last()
        .map(|s| s.trim_end_matches(".git").to_string())
}

pub async fn analyse_github_repos(
    json_file: String,
    neo4j_uri: String,
    _folder_path: String,
    ignore_repos: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;
    client.init_schema().await?;

    let file = File::open(json_file)?;
    let repos: Vec<serde_json::Value> = serde_json::from_reader(file)?;
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    let cache_base = Path::new("./repo_cache");
    fs::create_dir_all(cache_base)?;

    for repo in repos {
        let repo_url = repo["repo_url"].as_str().unwrap_or("");
        if repo_url.contains("github.com") {
            let repo_name = extract_repo_name(repo_url).unwrap_or_else(|| "unknown".to_string());
            println!("{}", repo_name);

            if ignore_repos.iter().any(|r| repo_name.contains(r)) {
                println!("Skipping {} - in ignore list", repo_name);
                continue;
            }

            println!("Processing: {}", repo_url);

            let repo_clone_path = cache_base.join(&repo_name);

            if repo_clone_path.exists() {
                println!("Using cached repo: {}", repo_name);
            } else {
                println!("Cloning {}...", repo_name);
                builder
                    .clone(repo_url, &repo_clone_path)
                    .map_err(|e| format!("Clone failed: {}", e))?;
            }

            let analyser =
                GitAnalyzer::new(repo_clone_path.display().to_string(), repo_url.to_string());

            let max_files_per_commit = 100;
            let max_renames_per_commit = 50;

            match analyser
                .analyze(
                    &client,
                    &repo_name,
                    max_files_per_commit,
                    max_renames_per_commit,
                )
                .await
            {
                Ok(commit_count) => {
                    client.compute_hub_scores(&repo_name).await?;
                    println!(
                        "Saved {} to Neo4j with {} commits analyzed",
                        repo_name, commit_count
                    );
                }
                Err(err) => println!("Error analysing {} repo : {}", repo_url, err),
            }
        }
    }

    Ok(())
}

pub async fn analyze_local_repo(
    repo_path: String,
    repo_name: String,
    neo4j_uri: String,
    prune: bool,
    threshold: i64,
    max_files_per_commit: usize,
    max_renames_per_commit: usize,
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;
    client.init_schema().await?;

    let analyser = GitAnalyzer::new(repo_path, "null".to_string());
    let commit_count = analyser
        .analyze(
            &client,
            &repo_name,
            max_files_per_commit,
            max_renames_per_commit,
        )
        .await?;

    println!(
        "Saved {} to Neo4j with {} commits analyzed",
        repo_name, commit_count
    );

    if prune {
        let deleted = client.remove_low_importance_connections(threshold).await?;
        println!("Pruned {} edges with weight < {}", deleted, threshold);
    }

    client.compute_hub_scores(&repo_name).await?;
    println!("Computed hub scores for {}", repo_name);

    Ok(commit_count)
}

pub async fn copy_files(
    neo4j_uri: String,
    limit: i64,
    output_dir: String,
    extension: String,
    ignore_repos: Vec<String>,
    risk: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;

    let repos_with_files = match risk {
        "high" => client.get_top_files_grouped(limit, &extension, &ignore_repos).await?,
        "low" => client.get_low_risk_files(limit, &extension, &ignore_repos).await?,
        _ => return Err("Risk must be 'high' or 'low'".into()),
    };

    let output_path = Path::new(&output_dir);
    fs::create_dir_all(output_path)?;

    let cache_base = Path::new("./repo_cache");
    fs::create_dir_all(cache_base)?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    for repo_data in repos_with_files {
        let repo_name = &repo_data.repo;
        let repo_url = &repo_data.repo_url;

        if repo_url.is_empty() {
            println!("Skipping {} - no repo URL", repo_name);
            continue;
        }

        if repo_data.files.is_empty() {
            println!("Skipping {} - no files", repo_name);
            continue;
        }

        println!("Processing {} - {} files", repo_name, repo_data.files.len());

        let repo_clone_path = cache_base.join(repo_name);

        if !repo_clone_path.exists() {
            println!("Cloning {}...", repo_name);
            builder
                .clone(repo_url, &repo_clone_path)
                .map_err(|e| format!("Failed to clone {}: {}", repo_url, e))?;
        } else {
            println!("Using cached repo: {}", repo_name);
        }

        for file in &repo_data.files {
            let source_file = repo_clone_path.join(&file.path);
            if !source_file.exists() {
                println!("  File not found: {}", file.path);
                continue;
            }

            let filename = Path::new(&file.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.clone());

            let dest_filename = format!("{}__{}", repo_name, filename);
            let dest_file = output_path.join(&dest_filename);

            fs::copy(&source_file, &dest_file)
                .map_err(|e| format!("Failed to copy {}: {}", file.path, e))?;

            println!("  Copied: {}", dest_filename);
        }

        println!("Kept in cache: {}", repo_name);
    }

    println!("Done! Files saved to {}", output_dir);

    Ok(())
}
