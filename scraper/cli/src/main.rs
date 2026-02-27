use clap::Parser;

#[derive(Parser)]
#[command(name = "scraper")]
#[command(about = "CLI for scraping JOSS papers and GitHub statistics", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    Joss(JossArgs),
    Github(GithubArgs),
    Graph(GraphArgs),
    Clone(CloneArgs),
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
#[command(about = "Analyze a local Git repository to build a file dependency graph", long_about = None)]
struct GraphArgs {
    #[arg(short, long)]
    repo: String,

    #[arg(short, long, default_value = "file_graph.json")]
    output: String,
}

#[derive(Parser, Debug)]
#[command(about = "Clone and analyze GitHub repositories from a JSON file", long_about = None)]
struct CloneArgs {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = "repo_graphs.json")]
    output: String,

    #[arg(short, long, default_value = "/tmp/repos")]
    path: String,
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
            println!("Output file: {}", args.output);
            
            let mut analyzer = repo_analyser::GitAnalyzer::new(args.repo);
            let graph = analyzer.analyze()?;
            
            repo_analyser::save_graph_to_json(&graph, &args.output)?;
            println!("Successfully saved graph to {}", args.output);
        }
        Commands::Clone(args) => {
            println!("Cloning and analyzing repositories from: {}", args.input);
            println!("Output file: {}", args.output);
            println!("Clone path: {}", args.path);
            
            repo_analyser::entrypoint::analyse_github_repos(
                args.input,
                args.output,
            ).await?;
            println!("Successfully analyzed all repositories");
        }
    }

    Ok(())
}
