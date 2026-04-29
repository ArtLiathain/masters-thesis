use git2::{Cred, RemoteCallbacks};
use std::{
    env,
    fs::{self, File},
    path::Path,
};

use crate::codescene_client::{label_from_code_health, CodeSceneClient};
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
    extension: String,
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

            let analyser = GitAnalyzer::new(
                repo_clone_path.display().to_string(),
                repo_url.to_string(),
                extension.clone(),
            );

            let max_files_per_commit = 200;
            let max_renames_per_commit = 300;

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
                    client.compute_hub_scores(&repo_name, 0.0).await?;
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
    neo4j_database: String,
    prune: bool,
    threshold: i64,
    max_files_per_commit: usize,
    max_renames_per_commit: usize,
    hub_threshold: f64,
    extension: String,
    output_csv: String,
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = Neo4jClient::new_with_database(&neo4j_uri, &neo4j_database).await?;
    client.init_schema().await?;
    client.delete_all_nodes().await?;

    let analyser = GitAnalyzer::new(repo_path.clone(), "null".to_string(), extension.clone());
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

    client.compute_hub_scores(&repo_name, 0.0).await?;
    println!("Computed hub scores for {}", repo_name);

    let temp_folder = format!("/tmp/{}", repo_name);
    copy_files_by_hub_threshold(
        neo4j_uri,
        neo4j_database,
        hub_threshold,
        temp_folder.clone(),
        extension,
        vec![],
        Some(repo_path.clone()),
    )
    .await?;

    crate::file_metrics_analyser::convert_balanced_metrics(temp_folder.clone(), output_csv)?;

    let _ = fs::remove_dir_all(&temp_folder);

    Ok(commit_count)
}

pub async fn copy_files_by_hub_threshold(
    neo4j_uri: String,
    neo4j_database: String,
    hub_threshold: f64,
    output_dir: String,
    extension: String,
    ignore_repos: Vec<String>,
    local_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Neo4jClient::new_with_database(&neo4j_uri, &neo4j_database).await?;

    let repos_with_files = client
        .get_files_by_hub_threshold(hub_threshold, &extension, &ignore_repos)
        .await?;

    let high_dir = Path::new(&output_dir).join("high");
    let low_dir = Path::new(&output_dir).join("low");

    fs::create_dir_all(&high_dir)?;
    fs::create_dir_all(&low_dir)?;

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

    let mut high_count = 0;
    let mut low_count = 0;

    for repo_data in repos_with_files {
        let repo_name = &repo_data.repo;
        let repo_url = &repo_data.repo_url;
        let is_high_risk = repo_data.is_high_risk;

        if repo_data.files.is_empty() {
            println!("Skipping {} - no files", repo_name);
            continue;
        }

        if let Some(ref local) = local_path {
            println!(
                "Using local repo for {} - {} files (high: {})",
                repo_name,
                repo_data.files.len(),
                is_high_risk
            );
            let source_base = Path::new(local);

            for file in &repo_data.files {
                let source_file = source_base.join(&file.path);
                if !source_file.exists() {
                    println!("  File not found: {}", file.path);
                    continue;
                }

                let dest_filename = format!("{}__{}", repo_name, file.path.replace('/', "_"));

                let target_dir = if file.hub_score >= hub_threshold {
                    &high_dir
                } else {
                    &low_dir
                };

                if file.hub_score >= hub_threshold {
                    high_count += 1;
                } else {
                    low_count += 1;
                }

                let dest_file = target_dir.join(&dest_filename);

                fs::copy(&source_file, &dest_file)
                    .map_err(|e| format!("Failed to copy {}: {}", file.path, e))?;

                println!("  Copied: {}", dest_filename);
            }
        } else {
            if repo_url.is_empty() {
                println!("Skipping {} - no repo URL", repo_name);
                continue;
            }

            println!(
                "Processing {} - {} files (high: {})",
                repo_name,
                repo_data.files.len(),
                is_high_risk
            );

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

                let dest_filename = format!("{}__{}", repo_name, file.path.replace('/', "_"));

                let target_dir = if file.hub_score >= hub_threshold {
                    &high_dir
                } else {
                    &low_dir
                };

                if file.hub_score >= hub_threshold {
                    high_count += 1;
                } else {
                    low_count += 1;
                }

                let dest_file = target_dir.join(&dest_filename);

                fs::copy(&source_file, &dest_file)
                    .map_err(|e| format!("Failed to copy {}: {}", file.path, e))?;

                println!("  Copied: {}", dest_filename);
            }
        }

        println!("Kept in cache: {}", repo_name);
    }

    println!(
        "Done! Files saved to {} (high: {}, low: {})",
        output_dir, high_count, low_count
    );

    Ok(())
}

pub async fn analyze_with_codescene(
    repo_path: String,
    token: String,
    project_id: String,
    threshold: f64,
    file_extensions: String,
    output_csv: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use csv::Writer;
    use rust_code_analysis::get_function_spaces;

    let repo_name = Path::new(&repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("Analyzing repository: {}", repo_name);

    let codescene_client = CodeSceneClient::new("https://api.codescene.io", &token);

    println!("Fetching code health data from CodeScene...");
    let health_map = codescene_client
        .get_file_code_health(&project_id, &repo_name, &file_extensions)
        .await?;

    println!("Found code health data for {} files", health_map.len());

    let mut wtr = Writer::from_path(&output_csv)?;

    let mut high_risk_count = 0;
    let mut low_risk_count = 0;
    let mut processed_count = 0;

    for (file_path, code_health) in &health_map {
        let source_file = Path::new(&repo_path).join(file_path);
        if !source_file.exists() {
            continue;
        }

        let is_high_risk = label_from_code_health(*code_health, threshold);

        let file_as_bytes = match fs::read(&source_file) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("  Warning: Could not read file {}: {}", file_path, e);
                continue;
            }
        };

        let results = match get_function_spaces(
            &rust_code_analysis::LANG::Cpp,
            file_as_bytes,
            &source_file,
            None,
        ) {
            Some(r) => r,
            None => {
                eprintln!("  Warning: Could not analyze file: {}", file_path);
                continue;
            }
        };

        let flattened = crate::file_metrics_analyser::flatten_metrics(
            file_path,
            &results.metrics,
            is_high_risk,
        );

        wtr.serialize(flattened)?;

        if is_high_risk {
            high_risk_count += 1;
        } else {
            low_risk_count += 1;
        }
        processed_count += 1;

        if processed_count % 100 == 0 {
            println!("  Processed {}/{} files", processed_count, health_map.len());
        }
    }

    wtr.flush()?;

    println!(
        "Analyzed {} files: {} high-risk, {} low-risk",
        processed_count, high_risk_count, low_risk_count
    );

    println!("Done! CSV saved to {}", output_csv);

    Ok(())
}
