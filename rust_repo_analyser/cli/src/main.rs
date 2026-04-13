use clap::Parser;

#[derive(Parser)]
#[command(name = "scraper")]
#[command(about = "CLI for scraping JOSS papers and GitHub statistics", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "bolt://localhost:7687")]
    neo4j_uri: String,
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
    SonarAnalyze(SonarAnalyzeArgs),
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

    #[arg(long, default_value = "true")]
    prune: bool,

    #[arg(long, default_value = "10")]
    threshold: i64,

    #[arg(long, default_value = "100")]
    max_files_per_commit: usize,

    #[arg(long, default_value = "50")]
    max_renames_per_commit: usize,
}

#[derive(Parser, Debug)]
#[command(about = "Clone and analyze GitHub repositories from a JSON file", long_about = None)]
struct CloneArgs {
    #[arg(short, long)]
    input: String,

    #[arg(long, default_value = "./repo_cache", hide = true)]
    path: String,

    #[arg(short, long, num_args = 0.., default_value = "")]
    ignore: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(about = "Verify and retrieve graph data from Neo4j", long_about = None)]
struct VerifyArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(short, long, default_value = "graph_output.json")]
    output: String,
}

#[derive(Parser, Debug)]
#[command(about = "Copy files from Neo4j repos to local folder", long_about = None)]
struct CopyTopFilesArgs {
    #[arg(short, long)]
    risk: String,

    #[arg(short, long, default_value = "200")]
    limit: i64,

    #[arg(short, long, default_value = "data/files")]
    output: String,

    #[arg(short, long, default_value = "cpp")]
    extension: String,

    #[arg(short, long, num_args = 0.., default_value = "")]
    ignore: Vec<String>,
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
#[command(about = "Analyze a local repo using SonarQube for risk labels", long_about = None)]
struct SonarAnalyzeArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(long, default_value = "http://localhost:9000")]
    sonar_url: String,

    #[arg(long, default_value = "admin")]
    username: String,

    #[arg(long, default_value = "admin")]
    token: String,

    #[arg(long, default_value = "189")]
    td_threshold: i64,

    #[arg(long, default_value = "data/sonar_files")]
    output_folder: String,

    #[arg(long, default_value = "data/sonar_metrics.csv")]
    output_csv: String,
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
            println!("Neo4j URI: {}", cli.neo4j_uri);
            println!("Prune: {}, threshold: {}", args.prune, args.threshold);
            println!(
                "Max files per commit: {}, max renames per commit: {}",
                args.max_files_per_commit, args.max_renames_per_commit
            );

            repo_analyser::entrypoint::analyze_local_repo(
                args.repo,
                args.name,
                cli.neo4j_uri,
                args.prune,
                args.threshold,
                args.max_files_per_commit,
                args.max_renames_per_commit,
            )
            .await?;
            println!("Successfully saved graph to Neo4j");
        }
        Commands::Clone(args) => {
            println!("Cloning and analyzing repositories from: {}", args.input);
            println!("Clone path: {}", args.path);
            println!("Neo4j URI: {}", cli.neo4j_uri);

            repo_analyser::entrypoint::analyse_github_repos(
                args.input,
                cli.neo4j_uri,
                args.path,
                args.ignore,
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
            println!("Copying {} files (risk: {})", args.limit, args.risk);
            println!("Output: {}", args.output);
            println!("Extension: {}", args.extension);
            println!("Neo4j URI: {}", cli.neo4j_uri);

            repo_analyser::entrypoint::copy_files(
                cli.neo4j_uri,
                args.limit,
                args.output,
                args.extension,
                args.ignore.clone(),
                &args.risk,
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
        Commands::SonarAnalyze(args) => {
            println!("Analyzing repository with SonarQube: {}", args.repo);
            println!("SonarQube URL: {}", args.sonar_url);
            println!("TD threshold: {} minutes", args.td_threshold);

            repo_analyser::entrypoint::analyze_with_sonar(
                args.repo,
                args.sonar_url,
                args.username,
                args.token,
                args.td_threshold,
                args.output_folder,
                args.output_csv,
            )
            .await?;
            println!("Successfully analyzed with SonarQube");
        }
    }

    Ok(())
}
