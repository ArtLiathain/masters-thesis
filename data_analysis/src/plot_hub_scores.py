import os
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from neo4j import GraphDatabase

NEO4J_URI = os.getenv("NEO4J_URI", "bolt://localhost:7687")
NEO4J_USER = os.getenv("NEO4J_USER", "")
NEO4J_PASSWORD = os.getenv("NEO4J_PASSWORD", "")
OUTPUT_DIR = "/home/art/Development/masters-thesis/results"


def get_hub_scores_from_neo4j():
    driver = GraphDatabase.driver(NEO4J_URI, auth=(NEO4J_USER, NEO4J_PASSWORD))

    query = """
    MATCH (f:File)
    WHERE f.hub_score IS NOT NULL
    RETURN f.repo as repo,
           f.path as path,
           f.commit_count as commit_count,
           f.additions as additions,
           f.deletions as deletions,
           f.additions + f.deletions as churn,
           f.partner_count as partner_count,
           f.avg_coupling as avg_coupling,
           f.hub_score as hub_score,
           f.deleted_at_commit as deleted_at_commit
    """

    records = []
    with driver.session() as session:
        result = session.run(query)
        for row in result:
            records.append(
                {
                    "repo": row["repo"],
                    "path": row["path"],
                    "commit_count": row["commit_count"],
                    "additions": row["additions"],
                    "deletions": row["deletions"],
                    "churn": row["churn"],
                    "partner_count": row["partner_count"],
                    "avg_coupling": row["avg_coupling"],
                    "hub_score": row["hub_score"],
                    "deleted": row["deleted_at_commit"] is not None,
                }
            )

    driver.close()
    return pd.DataFrame(records)


df = get_hub_scores_from_neo4j()

if df.empty:
    print("No hub score data found in Neo4j")
    exit(1)

sns.set_theme(style="whitegrid", font_scale=1.1)

fig, axes = plt.subplots(2, 2, figsize=(16, 14))

active = df[~df["deleted"]]
deleted = df[df["deleted"]]

ax1 = axes[0, 0]
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

ax2 = axes[0, 1]
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

ax3 = axes[1, 0]
top_15 = df.nlargest(15, "hub_score").copy()
top_15["short_path"] = top_15["path"].apply(lambda x: os.path.basename(x))
max_score = top_15["hub_score"].max()
colors = [plt.cm.Reds(s / max_score) for s in top_15["hub_score"]]
bars = ax3.barh(range(len(top_15)), top_15["hub_score"], color=colors)
ax3.set_yticks(range(len(top_15)))
ax3.set_yticklabels(top_15["short_path"], fontsize=9)
ax3.set_xlabel("Hub Score")
ax3.set_title("C. Top 15 Most Problematic Files")
ax3.invert_yaxis()

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

ax4 = axes[1, 1]
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
plt.savefig(f"{OUTPUT_DIR}/hub_score_visualization.png")
print(f"Saved: {OUTPUT_DIR}/hub_score_visualization.png")

repos = df["repo"].unique()
print(f"\n{'=' * 60}")
print("SUMMARY STATISTICS")
print("=" * 60)
print(f"Repositories: {len(repos)}")
print(f"Files analyzed: {len(df)}")
print(f"  Active files: {len(active)}")
print(f"  Deleted files: {len(deleted)}")
print(f"Mean hub score: {df['hub_score'].mean():.6f}")
print(f"Max hub score: {df['hub_score'].max():.6f}")
print(f"Mean partner count: {df['partner_count'].mean():.1f}")
print(f"Mean coupling: {df['avg_coupling'].mean():.4f}")
