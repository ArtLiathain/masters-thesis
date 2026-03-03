import json
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os

INPUT_FILE = "/home/art/Development/masters-thesis/results/hub_score_summary.csv"
OUTPUT_DIR = "/home/art/Development/masters-thesis/results"

# Load data
df = pd.read_csv(INPUT_FILE)

sns.set_theme(style="whitegrid", font_scale=1.1)

fig, axes = plt.subplots(2, 2, figsize=(16, 14))

# Plot 1: Hub Score Distribution
ax1 = axes[0, 0]
active = df[~df["deleted"]]
deleted = df[df["deleted"]]
ax1.hist(
    active["hub_score"],
    bins=30,
    alpha=0.7,
    color="steelblue",
    edgecolor="black",
    label=f"Active ({len(active)})",
)
if len(deleted) > 0:
    ax1.hist(
        deleted["hub_score"],
        bins=30,
        alpha=0.5,
        color="red",
        edgecolor="black",
        label=f"Deleted ({len(deleted)})",
    )
ax1.set_xlabel("Hub Score")
ax1.set_ylabel("Frequency")
ax1.set_title("A. Distribution of Hub Scores")
ax1.axvline(
    df["hub_score"].mean(),
    color="red",
    linestyle="--",
    label=f"Mean: {df['hub_score'].mean():.2f}",
)
ax1.legend()

# Plot 2: Partner Count vs Average Coupling (scatter)
ax2 = axes[0, 1]
active = df[~df["deleted"]]
deleted = df[df["deleted"]]
ax2.scatter(
    active["partner_count"],
    active["avg_coupling"],
    c=active["hub_score"],
    cmap="viridis",
    alpha=0.6,
    s=50,
    label="Active",
)
if len(deleted) > 0:
    ax2.scatter(
        deleted["partner_count"],
        deleted["avg_coupling"],
        c="red",
        alpha=0.6,
        s=50,
        marker="x",
        label="Deleted",
    )
ax2.set_xlabel("Partner Count (Number of Co-changed Files)")
ax2.set_ylabel("Average Coupling Ratio")
ax2.set_title("B. Partner Count vs Coupling (color = Hub Score)")
ax2.legend()

# Plot 3: Top 15 Files by Hub Score
ax3 = axes[1, 0]
top_15 = df.nlargest(15, "hub_score").copy()
top_15["short_path"] = top_15["path"].apply(lambda x: os.path.basename(x))
colors = plt.cm.Reds(top_15["hub_score"] / top_15["hub_score"].max())
bars = ax3.barh(range(len(top_15)), top_15["hub_score"], color=colors)
ax3.set_yticks(range(len(top_15)))
ax3.set_yticklabels(top_15["short_path"], fontsize=9)
ax3.set_xlabel("Hub Score")
ax3.set_title("C. Top 15 Most Problematic Files")
ax3.invert_yaxis()

# Add hub score values and deleted markers
for idx, (_, row) in enumerate(top_15.iterrows()):
    label = f"{row['hub_score']:.2f}"
    if row["deleted"]:
        label += " [DEL]"
        ax3.text(
            row["hub_score"] + 0.5,
            idx,
            label,
            va="center",
            fontsize=8,
            color="red",
            fontweight="bold",
        )
    else:
        ax3.text(
            row["hub_score"] + 0.5, idx, label, va="center", fontsize=8, color="gray"
        )

# Plot 4: Churn vs Hub Score
ax4 = axes[1, 1]
active = df[~df["deleted"]]
deleted = df[df["deleted"]]
scatter4 = ax4.scatter(
    active["churn"],
    active["hub_score"],
    alpha=0.5,
    c=active["hub_score"],
    cmap="plasma",
    s=30,
    label="Active",
)
if len(deleted) > 0:
    ax4.scatter(
        deleted["churn"],
        deleted["hub_score"],
        alpha=0.7,
        c="red",
        s=30,
        marker="x",
        label="Deleted",
    )
ax4.set_xlabel("Churn (Additions + Deletions)")
ax4.set_ylabel("Hub Score")
ax4.set_title("D. Churn vs Hub Score")
ax4.set_xscale("log")
ax4.legend()

plt.tight_layout()
print(f"Saved: {OUTPUT_DIR}/hub_score_visualization.png")

print("\n" + "=" * 60)
print("SUMMARY STATISTICS")
print("=" * 60)
print(f"Files analyzed: {len(df)}")
print(f"  Active files: {len(active)}")
print(f"  Deleted files: {len(deleted)}")
print(f"Mean hub score: {df['hub_score'].mean():.2f}")
print(f"Max hub score: {df['hub_score'].max():.2f}")
print(f"Mean partner count: {df['partner_count'].mean():.1f}")
print(f"Mean coupling: {df['avg_coupling'].mean():.4f}")
