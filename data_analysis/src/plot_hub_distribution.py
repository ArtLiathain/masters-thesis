import argparse
import json

import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd


def main():
    parser = argparse.ArgumentParser(description="Plot hub score distribution from JSON")
    parser.add_argument("--input", required=True, help="Path to hub_scores.json")
    parser.add_argument(
        "--output", help="Path to save plot (default: <input>_distribution.png)"
    )
    args = parser.parse_args()

    with open(args.input) as f:
        data = json.load(f)

    df = pd.DataFrame(data)

    if df.empty:
        print("No data found")
        return

    output_path = args.output or args.input.replace(".json", "_distribution.png")

    sns.set_theme(style="whitegrid", font_scale=1.1)

    fig, axes = plt.subplots(3, 2, figsize=(14, 16))

    ax1 = axes[0, 0]
    ax1.hist(df["hub_score"], bins=30, alpha=0.7, color="steelblue", edgecolor="black")
    ax1.set_xlabel("Hub Score")
    ax1.set_ylabel("Frequency")
    ax1.set_title("Distribution of Hub Scores")
    ax1.axvline(df["hub_score"].mean(), color="red", linestyle="--", label=f"Mean: {df['hub_score'].mean():.4f}")
    ax1.legend()

    ax2 = axes[0, 1]
    ax2.hist(df["avg_coupling"], bins=30, alpha=0.7, color="forestgreen", edgecolor="black")
    ax2.set_xlabel("Average Coupling")
    ax2.set_ylabel("Frequency")
    ax2.set_title("Distribution of Average Coupling")
    ax2.axvline(df["avg_coupling"].mean(), color="red", linestyle="--", label=f"Mean: {df['avg_coupling'].mean():.4f}")
    ax2.legend()

    ax3 = axes[1, 0]
    ax3.scatter(df["partner_count"], df["hub_score"], alpha=0.5, c="steelblue", s=30)
    ax3.set_xlabel("Partner Count")
    ax3.set_ylabel("Hub Score")
    ax3.set_title("Partner Count vs Hub Score")

    ax4 = axes[1, 1]
    ax4.scatter(df["commit_count"], df["hub_score"], alpha=0.5, c="coral", s=30)
    ax4.set_xlabel("Commit Count")
    ax4.set_ylabel("Hub Score")
    ax4.set_title("Commit Count vs Hub Score")

    ax5 = axes[2, 0]
    ax5.scatter(df["churn"], df["hub_score"], alpha=0.5, c="purple", s=30)
    ax5.set_xlabel("Churn (Additions + Deletions)")
    ax5.set_ylabel("Hub Score")
    ax5.set_title("Churn vs Hub Score")

    ax6 = axes[2, 1]
    ax6.scatter(df["avg_coupling"], df["hub_score"], alpha=0.5, c="forestgreen", s=30)
    ax6.set_xlabel("Average Coupling")
    ax6.set_ylabel("Hub Score")
    ax6.set_title("Average Coupling vs Hub Score")

    plt.tight_layout()
    plt.savefig(output_path)
    print(f"Saved: {output_path}")

    repos = df["repo"].nunique()
    print(f"Files: {len(df)}, Repos: {repos}")
    print(f"Hub score - min: {df['hub_score'].min():.4f}, max: {df['hub_score'].max():.4f}, mean: {df['hub_score'].mean():.4f}")


if __name__ == "__main__":
    main()