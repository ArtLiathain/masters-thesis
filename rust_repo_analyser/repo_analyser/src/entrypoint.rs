use git2::{Cred, RemoteCallbacks};
use std::{
    env,
    fs::{self, File},
    path::Path,
};

use crate::git_analyzer::GitAnalyzer;
use crate::storage::Neo4jClient;

fn extract_repo_name(url: &str) -> Option<String> {
    url.split("github.com/")
        .nth(1)?
        .split('/')
        .last()
        .map(|s| s.trim_end_matches(".git").to_string())
}

pub async fn analyse_github_repos(
    json_file: String,
    neo4j_uri: String,
    folder_path: String,
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

    let path = Path::new(&folder_path);
    for repo in repos {
        let repo_url = repo["repo_url"].as_str().unwrap_or("");
        if repo_url.contains("github.com") {
            println!("Processing: {}", repo_url);

            fs::remove_dir_all(path).ok();
            builder
                .clone(repo_url, path)
                .map_err(|e| format!("Clone failed: {}", e))?;

            let repo_name = extract_repo_name(repo_url).unwrap_or_else(|| "unknown".to_string());

            let analyser = GitAnalyzer::new(path.display().to_string(), repo_url.to_string());

            match analyser.analyze(&client, &repo_name).await {
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
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;
    client.init_schema().await?;

    let analyser = GitAnalyzer::new(repo_path, "null".to_string());
    let commit_count = analyser.analyze(&client, &repo_name).await?;

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

pub async fn copy_top_files(
    neo4j_uri: String,
    limit: i64,
    output_dir: String,
    clone_path: String,
    extension: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;

    let repos_with_files = client.get_top_files_grouped(limit, &extension).await?;
    println!("{:?}", repos_with_files);

    let output_path = Path::new(&output_dir);
    fs::create_dir_all(output_path)?;

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

    let clone_base = Path::new(&clone_path);

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

        let repo_clone_path = clone_base.join(repo_name);

        if repo_clone_path.exists() {
            fs::remove_dir_all(&repo_clone_path)?;
        }

        builder
            .clone(repo_url, &repo_clone_path)
            .map_err(|e| format!("Failed to clone {}: {}", repo_url, e))?;

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

        fs::remove_dir_all(&repo_clone_path).ok();
        println!("Cleaned up clone for {}", repo_name);
    }

    println!("Done! Files saved to {}", output_dir);

    Ok(())
}
