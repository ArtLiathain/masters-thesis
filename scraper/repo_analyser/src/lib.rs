pub mod file_graph;
pub mod git_analyzer;

pub use file_graph::{ChangedFile, Edge, FileGraph, FileNode};
pub use git_analyzer::GitAnalyzer;

use std::fs::File;
use std::io::Write;

pub fn save_graph_to_json(graph: &FileGraph, output_path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(graph)
        .map_err(|e| format!("Failed to serialize graph: {}", e))?;

    let mut file =
        File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;

    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}
