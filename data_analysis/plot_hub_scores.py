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

# Plot 1: Hub Score Distribution by Era
ax1 = axes[0, 0]
for era in df["era_index"].unique():
    era_data = df[df["era_index"] == era]
    ax1.hist(
        era_data["hub_score"], bins=30, alpha=0.6, label=f"Era {era}", edgecolor="black"
    )
ax1.set_xlabel("Hub Score")
ax1.set_ylabel("Frequency")
ax1.set_title("A. Distribution of Hub Scores by Era")
ax1.legend()

# Plot 2: Partner Count vs Average Coupling (scatter)
ax2 = axes[0, 1]
scatter = ax2.scatter(
    df["partner_count"],
    df["avg_coupling"],
    c=df["hub_score"],
    cmap="viridis",
    alpha=0.6,
    s=50,
)
ax2.set_xlabel("Partner Count (Number of Co-changed Files)")
ax2.set_ylabel("Average Coupling Ratio")
ax2.set_title("B. Partner Count vs Coupling (color = Hub Score)")
plt.colorbar(scatter, ax=ax2, label="Hub Score")

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

# Add era labels
for idx, (_, row) in enumerate(top_15.iterrows()):
    ax3.text(
        row["hub_score"] + 1,
        idx,
        f"E{row['era_index']}",
        va="center",
        fontsize=8,
        color="gray",
    )

# Plot 4: Churn vs Hub Score
ax4 = axes[1, 1]
ax4.scatter(
    df["churn"], df["hub_score"], alpha=0.5, c=df["era_index"], cmap="coolwarm", s=30
)
ax4.set_xlabel("Churn (Additions + Deletions)")
ax4.set_ylabel("Hub Score")
ax4.set_title("D. Churn vs Hub Score (color = Era)")
ax4.set_xscale("log")

handles = []
eras = sorted(df["era_index"].unique())
norm = plt.Normalize(df["era_index"].min(), df["era_index"].max())
sm = plt.cm.ScalarMappable(cmap="coolwarm", norm=norm)
sm.set_array([])
for era in eras:
    handles.append(
        plt.Line2D(
            [0],
            [0],
            marker="o",
            color="w",
            markerfacecolor=sm.to_rgba(era),
            markersize=10,
            label=f"Era {era}",
        )
    )
ax4.legend(handles=handles, title="Era")

plt.tight_layout()
plt.savefig(f"{OUTPUT_DIR}/hub_score_visualization.png", dpi=150, bbox_inches="tight")
print(f"Saved: {OUTPUT_DIR}/hub_score_visualization.png")

# Additional: Era comparison summary
print("\n" + "=" * 60)
print("ERA COMPARISON SUMMARY")
print("=" * 60)
for era in sorted(df["era_index"].unique()):
    era_data = df[df["era_index"] == era]
    print(f"\nEra {era}:")
    print(f"  Files analyzed: {len(era_data)}")
    print(f"  Mean hub score: {era_data['hub_score'].mean():.2f}")
    print(f"  Max hub score: {era_data['hub_score'].max():.2f}")
    print(f"  Mean partner count: {era_data['partner_count'].mean():.1f}")
    print(f"  Mean coupling: {era_data['avg_coupling'].mean():.4f}")

plt.show()
