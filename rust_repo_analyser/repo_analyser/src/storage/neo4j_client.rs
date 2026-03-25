use neo4rs::{query, ConfigBuilder, Graph};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    pub additions: i64,
    pub deletions: i64,
    pub commit_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub repo: String,
    pub total_commits_analyzed: i64,
    pub files: Vec<FileNode>,
    pub edges: Vec<Edge>,
}

pub struct Neo4jClient {
    graph: Arc<Mutex<Graph>>,
}

impl Neo4jClient {
    pub async fn new(uri: &str) -> Result<Self, String> {
        let config = ConfigBuilder::default()
            .uri(uri)
            .user("")
            .password("")
            .build()
            .map_err(|e| format!("Failed to build config: {}", e))?;

        let graph = Graph::connect(config)
            .await
            .map_err(|e| format!("Failed to connect to Neo4j: {}", e))?;

        Ok(Self {
            graph: Arc::new(Mutex::new(graph)),
        })
    }

    pub async fn init_schema(&self) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let queries = vec![
            "CREATE CONSTRAINT IF NOT EXISTS FOR (r:Repository) REQUIRE r.name IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (f:File) REQUIRE (f.repo, f.path) IS UNIQUE",
            "CREATE INDEX IF NOT EXISTS FOR (f:File) ON (f.repo)",
        ];

        for q in queries {
            graph
                .run(query(q))
                .await
                .map_err(|e| format!("Failed to execute schema query: {}", e))?;
        }

        Ok(())
    }

    pub async fn save_repository(&self, name: &str, url: Option<&str>) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = if url.is_some() {
            query("MERGE (r:Repository {name: $name}) SET r.url = $url")
                .param("name", name)
                .param("url", url.unwrap())
        } else {
            query("MERGE (r:Repository {name: $name})").param("name", name)
        };

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to save repository: {}", e))?;

        Ok(())
    }

    pub async fn update_commit_count(&self, repo: &str, total_commits: i64) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = query("MATCH (r:Repository {name: $repo}) SET r.total_commits = $total_commits")
            .param("repo", repo)
            .param("total_commits", total_commits);

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to update commit count: {}", e))?;

        Ok(())
    }

    pub async fn save_file_node(
        &self,
        repo: &str,
        path: &str,
        additions: i64,
        deletions: i64,
        deleted_at_commit: Option<&str>,
    ) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = query(
            "MERGE (f:File {repo: $repo, path: $path}) \
             SET f.additions = COALESCE(f.additions, 0) + $additions, \
                 f.deletions = COALESCE(f.deletions, 0) + $deletions, \
                 f.commit_count = COALESCE(f.commit_count, 0) + 1, \
                 f.deleted_at_commit = COALESCE(f.deleted_at_commit, $deleted_at_commit)",
        )
        .param("repo", repo)
        .param("path", path)
        .param("additions", additions)
        .param("deletions", deletions)
        .param("deleted_at_commit", deleted_at_commit);

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to save file node: {}", e))?;

        Ok(())
    }

    pub async fn save_cochange_relationship(
        &self,
        repo: &str,
        source: &str,
        target: &str,
    ) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = query(
            "MATCH (f1:File {repo: $repo, path: $source}) MATCH (f2:File {repo: $repo, path: $target}) MERGE (f1)-[r:CO_CHANGED]->(f2) SET r.weight = COALESCE(r.weight, 0) + 1",
        )
        .param("repo", repo)
        .param("source", source)
        .param("target", target);

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to save co-change relationship: {}", e))?;

        Ok(())
    }

    pub async fn get_graph(&self, repo: &str) -> Result<GraphData, String> {
        let graph = self.graph.lock().await;

        let files_query = query(
            "MATCH (f:File {repo: $repo}) RETURN f.path as path, f.additions as additions, f.deletions as deletions, f.commit_count as commit_count",
        )
        .param("repo", repo);

        let mut files_result = graph
            .execute(files_query)
            .await
            .map_err(|e| format!("Failed to query files: {}", e))?;

        let mut files = Vec::new();
        while let Ok(Some(row)) = files_result.next().await {
            let path: String = row.get::<String>("path").unwrap_or_default();
            let additions: i64 = row.get::<i64>("additions").unwrap_or(0);
            let deletions: i64 = row.get::<i64>("deletions").unwrap_or(0);
            let commit_count: i64 = row.get::<i64>("commit_count").unwrap_or(0);

            files.push(FileNode {
                path,
                additions,
                deletions,
                commit_count,
            });
        }

        let edges_query = query(
            "MATCH (f1:File {repo: $repo})-[r:CO_CHANGED]->(f2:File {repo: $repo}) RETURN f1.path as source, f2.path as target, r.weight as weight",
        )
        .param("repo", repo);

        let mut edges_result = graph
            .execute(edges_query)
            .await
            .map_err(|e| format!("Failed to query edges: {}", e))?;

        let mut edges = Vec::new();
        while let Ok(Some(row)) = edges_result.next().await {
            let source: String = row.get::<String>("source").unwrap_or_default();
            let target: String = row.get::<String>("target").unwrap_or_default();
            let weight: i64 = row.get::<i64>("weight").unwrap_or(0);

            edges.push(Edge {
                source,
                target,
                weight,
            });
        }

        let count_query =
            query("MATCH (r:Repository {name: $repo}) RETURN r.total_commits as total_commits")
                .param("repo", repo);

        let mut count_result = graph
            .execute(count_query)
            .await
            .map_err(|e| format!("Failed to query commit count: {}", e))?;

        let total_commits = if let Ok(Some(row)) = count_result.next().await {
            row.get::<i64>("total_commits").unwrap_or(0)
        } else {
            0
        };

        Ok(GraphData {
            repo: repo.to_string(),
            total_commits_analyzed: total_commits,
            files,
            edges,
        })
    }

    pub async fn link_file_to_repo(&self, repo: &str, file_path: &str) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = query(
            "MATCH (r:Repository {name: $repo}) MATCH (f:File {repo: $repo, path: $file_path}) MERGE (r)-[:CONTAINS]->(f)",
        )
        .param("repo", repo)
        .param("file_path", file_path);

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to link file to repo: {}", e))?;

        Ok(())
    }

    pub async fn remove_low_importance_connections(&self, threshold: i64) -> Result<u64, String> {
        let graph = self.graph.lock().await;

        let q = query(
            "MATCH (f1:File)-[r:CO_CHANGED]->(f2:File)
     WHERE r.weight < $threshold
     DELETE r
     RETURN count(r) as deleted_count",
        )
        .param("threshold", threshold);

        let mut result = graph
            .execute(q)
            .await
            .map_err(|e| format!("Failed to prune edges: {}", e))?;

        let deleted_edges = if let Ok(Some(row)) = result.next().await {
            row.get::<i64>("deleted_count").unwrap_or(0) as u64
        } else {
            0
        };

        let q2 = query(
            "
     MATCH (f:File)
     WHERE NOT (f)-[:CO_CHANGED]->(:File) AND NOT (:File)-[:CO_CHANGED]->(f)
     DETACH DELETE f
 ",
        );

        graph
            .run(q2)
            .await
            .map_err(|e| format!("Failed to remove orphan files: {}", e))?;

        let zero_churn_query = query(
            "MATCH (f:File) \
             WHERE f.additions = 0 AND f.deletions = 0 AND f.deleted_at_commit IS NULL \
             DETACH DELETE f",
        );

        graph
            .run(zero_churn_query)
            .await
            .map_err(|e| format!("Failed to prune zero-churn files: {}", e))?;

        Ok(deleted_edges)
    }

    pub async fn compute_hub_scores(&self, repo: &str) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let totals_query = query(
            "MATCH (f:File {repo: $repo}) RETURN count(f) as total_files, sum(f.additions + f.deletions) as total_churn"
        )
        .param("repo", repo);

        let mut totals_result = graph
            .execute(totals_query)
            .await
            .map_err(|e| format!("Failed to get totals: {}", e))?;

        let (total_files, total_churn) = if let Ok(Some(row)) = totals_result.next().await {
            let tf: i64 = row.get::<i64>("total_files").unwrap_or(0);
            let tc: i64 = row.get::<i64>("total_churn").unwrap_or(0);
            (tf, tc)
        } else {
            return Err("No files found in repository".to_string());
        };

        if total_files == 0 || total_churn == 0 {
            return Ok(());
        }

        let compute_query = query(
            "MATCH (f:File {repo: $repo})-[r:CO_CHANGED]->(t:File {repo: $repo}) \
             WITH f, collect({weight: r.weight, target_commits: t.commit_count}) as edges, \
                  (f.additions + f.deletions) as file_churn \
             WITH f, [e IN edges WHERE e.weight IS NOT NULL] as valid_edges, file_churn \
             WITH f, size(valid_edges) as partner_count, valid_edges, file_churn \
             UNWIND valid_edges as e \
             WITH f, partner_count, avg(toFloat(e.weight) / e.target_commits) as avg_coupling, file_churn \
             RETURN f.path as path, partner_count, avg_coupling, file_churn"
        )
        .param("repo", repo);

        let mut compute_result = graph
            .execute(compute_query)
            .await
            .map_err(|e| format!("Failed to compute hub scores: {}", e))?;

        let mut updates = Vec::new();
        while let Ok(Some(row)) = compute_result.next().await {
            let path: String = row.get::<String>("path").unwrap_or_default();
            let partner_count: i64 = row.get::<i64>("partner_count").unwrap_or(0);
            let avg_coupling: f64 = row.get::<f64>("avg_coupling").unwrap_or(0.0);
            let file_churn: i64 = row.get::<i64>("file_churn").unwrap_or(0);

            let hub_score = avg_coupling
                * (partner_count as f64 / total_files as f64)
                * (file_churn as f64 / total_churn as f64);

            updates.push((path, partner_count, avg_coupling, hub_score));
        }

        for (path, partner_count, avg_coupling, hub_score) in updates {
            let update_query = query(
                "MATCH (f:File {repo: $repo, path: $path}) \
                 SET f.partner_count = $partner_count, f.avg_coupling = $avg_coupling, f.hub_score = $hub_score"
            )
            .param("repo", repo)
            .param("path", path)
            .param("partner_count", partner_count)
            .param("avg_coupling", avg_coupling)
            .param("hub_score", hub_score);

            graph
                .run(update_query)
                .await
                .map_err(|e| format!("Failed to update hub score: {}", e))?;
        }
        Ok(())
    }

    pub async fn link_all_files_to_repo(&self, repo: &str) -> Result<(), String> {
        let graph = self.graph.lock().await;

        let q = query(
            "MATCH (r:Repository {name: $repo}) MATCH (f:File {repo: $repo}) WHERE NOT (r)-[:CONTAINS]->(f) MERGE (r)-[:CONTAINS]->(f)",
        )
        .param("repo", repo);

        graph
            .run(q)
            .await
            .map_err(|e| format!("Failed to link all files: {}", e))?;

        Ok(())
    }
}
