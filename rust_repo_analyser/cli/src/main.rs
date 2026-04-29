use clap::Parser;

#[derive(Parser)]
#[command(name = "scraper")]
#[command(about = "CLI for scraping JOSS papers and GitHub statistics", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,

    #[arg(long, default_value = "", help = "Neo4j database name (optional)")]
    neo4j_database: String,
}

#[derive(Parser)]
enum Commands {
    Joss(JossArgs),
    Github(GithubArgs),
    Graph(GraphArgs),
    Clone(CloneArgs),
    Verify(VerifyArgs),
    Copy(CopyTopFilesArgs),
    Metrics(MetricsArgs),
    CodeSceneAnalyze(CodeSceneAnalyzeArgs),
    ExportHubScores(ExportHubScoresArgs),
    RecomputeHubScores(RecomputeHubScoresArgs),
}

#[derive(Parser, Debug)]
#[command(about = "Scrape papers from JOSS (Journal of Open Source Software)", long_about = None)]
struct JossArgs {
    #[arg(short, long, default_value = "python")]
    language: String,

    #[arg(short, long, default_value = "joss_papers.json")]
    output: String,
}

#[derive(Parser, Debug)]
#[command(about = "Scrape GitHub statistics from JOSS paper repositories", long_about = None)]
struct GithubArgs {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = "github_stats.json")]
    output: String,

    #[arg(short, long)]
    token: String,
}

#[derive(Parser, Debug)]
#[command(about = "Analyze a local Git repository and save to Neo4j", long_about = None)]
struct GraphArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(short, long)]
    name: String,

    #[arg(long, default_value_t = false)]
    prune: bool,

    #[arg(long, default_value = "5")]
    threshold: i64,

    #[arg(long, default_value = "200")]
    max_files_per_commit: usize,

    #[arg(long, default_value = "300")]
    max_renames_per_commit: usize,

    #[arg(
        long,
        help = "Hub score threshold: files >= threshold go to high/, < threshold go to low/"
    )]
    hub_threshold: f64,

    #[arg(
        long,
        default_value = ".cpp",
        help = "Comma-separated file extensions to filter (e.g., '.cpp,.h,.hpp')"
    )]
    extension: String,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[derive(Parser, Debug)]
#[command(about = "Clone and analyze GitHub repositories from a JSON file", long_about = None)]
struct CloneArgs {
    #[arg(short, long)]
    input: String,

    #[arg(long, default_value = "./repo_cache", hide = true)]
    path: String,

    #[arg(long, default_value = ".cpp")]
    extension: String,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[derive(Parser)]
#[command(about = "Verify and retrieve graph data from Neo4j", long_about = None)]
struct VerifyArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(short, long, default_value = "graph_output.json")]
    output: String,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[derive(Parser, Debug)]
#[command(about = "Copy files from Neo4j repos to local folder", long_about = None)]
struct CopyTopFilesArgs {
    #[arg(
        short,
        long,
        help = "Hub score threshold: files >= threshold go to high/, < threshold go to low/"
    )]
    score_hub_threshold: f64,

    #[arg(short, long, default_value = "data/files")]
    output: String,

    #[arg(short, long, default_value = "cpp")]
    extension: String,

    #[arg(short, long, num_args = 0..)]
    ignore: Vec<String>,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[derive(Parser, Debug)]
#[command(about = "Analyze file metrics from a folder", long_about = None)]
struct MetricsArgs {
    #[arg(short, long)]
    folder: String,

    #[arg(short, long, default_value = "../results/metrics.csv")]
    output: String,
}

#[derive(Parser, Debug)]
#[command(about = "Analyze a local repo using CodeScene for risk labels", long_about = None)]
struct CodeSceneAnalyzeArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(long)]
    token: String,

    #[arg(long)]
    project_id: String,

    #[arg(long, default_value = "9.0")]
    threshold: f64,

    #[arg(
        long,
        default_value = ".cpp",
        help = "Comma-separated file extensions to filter (e.g., '.cpp,.h,.hpp')"
    )]
    extensions: String,

    #[arg(long, default_value = "../results/codescene_metrics.csv")]
    output_csv: String,
}

#[derive(Parser, Debug)]
#[command(about = "Export all hub scores to JSON", long_about = None)]
struct ExportHubScoresArgs {
    #[arg(long, default_value = ".cpp")]
    extension: String,

    #[arg(long, default_value = "../results/hub_scores.json")]
    output: String,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[derive(Parser, Debug)]
#[command(about = "Recompute hub scores for all repos with minimum coupling threshold", long_about = None)]
struct RecomputeHubScoresArgs {
    #[arg(long, help = "Minimum coupling threshold (e.g., 0.3)")]
    min_coupling: f64,

    #[arg(long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Joss(args) => {
            println!("Scraping JOSS papers for language: {}", args.language);
            println!("Output file: {}", args.output);
            repo_scraper::joss_scraper::scrape_joss_papers(args.language, args.output).await?;
        }
        Commands::Github(args) => {
            println!("Scraping GitHub stats from: {}", args.input);
            println!("Output file: {}", args.output);
            repo_scraper::github_stats_scraper::get_github_metrics_from_json(
                args.input,
                args.output,
                &args.token,
            )
            .await?;
        }
        Commands::Graph(args) => {
            println!("Analyzing repository: {}", args.repo);
            println!("Neo4j URI: {}", args.neo4j_uri);
            println!("Prune: {}, threshold: {}", args.prune, args.threshold);
            println!(
                "Max files per commit: {}, max renames per commit: {}",
                args.max_files_per_commit, args.max_renames_per_commit
            );
            println!("Hub threshold: {}", args.hub_threshold);
            println!("Extension: {}", args.extension);

            let output_csv = format!("../results/{}_metrics.csv", args.name);

            repo_analyser::entrypoint::analyze_local_repo(
                args.repo,
                args.name,
                args.neo4j_uri,
                "".to_string(),
                args.prune,
                args.threshold,
                args.max_files_per_commit,
                args.max_renames_per_commit,
                args.hub_threshold,
                args.extension,
                output_csv,
            )
            .await?;
            println!("Successfully saved graph to Neo4j");
        }
        Commands::Clone(args) => {
            println!("Cloning and analyzing repositories from: {}", args.input);
            println!("Clone path: {}", args.path);
            println!("Extension: {}", args.extension);
            println!("Neo4j URI: {}", args.neo4j_uri);

            repo_analyser::entrypoint::analyse_github_repos(
                args.input,
                args.neo4j_uri,
                args.path,
                args.extension,
            )
            .await?;
            println!("Successfully analyzed all repositories");
        }
        Commands::Verify(args) => {
            println!("Verifying graph for repository: {}", args.repo);
            println!("Neo4j URI: {}", cli.neo4j_uri);

            let client = repo_analyser::Neo4jClient::new(&cli.neo4j_uri).await?;
            let graph = client.get_graph(&args.repo).await?;

            let json = serde_json::to_string_pretty(&graph)?;
            std::fs::write(&args.output, json)?;
            println!(
                "Saved graph to {} - {} files, {} edges",
                args.output,
                graph.files.len(),
                graph.edges.len()
            );
        }
        Commands::Copy(args) => {
            println!(
                "Copying files (hub_threshold: {})",
                args.score_hub_threshold
            );
            println!("Output: {}", args.output);
            println!("Extension: {}", args.extension);
            println!("Neo4j URI: {}", cli.neo4j_uri);

            repo_analyser::entrypoint::copy_files_by_hub_threshold(
                cli.neo4j_uri,
                cli.neo4j_database,
                args.score_hub_threshold,
                args.output,
                args.extension,
                args.ignore.clone(),
                None,
            )
            .await?;
            println!("Successfully copied all files");
        }
        Commands::Metrics(args) => {
            println!("Analyzing file metrics from: {}", args.folder);
            repo_analyser::file_metrics_analyser::convert_balanced_metrics(
                args.folder,
                args.output,
            )?;
            println!("Successfully analyzed file metrics");
        }
        Commands::CodeSceneAnalyze(args) => {
            println!("Analyzing repository with CodeScene: {}", args.repo);
            println!("Code health threshold: {}", args.threshold);

            repo_analyser::entrypoint::analyze_with_codescene(
                args.repo,
                args.token,
                args.project_id,
                args.threshold,
                args.extensions,
                args.output_csv,
            )
            .await?;
            println!("Successfully analyzed with CodeScene");
        }
        Commands::ExportHubScores(args) => {
            println!("Exporting hub scores for extension: {}", args.extension);
            println!("Output: {}", args.output);
            println!("Neo4j URI: {}", cli.neo4j_uri);

            let client = repo_analyser::Neo4jClient::new(&cli.neo4j_uri).await?;
            let hub_scores = client.get_all_hub_scores(&args.extension).await?;

            let json = serde_json::to_string_pretty(&hub_scores)?;
            std::fs::write(&args.output, json)?;
            println!("Saved {} hub scores to {}", hub_scores.len(), args.output);
        }
        Commands::RecomputeHubScores(args) => {
            println!(
                "Recomputing hub scores for all repos (min_coupling: {})",
                args.min_coupling
            );
            println!("Neo4j URI: {}", args.neo4j_uri);

            let client = repo_analyser::Neo4jClient::new(&args.neo4j_uri).await?;
            let repos = client.get_all_repo_names().await?;

            println!("Found {} repos to process", repos.len());

            for (i, repo) in repos.iter().enumerate() {
                print!("[{}/{}] Processing {}...", i + 1, repos.len(), repo);
                client.compute_hub_scores(repo, args.min_coupling).await?;
                println!(" done");
            }

            println!("Successfully recomputed hub scores for all repos");
        }
    }

    Ok(())
}
