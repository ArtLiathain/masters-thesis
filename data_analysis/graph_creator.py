import json
import os
import pandas as pd

OUTPUT_DIR = "/home/art/Development/masters-thesis/results"

COMMIT_PERCENTAGE_THRESHOLD = 5
CODE_CHURN_PERCENTAGE_THRESHOLD = 1


def load_data(filepath):
    with open(filepath, "r") as f:
        return json.load(f)


def identify_tech_debt_for_era(era_data, total_commits):
    nodes_df = pd.DataFrame(era_data["nodes"])

    if nodes_df.empty:
        return pd.DataFrame()

    nodes_df["code_churn"] = nodes_df["additions"].fillna(0) + nodes_df[
        "deletions"
    ].fillna(0)
    total_era_churn = nodes_df["code_churn"].sum()

    if total_era_churn == 0:
        return pd.DataFrame()

    nodes_df["commit_percentage"] = (nodes_df["commit_count"] / total_commits) * 100
    nodes_df["code_churn_percentage"] = (nodes_df["code_churn"] / total_era_churn) * 100

    high_debt = nodes_df[
        (nodes_df["commit_percentage"] >= COMMIT_PERCENTAGE_THRESHOLD)
        | (nodes_df["code_churn_percentage"] >= CODE_CHURN_PERCENTAGE_THRESHOLD)
    ].copy()

    def get_reasons(row):
        reasons = []
        if row["commit_percentage"] >= COMMIT_PERCENTAGE_THRESHOLD:
            reasons.append(f"high_commits ({row['commit_percentage']:.1f}%)")
        if row["code_churn_percentage"] >= CODE_CHURN_PERCENTAGE_THRESHOLD:
            reasons.append(f"high_churn ({row['code_churn_percentage']:.1f}%)")
        return reasons

    high_debt["reasons"] = high_debt.apply(get_reasons, axis=1)

    return high_debt.sort_values(by="code_churn", ascending=False)


def identify_tech_debt(repo_data, commit_pct_threshold, churn_pct_threshold):
    global COMMIT_PERCENTAGE_THRESHOLD, CODE_CHURN_PERCENTAGE_THRESHOLD
    COMMIT_PERCENTAGE_THRESHOLD = commit_pct_threshold
    CODE_CHURN_PERCENTAGE_THRESHOLD = churn_pct_threshold

    total_commits = repo_data["total_commits_analyzed"]
    eras = repo_data.get("eras", [])

    era_results = []
    for era in eras:
        era_index = era["era_index"]
        commits_in_era = era["commits_in_era"]
        reset_commit = era.get("reset_commit")

        high_debt_df = identify_tech_debt_for_era(era, commits_in_era)

        if not high_debt_df.empty:
            era_results.append(
                {
                    "era_index": era_index,
                    "commits_in_era": commits_in_era,
                    "reset_commit": reset_commit,
                    "high_debt_df": high_debt_df,
                }
            )

    return era_results


def display_tech_debt_per_era(repo_data, era_results):
    repo_name = repo_data["repo"]
    total_commits = repo_data["total_commits_analyzed"]

    print(f"\n{'=' * 85}")
    print(f"TECH DEBT REPORT: {repo_name}")
    print(f"{'=' * 85}")
    print(f"Total commits analyzed: {total_commits}")
    print(
        f"Thresholds: >= {COMMIT_PERCENTAGE_THRESHOLD}% commits, >= {CODE_CHURN_PERCENTAGE_THRESHOLD}% of total churn"
    )

    for era_result in era_results:
        era_idx = era_result["era_index"]
        commits = era_result["commits_in_era"]
        reset = era_result["reset_commit"]
        high_debt_df = era_result["high_debt_df"]

        print(f"\n--- Era {era_idx} ({commits} commits) ---")
        if reset:
            print(f"Reset at commit: {reset[:8]}...")
        print(f"High tech debt files found: {len(high_debt_df)}")
        print(f"{'-' * 85}")
        print(f"{'File':<45} {'Commits%':<10} {'Churn%':<10} {'Churn':<10} {'Reasons'}")
        print(f"{'-' * 85}")

        for _, row in high_debt_df.iterrows():
            filename = os.path.basename(row["path"])
            if len(filename) > 40:
                filename = f"...{filename[-37:]}"
            reasons = ", ".join(row["reasons"])
            print(
                f"{filename:<45} {row['commit_percentage']:>7.1f}%  {row['code_churn_percentage']:>7.1f}%  {row['code_churn']:>8.0f}  {reasons}"
            )

    print(f"\n{'=' * 85}\n")


def main():
    input_file = os.path.join(OUTPUT_DIR, "testing2.json")
    data = load_data(input_file)

    all_tech_debt = []

    if isinstance(data, dict):
        data = [data]

    for idx, repo_data in enumerate(data, 1):
        repo_label = f"repo_{idx}"
        print(f"\nProcessing {repo_label}...")

        era_results = identify_tech_debt(
            repo_data, COMMIT_PERCENTAGE_THRESHOLD, CODE_CHURN_PERCENTAGE_THRESHOLD
        )

        display_tech_debt_per_era(repo_data, era_results)

        era_debt_records = []
        for era_result in era_results:
            era_df = era_result["high_debt_df"]
            for _, row in era_df.iterrows():
                era_debt_records.append(
                    {
                        "era_index": era_result["era_index"],
                        "era_commits": era_result["commits_in_era"],
                        "reset_commit": era_result["reset_commit"],
                        "path": row["path"],
                        "commit_count": row["commit_count"],
                        "commit_percentage": row["commit_percentage"],
                        "additions": row["additions"],
                        "deletions": row["deletions"],
                        "code_churn": row["code_churn"],
                        "code_churn_percentage": row["code_churn_percentage"],
                        "reasons": row["reasons"],
                    }
                )

        result = {
            "repo_label": repo_label,
            "repo_name": repo_data["repo"],
            "total_commits": repo_data["total_commits_analyzed"],
            "num_eras": len(repo_data.get("eras", [])),
            "era_based": True,
            "high_debt_files": era_debt_records,
        }

        all_tech_debt.append(result)

    output_file = os.path.join(OUTPUT_DIR, "tech_debt_report.json")
    with open(output_file, "w") as f:
        json.dump(all_tech_debt, f, indent=2)
    print(f"Saved: {output_file}")


if __name__ == "__main__":
    main()
