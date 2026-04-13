use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SonarClient {
    client: Client,
    base_url: String,
    auth: (String, String),
}

impl SonarClient {
    pub fn new(base_url: &str, username: &str, token: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth: (username.to_string(), token.to_string()),
        }
    }

    pub async fn get_file_technical_debt(
        &self,
        project_key: &str,
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut files: HashMap<String, f64> = HashMap::new();
        let mut page = 1;
        let page_size = 500;

        loop {
            let url = format!(
                "{}/api/components/search?qualifiers=FIL&component={}&ps={}&p={}",
                self.base_url, project_key, page_size, page
            );

            let response = self
                .client
                .get(&url)
                .basic_auth(&self.auth.0, Some(&self.auth.1))
                .send()
                .await?;

            let search_result: ComponentSearchResponse = response.json().await?;

            if search_result.components.is_empty() {
                break;
            }

            for component in &search_result.components {
                let relative_path = component.key.replace(&format!("{}:", project_key), "");
                let is_cpp_file = relative_path.ends_with(".cpp")
                    || relative_path.ends_with(".h")
                    || relative_path.ends_with(".hpp")
                    || relative_path.ends_with(".cc")
                    || relative_path.ends_with(".cxx");
                if !is_cpp_file {
                    continue;
                }

                if let Some(td) = self.get_file_td(project_key, &component.key).await {
                    files.insert(relative_path, td);
                }
            }

            if search_result.components.len() < page_size {
                break;
            }
            page += 1;
        }

        Ok(files)
    }

    async fn get_file_td(
        &self,
        _project_key: &str,
        component_key: &str,
    ) -> Option<f64> {
        let url = format!(
            "{}/api/measures/component?component={}&metricKeys=sqale_index",
            self.base_url, component_key
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.auth.0, Some(&self.auth.1))
            .send()
            .await
            .ok()?;

        let measures_response: MeasuresResponse = response.json().await.ok()?;

        measures_response.component.measures.first().map(|m| {
            m.value.parse::<f64>().unwrap_or(0.0) / 60.0
        })
    }

    pub async fn wait_for_quality_gate(
        &self,
        project_key: &str,
        max_attempts: usize,
        delay_ms: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        for _ in 0..max_attempts {
            let url = format!("{}/api/analysis_status?component={}", self.base_url, project_key);

            let response = self
                .client
                .get(&url)
                .basic_auth(&self.auth.0, Some(&self.auth.1))
                .send()
                .await?;

            let status: AnalysisStatus = response.json().await?;

            if status.project_status.status == "FINAL" || status.project_status.status == "OK" {
                return Ok(status.project_status.status);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
        }

        Err("Timeout waiting for analysis to complete".into())
    }
}

#[derive(Debug, Deserialize)]
struct ComponentSearchResponse {
    components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
struct Component {
    key: String,
    name: String,
    qualifier: String,
}

#[derive(Debug, Deserialize)]
struct MeasuresResponse {
    component: ComponentMeasures,
}

#[derive(Debug, Deserialize)]
struct ComponentMeasures {
    measures: Vec<Measure>,
}

#[derive(Debug, Deserialize)]
struct Measure {
    metric: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct AnalysisStatus {
    #[serde(rename = "projectStatus")]
    project_status: ProjectStatus,
}

#[derive(Debug, Deserialize)]
struct ProjectStatus {
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileWithRisk {
    pub path: String,
    pub technical_debt_minutes: f64,
    pub is_high_risk: bool,
}

pub fn label_from_td(technical_debt_minutes: f64, threshold_minutes: i64) -> bool {
    technical_debt_minutes >= threshold_minutes as f64
}