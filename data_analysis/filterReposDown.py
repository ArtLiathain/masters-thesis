import pandas as pd
import json
import matplotlib.pyplot as plt
import seaborn as sns

# 1. Load your data
with open("thesis_data_v2.json", "r") as f:
    df = pd.DataFrame(json.load(f))

# 1. Identify the outlier

OUTLIER_MAX_COMMITS = 5000
MIN_COMMITS = 100
MIN_CONTRIBUTORS = 4

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
sns.histplot(df_final["commit_count"], kde=True, ax=axes[0, 0], color="#2c3e50")
axes[0, 0].set_title("A. Distribution of Commit Counts", fontsize=15, fontweight="bold")
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
axes[0, 1].set_title("B. Distribution of Contributors", fontsize=15, fontweight="bold")
axes[0, 1].set_xlabel("Number of Contributors")

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
axes[1, 0].set_title("C. Commit vs Contributor Density", fontsize=15, fontweight="bold")
axes[1, 0].set_xlabel("Contributors (Log Scale)")
axes[1, 0].set_ylabel("Commits (Log Scale)")

# --- Plot 4: Repo Size Boxplot ---
sns.boxplot(x=df_final["size_kb"], ax=axes[1, 1], color="#2ecc71")
axes[1, 1].set_title(
    "D. Distribution of Repository Size (KB)", fontsize=15, fontweight="bold"
)
axes[1, 1].set_xlabel("Size in Kilobytes")

plt.tight_layout()
plt.savefig("pre_filter_distribution.png", dpi=300)
print("Graph saved as 'pre_filter_distribution.png'")

df_final.to_json("filtered_repos.json", orient="records", indent=2)
print("Filtered repos saved to 'filtered_repos.json'")

# Print some quick stats for your analysis text
print("\n--- Summary Statistics (Pre-Filtering) ---")
print(df_final[["commit_count", "contributor_count", "size_kb"]].describe().to_string())
