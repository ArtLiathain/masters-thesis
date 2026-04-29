import json
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
from scipy.stats import mstats
from jenkspy import JenksNaturalBreaks


def winsorize(series, limits=(0.05, 0.05)):
    return mstats.winsorize(series, limits=limits).data


def main():
    df = pd.read_json("../results/hub_scores.json")

    df["hub_score_winsorized"] = winsorize(
        df["hub_score"].values, limits=(0.05, 0.05))
    df["partner_count_winsorized"] = winsorize(
        df["partner_count"].values, limits=(0.05, 0.05))
    df["avg_coupling_winsorized"] = winsorize(
        df["avg_coupling"].values, limits=(0.05, 0.05))

    df["hub_score_log"] = np.log1p(df["hub_score_winsorized"])

    jnb = JenksNaturalBreaks(n_classes=3)
    jnb.fit(df["hub_score_winsorized"].values)

    break1, break2 = jnb.inner_breaks_
    df["risk_label"] = ["low" if l == 0 else "medium" if l ==
                        1 else "high" for l in jnb.labels_]
    df["risk_binary"] = df["risk_label"].apply(
        lambda x: "low" if x == "low" else "medium_high")

    metrics = ["avg_coupling", "commit_count", "partner_count", "churn"]
    titles = ["Avg Coupling", "Commit Count", "Partner Count", "Churn"]

    fig, axes = plt.subplots(2, 3, figsize=(18, 10))
    fig.suptitle("Hub Score Risk Analysis", fontsize=16, fontweight="bold")

    colors = {"low": "#4C9BE8", "medium_high": "#E65C00"}

    for ax, metric, title in zip(axes.flat[:4], metrics, titles):
        for label, group in df.groupby("risk_binary"):
            ax.scatter(
                group["hub_score_log"],
                group[metric],
                c=colors[label],
                alpha=0.55,
                s=18,
                label=label.replace("_", " ").capitalize() + " Risk",
                edgecolors="none",
            )

        break1_log = np.log1p(break1)
        ax.axvline(break1_log, color="black", linestyle="--", linewidth=1.2,
                   label=f"Break ≈ {break1:.3f}")

        log_ticks = np.linspace(
            df["hub_score_log"].min(), df["hub_score_log"].max(), 6)
        ax.set_xticks(log_ticks)
        ax.set_xticklabels(
            [f"{np.expm1(v):.1f}" for v in log_ticks], fontsize=8)

        ax.set_xlabel("Hub Score", fontsize=9)
        ax.set_ylabel(title, fontsize=9)
        ax.set_title(title, fontsize=11, fontweight="bold")
        ax.legend(fontsize=8)
        ax.grid(True, alpha=0.2)

    ax_hist1 = axes.flat[4]
    hub_scores = df["hub_score_winsorized"]
    hub_min, hub_max = hub_scores.min(), hub_scores.max()

    ax_hist1.axvspan(hub_min, break1, alpha=0.2,
                     color="#4C9BE8", label="Low Risk Zone")
    ax_hist1.axvspan(break1, hub_max, alpha=0.2,
                     color="#E65C00", label="High Risk Zone")

    bins = np.linspace(hub_min, hub_max, 51)
    counts, bin_edges = np.histogram(hub_scores, bins=bins)

    bin_centers = (bin_edges[:-1] + bin_edges[1:]) / 2
    bar_colors = ["#4C9BE8" if b < break1 else "#E65C00" for b in bin_centers]

    ax_hist1.bar(bin_centers, counts, width=bins[1] - bins[0],
                 color=bar_colors, alpha=0.7, edgecolor="black", linewidth=0.5)
    ax_hist1.axvline(break1, color="black", linestyle="--",
                     linewidth=1.2, label=f"Break ≈ {break1:.4f}")

    ax_hist1.set_xlabel("Hub Score (Winsorized)", fontsize=9)
    ax_hist1.set_ylabel("Frequency", fontsize=9)
    ax_hist1.set_title("Hub Score Distribution",
                       fontsize=11, fontweight="bold")
    ax_hist1.legend(fontsize=8)
    ax_hist1.grid(True, alpha=0.2)

    ax_hist2 = axes.flat[5]
    avg_coupling = df["avg_coupling_winsorized"]
    ac_min, ac_max = avg_coupling.min(), avg_coupling.max()

    ax_hist2.hist(avg_coupling, bins=50, color="#E65C00",
                  alpha=0.7, edgecolor="black", linewidth=0.5)
    ax_hist2.set_xlabel("Avg Coupling (Winsorized)", fontsize=9)
    ax_hist2.set_ylabel("Frequency", fontsize=9)
    ax_hist2.set_title("Avg Coupling Distribution",
                       fontsize=11, fontweight="bold")
    ax_hist2.grid(True, alpha=0.2)

    plt.tight_layout()
    plt.savefig("hub_risk_output.png", dpi=150, bbox_inches="tight")

    low_count = (df["risk_label"] == "low").sum()
    med_count = (df["risk_label"] == "medium").sum()
    high_count = (df["risk_label"] == "high").sum()
    total = len(df)

    print(f"Jenks Natural Breaks (3 classes):")
    print(f"  Break 1 (Low->Medium): {break1:.4f}")
    print(f"  Break 2 (Medium->High): {break2:.4f}")
    print(f"  Group counts: Low={low_count} ({100*low_count/total:.1f}%), Medium={
          med_count} ({100*med_count/total:.1f}%), High={high_count} ({100*high_count/total:.1f}%)")
    print(
        f"  Binary (Low vs Medium+High): Low={low_count}, Medium+High={med_count+high_count}")
    print(f"Hub score range (original): {
          df['hub_score'].min():.4f} - {df['hub_score'].max():.4f}")
    print(f"Hub score range (winsorized): {
          df['hub_score_winsorized'].min():.4f} - {df['hub_score_winsorized'].max():.4f}")


if __name__ == "__main__":
    main()
