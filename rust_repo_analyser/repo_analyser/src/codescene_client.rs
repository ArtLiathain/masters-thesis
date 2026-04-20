use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CodeSceneClient {
    client: Client,
    base_url: String,
    token: String,
}

impl CodeSceneClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    pub async fn get_file_code_health(
        &self,
        project_id: &str,
        repo_name: &str,
        file_extensions: &str,
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut files: HashMap<String, f64> = HashMap::new();
        let mut page = 1;
        let page_size = 500;

        let extensions: Vec<&str> = file_extensions.split(',').collect();
        let prefix_to_strip = format!("{}/", repo_name);

        loop {
            let url = format!(
                "{}/v2/projects/{}/analyses/latest/files?page={}&page_size={}&fields=path,code_health",
                self.base_url, project_id, page, page_size
            );

            eprintln!("[CodeScene] Requesting: {}", url);

            let request = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/json")
                .header("User-Agent", "curl/8.19.0");

            eprintln!("[CodeScene] Request: GET {} HTTP/1.1", url);
            eprintln!(
                "[CodeScene] Request Headers: Authorization: Bearer {}, Accept: application/json, User-Agent: curl/8.19.0",
                if self.token.len() > 20 {
                    format!("{}...", &self.token[..20])
                } else {
                    self.token.clone()
                }
            );

            let response = request.send().await?;

            let status = response.status();
            eprintln!(
                "[CodeScene] Response status: {} {:?}",
                status.as_u16(),
                status.canonical_reason()
            );

            if !status.is_success() {
                let body = response.text().await?;
                eprintln!("[CodeScene] Response body: {}", body);
                return Err(format!("API request failed with status {}: {}", status, body).into());
            }

            let file_response: FilesResponse = response.json().await?;

            if file_response.files.is_empty() {
                break;
            }

            for file in &file_response.files {
                let has_matching_extension = extensions.iter().any(|ext| file.path.ends_with(ext));
                if !has_matching_extension {
                    continue;
                }

                let code_health = file.code_health.as_ref().and_then(|ch| ch.current_score);

                if code_health.is_none() {
                    continue;
                }

                let code_health = code_health.unwrap();

                let stripped_path = file
                    .path
                    .strip_prefix(&prefix_to_strip)
                    .unwrap_or(&file.path)
                    .to_string();

                files.insert(stripped_path, code_health);
            }

            if file_response.files.len() < page_size {
                break;
            }
            page += 1;
        }

        Ok(files)
    }

    pub async fn get_project_id_by_name(&self, project_name: &str) -> Option<String> {
        let url = format!("{}/v2/projects", self.base_url);

        eprintln!("[CodeScene] Requesting: {}", url);
        eprintln!("[CodeScene] Request: GET {} HTTP/1.1", url);
        eprintln!(
            "[CodeScene] Request Headers: Authorization: Bearer {}, Accept: (none)",
            if self.token.len() > 20 {
                format!("{}...", &self.token[..20])
            } else {
                self.token.clone()
            }
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .ok()?;

        let status = response.status();
        eprintln!(
            "[CodeScene] Response status: {} {:?}",
            status.as_u16(),
            status.canonical_reason()
        );

        if !status.is_success() {
            let body = response.text().await.ok()?;
            eprintln!("[CodeScene] Response body: {}", body);
            return None;
        }

        let projects_response: ProjectsResponse = response.json().await.ok()?;

        projects_response
            .projects
            .into_iter()
            .find(|p| p.name == project_name)
            .map(|p| p.id)
    }
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    files: Vec<FileData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct CodeHealthData {
    #[serde(default, deserialize_with = "deserialize_number_or_string")]
    current_score: Option<f64>,
}

fn deserialize_number_or_string<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(f64),
        String(String),
    }

    let value: Option<NumberOrString> = Option::deserialize(deserializer)?;
    Ok(value.and_then(|v| match v {
        NumberOrString::Number(n) => Some(n),
        NumberOrString::String(s) => s.parse::<f64>().ok(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FileData {
    path: String,
    code_health: Option<CodeHealthData>,
    change_frequency: Option<f64>,
    lines_of_code: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileWithRisk {
    pub path: String,
    pub code_health: f64,
    pub is_high_risk: bool,
}

pub fn label_from_code_health(code_health: f64, threshold: f64) -> bool {
    code_health < threshold
}
