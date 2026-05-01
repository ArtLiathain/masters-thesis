# Architectural Guardrails for Research Software

A master's thesis exploring the use of machine learning to predict maintainability risk in C++ research repositories using a novel "hub score" metric.

## Overview

This project analyzes C++ repositories from the Journal of Open Source Software (JOSS) to:
1. Calculate **hub scores** - a measure of temporal coupling between files based on commit history
2. Generate structural code metrics using CodeScene
3. Train machine learning models (Random Forests, Decision Trees) to predict high-risk files
4. Derive domain-specific static analysis rules for research software

## Prerequisites

- **Rust** (for the repo analyzer tool)
- **Docker & Docker Compose** (for Neo4j)
- **uv** (for Python package management)
- **Typst** (for building the thesis)

## Setup

### 1. Install Rust Tool

```bash
cd rust_repo_analyser
cargo build --release
```

The binary will be at `target/release/cli` (or `target/debug/cli` for debug build).

### 2. Start Neo4j

```bash
cd rust_repo_analyser
docker-compose up -d
```

This starts two Neo4j instances:
- `neo4j` on http://localhost:7474 (main database)
- `neo4j-test` on http://localhost:7475 (testing)

Both run with authentication disabled.

### 3. Install Python Dependencies

```bash
cd data_analysis
uv sync
```

Required packages include: pandas, scikit-learn, matplotlib, seaborn, neo4j, torch, skope-rules, te2rules

## Complete Pipeline

### Phase 1: Data Collection

**1. Scrape JOSS Papers** - Fetch C++ papers from Journal of Open Source Software:
```bash
cd rust_repo_analyser
cargo run --release -- joss --language c++ --output ../results/joss_c++_papers.json
```

**2. Scrape GitHub Statistics** - Requires GitHub token:
```bash
cargo run --release -- github \
  --input ../results/joss_c++_papers.json \
  --output ../results/joss_c++_repo_stats.json \
  --token YOUR_GITHUB_TOKEN
```

**3. Filter Repositories** - Python filters to "engineered software" (min 100 commits, 3+ contributors):
```bash
cd data_analysis
python src/filterReposDown.py \
  ../results/joss_c++_repo_stats.json \
  ../results/filtered_C++_papers.json
```

**4. Clone and Analyze** - Use filtered Python output to analyze in Neo4j:
```bash
cd rust_repo_analyser
cargo run --release -- clone \
  --input ../results/filtered_C++_papers.json \
  --extension .cpp
```

This clones repos to `./repo_cache` and populates Neo4j with the commit graph.

**5. Export Hub Scores** - Export to Python for threshold calculation:
```bash
cargo run --release -- export-hub-scores \
  --extension .cpp \
  --output ../results/hub_scores.json
```

### Phase 2: Threshold Calculation

**6. Calculate Jenks Breaks** - Python calculates risk thresholds from hub scores:
```bash
cd data_analysis
python src/hub_gmm_analysis.py
```

This prints the break thresholds (e.g., `Break 1: 0.1234`). Note this value for the next step.

### Phase 3: Generate ML Dataset

**7. Copy Files by Threshold** - Use Python's break threshold to categorize files:
```bash
cd rust_repo_analyser
cargo run --release -- copy \
  --score-hub-threshold 0.1234 \
  --output data/files \
  --extension cpp
```

This copies files to `data/files/high/` (≥ threshold) and `data/files/low/` (< threshold).

**8. Generate Metrics CSV** - Create CSV input for ML pipeline:
```bash
cargo run --release -- metrics \
  --folder data/files \
  --output ../results/ml_metrics.csv
```

### Phase 4: Validation

**9. Analyse Local** - Analyze one repo in isolation:
```bash
cargo run --release -- analyse-local \
  --repo ./case_study_repos/acts \
  --name acts \
  --extension .cpp,.h
```

**10. CodeScene Analyze** - Get metrics from online CodeScene platform:
```bash
cargo run --release -- codescene-analyze \
  --repo ./case_study_repos/acts \
  --token YOUR_CODESCENE_TOKEN \
  --project-id YOUR_PROJECT_ID \
  --output-csv ../results/acts_codescene.csv
```

## Analysis Scripts

### Compare Hub Score vs CodeScene
```bash
cd data_analysis
python src/compare_metrics.py \
  --hub-score ../results/hub_scores.csv \
  --codescene ../results/codescene_metrics.csv
```

### ML Validation
```bash
python machine_learning/validation.py \
  machine_learning/validation_data.csv
```

## Case Study Repositories

CERN-related C++ repositories in `case_study_repos/`:

- **ACTS** - Accelerated CMOS Particle Tracking Suite
- **Merlin** - Monte Carlo simulation for particle accelerators
- **vecmem** - Vectorised memory management for GPU HEP
- **sixtracklib** - SixTrack simulation library
- **LGC2** - Laser-Geant4 Coupler

## Results

Key output files in `results/`:

| File | Description |
|------|-------------|
| `hub_scores.json` | Hub scores for all analyzed files |
| `*_metrics.csv` | CodeScene metrics per repository |
| `joss_c++_papers.json` | Scraped JOSS C++ papers |
| `joss_c++_repo_stats.json` | GitHub statistics for repos |

## Building the Thesis

The thesis is written in Typst. To compile:

```bash
cd report
typst compile thesis.typ thesis.pdf
```

Or watch for changes:

```bash
typst watch thesis.typ thesis.pdf
```

## Project Structure

```
.
├── rust_repo_analyser/       # Rust CLI tool for repo analysis
│   ├── cli/                  # Command-line interface
│   ├── repo_analyser/        # Core analysis logic
│   ├── repo_scraper/         # JOSS/GitHub scrapers
│   └── docker-compose.yml    # Neo4j setup
├── data_analysis/            # Python ML analysis
│   ├── src/                  # Analysis scripts
│   └── machine_learning/     # ML model training & validation
├── case_study_repos/         # CERN repositories for validation
├── report/                   # Thesis (Typst)
└── results/                  # Generated data and metrics
```

## Key Concepts

- **Hub Score**: Measures how strongly a file is temporally coupled to others (changed together in commits)
- **Temporal Coupling**: Files that frequently change together indicate potential architectural issues
- **Jenks Natural Breaks**: Algorithm for finding optimal thresholds in data distribution
- **Random Forests**: ML model for predicting high-risk files from code metrics

## License

MIT License - see individual components for their licenses.