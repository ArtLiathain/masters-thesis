use git2::Repository;
use std::{
    fs::{self, File},
    path::Path,
};

use crate::GitAnalyzer;

pub async fn analyse_github_repos(
    json_file: String,
    output_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(json_file)?;
    let repos: Vec<serde_json::Value> = serde_json::from_reader(file)?;

    let mut extended_stats = Vec::new();

    let path = Path::new("/tmp/repoToAnalyse");
    for repo in repos {
        let repo_url = repo["repo_url"].as_str().unwrap_or("");
        if repo_url.contains("github.com") {
            println!("Processing: {}", repo_url);

            fs::remove_dir_all(path).ok();
            Repository::clone(repo_url, path).map_err(|e| format!("Clone failed: {}", e))?;

            let mut analyser = GitAnalyzer::new(path.display().to_string());

            match analyser.analyze() {
                Ok(file_graph) => {
                    extended_stats.push(file_graph);
                }
                Err(err) => println!("Error analysing {} repo : {}", repo_url, err),
            }
        }
    }

    let out_file = File::create(output_file)?;
    serde_json::to_writer_pretty(out_file, &extended_stats)?;
    Ok(())
}
