use git2::Repository;
use std::{
    fs::{self, File},
    path::Path,
};

use crate::GitAnalyzer;

fn extract_repo_name(url: &str) -> Option<String> {
    url.split("github.com/")
        .nth(1)?
        .split('/')
        .last()
        .map(|s| s.trim_end_matches(".git").to_string())
}

pub async fn analyse_github_repos(
    json_file: String,
    output_file_location: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(json_file)?;
    let repos: Vec<serde_json::Value> = serde_json::from_reader(file)?;

    let path = Path::new("/tmp/repoToAnalyse");
    for repo in repos {
        let repo_url = repo["repo_url"].as_str().unwrap_or("");
        if repo_url.contains("github.com") {
            println!("Processing: {}", repo_url);

            fs::remove_dir_all(path).ok();
            Repository::clone(repo_url, path).map_err(|e| format!("Clone failed: {}", e))?;

            let mut analyser = GitAnalyzer::new(path.display().to_string());

            match analyser.analyze() {
                Ok(mut file_graph) => {
                    let repo_name =
                        extract_repo_name(repo_url).unwrap_or_else(|| "unknown".to_string());

                    let out_file = File::create(
                        output_file_location.clone().to_string()
                            + "/"
                            + &repo_name.clone().to_string()
                            + ".json",
                    )?;
                    file_graph.repo = repo_name;
                    serde_json::to_writer_pretty(out_file, &file_graph)?;
                    println!("Wrote repos to {}", output_file_location);
                }
                Err(err) => println!("Error analysing {} repo : {}", repo_url, err),
            }
        }
    }

    Ok(())
}
