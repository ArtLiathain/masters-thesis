import json
import pandas as pd
import os

INPUT_FILE = "/home/art/Development/masters-thesis/results/htool.json"
OUTPUT_DIR = "/home/art/Development/masters-thesis/results"

COUPLING_THRESHOLD=0.3
TARGET_COMMIT_THRESHOLD=0.2
EDGE_WEIGHT_THRESHOLD=0.05
TARGET_COMMIT_MAXIMUM=10
EDGE_WEIGHT_MAXIMUM=3

def load_data(filepath):
    with open(filepath, "r") as f:
        return json.load(f)


def compute_coupling_ratio(weight: int, target_commits: int) -> float:
    """Compute coupling ratio: co-changes / min(commits_A, commits_B)

    We use weight as co-changes and target_commits as a proxy for min commits.
    """
    if target_commits == 0:
        return 0.0
    return weight / target_commits

def calculate_valid_partners(edges, commit_count):
    partner_count = len(edges)

    # Compute average coupling ratio
    if partner_count <= 0:
        return 0,0,0
    commit_threshold = min(int(TARGET_COMMIT_THRESHOLD * commit_count), TARGET_COMMIT_MAXIMUM)
    weight_threshold= min(int(EDGE_WEIGHT_THRESHOLD * commit_count), EDGE_WEIGHT_MAXIMUM)

    filtered_edges = [edge for edge in edges if edge["target_commits"] >= commit_threshold or edge["weight"] >= weight_threshold]
    coupling_ratios = [
        compute_coupling_ratio(e["weight"], e["target_commits"]) for e in filtered_edges
    ]
    filtered_coupling_ratios = [ratio for ratio in coupling_ratios if ratio > COUPLING_THRESHOLD]
    filtered_partner_count = len(filtered_coupling_ratios)
    if filtered_partner_count <= 0:
        return 0,0,0
    avg_coupling = sum(filtered_coupling_ratios) / len(filtered_coupling_ratios)
    max_coupling = max(filtered_coupling_ratios)
    if avg_coupling > 0.9:
        print(filtered_edges)


    return avg_coupling, max_coupling, filtered_partner_count


def compute_hub_scores(nodes, commit_count):
    """Compute hub scores for each file in an era.

    Hub Score = partner_count * avg_coupling_ratio
    """
    if not nodes:
        return []

    results = []
    for node in nodes:
        path = node.get("path", "")
        commit_count = node.get("commit_count", 0)
        additions = node.get("additions", 0)
        deletions = node.get("deletions", 0)
        churn = additions + deletions
        edges = node.get("edges", [])

        # Coupling metrics
        avg_coupling, max_coupling, partner_count = calculate_valid_partners(edges, commit_count)
        # Hub score: partner count * average coupling
        hub_score = partner_count * avg_coupling
        results.append(
            {
                "path": path,
                "commit_count": commit_count,
                "additions": additions,
                "deletions": deletions,
                "churn": churn,
                "partner_count": partner_count,
                "avg_coupling": round(avg_coupling, 4),
                "max_coupling": round(max_coupling, 4),
                "hub_score": round(hub_score, 4),
            }
        )

    return results


def analyze_repo(repo_data):
    """Analyze a single repo and compute hub scores."""
    repo_name = repo_data.get("repo", "unknown")
    total_commits = repo_data.get("total_commits_analyzed", 0)
    hub_scores = compute_hub_scores(repo_data.get("node_map", []), total_commits)

    df = pd.DataFrame(hub_scores)
    print(hub_scores)
    df = df.sort_values("hub_score", ascending=False)


    return {
        "repo": repo_name,
        "total_commits": total_commits,
        "hub_scores": df.to_dict("records"),
        "top_files": df.head(10).to_dict("records"),
    }


def display_hub_scores(repo_result):
    """Display hub scores in a readable format."""
    print(f"\n{'=' * 100}")
    print(f"HUB SCORE ANALYSIS: {repo_result['repo']}")
    print(f"{'=' * 100}")
    print(f"Total commits: {repo_result['total_commits']}")
    print(
        f"{'Rank':<5} {'File':<50} {'Partners':<10} {'Avg Coup':<10} {'Hub Score':<10}"
    )
    print(f"{'-' * 90}")

    for rank, file_data in enumerate(repo_result["top_files"], 1):
        filename = os.path.basename(file_data["path"])
        if len(filename) > 45:
            filename = f"...{filename[-42:]}"
        print(
            f"{rank:<5} {filename:<50} {file_data['partner_count']:<10} {file_data['avg_coupling']:<10.4f} {file_data['hub_score']:<10.4f}"
        )

    print(f"\n{'=' * 100}\n")


def main():
    print("Loading data...")
    data = load_data(INPUT_FILE)

    if isinstance(data, dict):
        data = [data]

    print(f"Processing {len(data)} repo(s)...")

    all_results = []
    for idx, repo_data in enumerate(data, 1):
        repo_label = f"repo_{idx}"
        print(f"\nProcessing {repo_label}...")

        result = analyze_repo(repo_data)
        all_results.append(result)

        display_hub_scores(result)

    # Save detailed results
    output_file = os.path.join(OUTPUT_DIR, "hub_score_analysis.json")
    with open(output_file, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\nSaved detailed results to: {output_file}")

    # Create summary CSV
    summary_rows = []
    for result in all_results:
        for file_data in result["hub_scores"]:
            summary_rows.append(
                {
                    "repo": result["repo"],
                    "path": file_data["path"],
                    "commit_count": file_data["commit_count"],
                    "churn": file_data["churn"],
                    "partner_count": file_data["partner_count"],
                    "avg_coupling": file_data["avg_coupling"],
                    "hub_score": file_data["hub_score"],
                }
            )

    summary_df = pd.DataFrame(summary_rows)
    summary_file = os.path.join(OUTPUT_DIR, "hub_score_summary.csv")
    summary_df.to_csv(summary_file, index=False)
    print(f"Saved summary CSV to: {summary_file}")

    # Print top problematic files overall
    print("\n" + "=" * 80)
    print("TOP 20 PROBLEMATIC FILES (by hub score)")
    print("=" * 80)
    top_overall = summary_df.nlargest(20, "hub_score")
    print(
        top_overall[
            ["repo","path", "partner_count", "avg_coupling", "hub_score"]
        ].to_string(index=False)
    )


if __name__ == "__main__":
    main()
