import pandas as pd
import json
import matplotlib.pyplot as plt
import seaborn as sns
import argparse

parser = argparse.ArgumentParser(
    description="Filter repositories based on commit and contributor thresholds.")
parser.add_argument("input_file", help="Path to input JSON file")
parser.add_argument("output_file", help="Path to output JSON file")
args = parser.parse_args()

with open(args.input_file, "r") as f:
    df = pd.DataFrame(json.load(f))


OUTLIER_MAX_COMMITS = 5000
MIN_COMMITS = 100
MIN_CONTRIBUTORS = 3

# 3. Perform Filtering
# Step A: Drop the giant outliers
df_no_outliers = df[df["commit_count"] <= OUTLIER_MAX_COMMITS].copy()

# Step B: Apply minimums for "Engineered Software"
df_final = df_no_outliers[
    (df_no_outliers["commit_count"] >= MIN_COMMITS)
    & (df_no_outliers["contributor_count"] >= MIN_CONTRIBUTORS)
].copy()

# 4. Reporting
print(f"Total projects in original JSON: {len(df)}")
print(
    f"Projects dropped as outliers (> {OUTLIER_MAX_COMMITS} commits): {
        len(df) - len(df_no_outliers)
    }"
)
print(
    f"Projects filtered out (below {MIN_COMMITS} commits or {
        MIN_CONTRIBUTORS
    } contributors): {len(df_no_outliers) - len(df_final)}"
)
print(f"Final sample size for analysis: {len(df_final)}")

# Set the visual style
sns.set_theme(style="whitegrid")
fig, axes = plt.subplots(2, 2, figsize=(15, 12))

# --- Plot 1: Commit Distribution ---
sns.histplot(df_final["commit_count"], kde=True,
             ax=axes[0, 0], color="#2c3e50")
axes[0, 0].set_title("A. Distribution of Commit Counts",
                     fontsize=15, fontweight="bold")
axes[0, 0].set_xlabel("Number of Commits")

# --- Plot 2: Contributor Distribution ---
# Using bins=range to handle the integer nature of contributors
sns.histplot(
    df_final["contributor_count"],
    kde=False,
    ax=axes[0, 1],
    color="#e74c3c",
    bins=range(1, df_final["contributor_count"].max() + 2),
)
axes[0, 1].set_title("B. Distribution of Contributors",
                     fontsize=15, fontweight="bold")
axes[0, 1].set_xlabel("Number of Contributors")
axes[0, 1].set_xlim(0, 20)

# --- Plot 3: Commits vs Contributors (Log-Log) ---
sns.scatterplot(
    data=df_final,
    x="contributor_count",
    y="commit_count",
    ax=axes[1, 0],
    alpha=0.6,
    s=100,
    color="#3498db",
)
axes[1, 0].set_xscale("log")
axes[1, 0].set_yscale("log")
axes[1, 0].set_title("C. Commit vs Contributor Density",
                     fontsize=15, fontweight="bold")
axes[1, 0].set_xlabel("Contributors (Log Scale)")
axes[1, 0].set_ylabel("Commits (Log Scale)")

# --- Plot 4: Repo Size Boxplot ---
sns.boxplot(x=df_final["size_kb"], ax=axes[1, 1], color="#2ecc71")
axes[1, 1].set_title(
    "D. Distribution of Repository Size (KB)", fontsize=15, fontweight="bold"
)
axes[1, 1].set_xlabel("Size in Kilobytes")

# PRE-FILTER: Raw data (with outliers) - shows why filtering is needed
# Cap outliers for visualization (winsorize) - note these are capped for display only
DISPLAY_CAP = 10000  # Cap at 10k for readable graph, note in text

df_commits_capped = df["commit_count"].clip(upper=DISPLAY_CAP)
df_contributors_capped = df["contributor_count"].clip(upper=20)
df_size_capped = df["size_kb"].clip(upper=df["size_kb"].quantile(0.99))

sns.set_theme(style="whitegrid")
fig1, axes1 = plt.subplots(2, 2, figsize=(15, 12))

sns.histplot(df_commits_capped, kde=True, ax=axes1[0, 0], color="#2c3e50")
axes1[0, 0].set_title("A. Distribution of Commit Counts (Raw)",
                      fontsize=15, fontweight="bold")
axes1[0, 0].set_xlabel("Number of Commits (capped at 10k for display)")
axes1[0, 0].axvline(MIN_COMMITS, color="#e74c3c",
                    linestyle="--", linewidth=2, label=f"Min: {MIN_COMMITS}")
axes1[0, 0].axvline(OUTLIER_MAX_COMMITS, color="#e74c3c", linestyle="-",
                    linewidth=2, label=f"Outlier cutoff: {OUTLIER_MAX_COMMITS}")
axes1[0, 0].axvspan(MIN_COMMITS, OUTLIER_MAX_COMMITS,
                    alpha=0.2, color="#27ae60", label="Kept zone")
axes1[0, 0].legend(fontsize=10)

sns.histplot(df_contributors_capped, kde=False, ax=axes1[0, 1], color="#e74c3c",
             bins=range(1, 22))
axes1[0, 1].set_title("B. Distribution of Contributors (Raw)",
                      fontsize=15, fontweight="bold")
axes1[0, 1].set_xlabel("Number of Contributors (capped at 20 for display)")
axes1[0, 1].axvline(MIN_CONTRIBUTORS, color="#2c3e50",
                    linestyle="--", linewidth=2, label=f"Min: {MIN_CONTRIBUTORS}")
axes1[0, 1].axvspan(MIN_CONTRIBUTORS, 21, alpha=0.2,
                    color="#27ae60", label="Kept zone")
axes1[0, 1].legend(fontsize=10)

sns.scatterplot(data=df, x="contributor_count", y="commit_count",
                ax=axes1[1, 0], alpha=0.6, s=100, color="#3498db")
axes1[1, 0].set_xscale("log")
axes1[1, 0].set_yscale("log")
axes1[1, 0].set_title("C. Commit vs Contributor Density (Raw)",
                      fontsize=15, fontweight="bold")
axes1[1, 0].set_xlabel("Contributors (Log Scale)")
axes1[1, 0].set_ylabel("Commits (Log Scale)")
axes1[1, 0].axhline(MIN_COMMITS, color="#e74c3c", linestyle="--", linewidth=2)
axes1[1, 0].axhline(OUTLIER_MAX_COMMITS, color="#e74c3c",
                    linestyle="-", linewidth=2)
axes1[1, 0].axvline(MIN_CONTRIBUTORS, color="#e74c3c",
                    linestyle="--", linewidth=2)

sns.boxplot(x=df_size_capped, ax=axes1[1, 1], color="#2ecc71")
axes1[1, 1].set_title("D. Distribution of Repository Size (KB) (Raw)",
                      fontsize=15, fontweight="bold")
axes1[1, 1].set_xlabel("Size in KB (capped at 99th percentile for display)")

plt.tight_layout()
fig1.savefig("pre_filter_distribution.png", dpi=300)
print("Graph saved as 'pre_filter_distribution.png'")
print("Note: Outliers are capped for display (commits at 10k, contributors at 20, size at 99th percentile)")

# POST-FILTER: Filtered data
fig2, axes2 = plt.subplots(2, 2, figsize=(15, 12))

sns.histplot(df_final["commit_count"], kde=True,
             ax=axes2[0, 0], color="#2c3e50")
axes2[0, 0].set_title("A. Distribution of Commit Counts (Filtered)",
                      fontsize=15, fontweight="bold")
axes2[0, 0].set_xlabel("Number of Commits")

sns.histplot(
    df_final["contributor_count"],
    kde=False,
    ax=axes2[0, 1],
    color="#e74c3c",
    bins=range(1, df_final["contributor_count"].max() + 2),
)
axes2[0, 1].set_title("B. Distribution of Contributors (Filtered)",
                      fontsize=15, fontweight="bold")
axes2[0, 1].set_xlabel("Number of Contributors")

sns.scatterplot(
    data=df_final,
    x="contributor_count",
    y="commit_count",
    ax=axes2[1, 0],
    alpha=0.6,
    s=100,
    color="#3498db",
)
axes2[1, 0].set_xscale("log")
axes2[1, 0].set_yscale("log")
axes2[1, 0].set_title("C. Commit vs Contributor Density (Filtered)",
                      fontsize=15, fontweight="bold")
axes2[1, 0].set_xlabel("Contributors (Log Scale)")
axes2[1, 0].set_ylabel("Commits (Log Scale)")

sns.boxplot(x=df_final["size_kb"], ax=axes2[1, 1], color="#2ecc71")
axes2[1, 1].set_title(
    "D. Distribution of Repository Size (KB) (Filtered)", fontsize=15, fontweight="bold"
)
axes2[1, 1].set_xlabel("Size in Kilobytes")

plt.tight_layout()
fig2.savefig("post_filter_distribution.png", dpi=300)
print("Graph saved as 'post_filter_distribution.png'")

df_final.to_json(args.output_file, orient="records", indent=2)
print(f"Filtered repos saved to '{args.output_file}'")

# Print some quick stats for your analysis text
print("\n--- Summary Statistics (Pre-Filtering - Raw) ---")
print(df[["commit_count", "contributor_count",
      "size_kb"]].describe().to_string())
print("\n--- Summary Statistics (Post-Filtering) ---")
print(df_final[["commit_count", "contributor_count",
      "size_kb"]].describe().to_string())
