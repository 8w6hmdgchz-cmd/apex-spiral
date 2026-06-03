use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct RepositoryManager {
    owner: String,
    repo: String,
    token: Option<String>,
    client: Client,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateFileRequest {
    pub message: String,
    pub content: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateFileRequest {
    pub message: String,
    pub content: String,
    pub sha: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitResponse {
    pub sha: String,
    pub node_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit: BranchCommit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BranchCommit {
    pub sha: String,
    pub url: String,
}

impl RepositoryManager {
    pub fn new(repository: &str, token: Option<&str>) -> Result<Self> {
        let parts: Vec<&str> = repository.split('/').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid repository format. Expected 'owner/repo'"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
            token: token.map(String::from),
            client,
        })
    }

    fn get_base_url(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.repo)
    }

    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());
        headers.insert("User-Agent", "NanoGPT-Claw".parse().unwrap());

        if let Some(ref token) = self.token {
            headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        headers
    }

    pub async fn get_file(&self, path: &str, ref_: Option<&str>) -> Result<FileContent> {
        let mut url = format!("{}/contents/{}", self.get_base_url(), path);

        if let Some(ref_) = ref_ {
            url = format!("{}?ref={}", url, ref_);
        }

        let response = self.client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await
            .context("Failed to get file")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to get file: {} - {}", status, error_text);
            return Err(anyhow!("Failed to get file: {}", status));
        }

        let content: FileContent = response.json().await
            .context("Failed to parse file response")?;

        debug!("Got file: {}", path);
        Ok(content)
    }

    pub async fn create_file(&self, path: &str, request: CreateFileRequest) -> Result<CommitResponse> {
        let url = format!("{}/contents/{}", self.get_base_url(), path);

        let body = serde_json::json!({
            "message": request.message,
            "content": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, request.content),
            "branch": request.branch
        });

        let response = self.client
            .put(&url)
            .headers(self.get_headers())
            .json(&body)
            .send()
            .await
            .context("Failed to create file")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to create file: {} - {}", status, error_text);
            return Err(anyhow!("Failed to create file: {}", status));
        }

        let commit: CommitResponse = response.json().await
            .context("Failed to parse commit response")?;

        info!("Created file: {}", path);
        Ok(commit)
    }

    pub async fn update_file(&self, path: &str, request: UpdateFileRequest) -> Result<CommitResponse> {
        let url = format!("{}/contents/{}", self.get_base_url(), path);

        let body = serde_json::json!({
            "message": request.message,
            "content": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, request.content),
            "sha": request.sha,
            "branch": request.branch
        });

        let response = self.client
            .put(&url)
            .headers(self.get_headers())
            .json(&body)
            .send()
            .await
            .context("Failed to update file")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to update file: {} - {}", status, error_text);
            return Err(anyhow!("Failed to update file: {}", status));
        }

        let commit: CommitResponse = response.json().await
            .context("Failed to parse commit response")?;

        info!("Updated file: {}", path);
        Ok(commit)
    }

    pub async fn delete_file(&self, path: &str, sha: &str, message: &str, branch: Option<&str>) -> Result<()> {
        let url = format!("{}/contents/{}", self.get_base_url(), path);

        let body = serde_json::json!({
            "message": message,
            "sha": sha,
            "branch": branch
        });

        let response = self.client
            .delete(&url)
            .headers(self.get_headers())
            .json(&body)
            .send()
            .await
            .context("Failed to delete file")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to delete file: {} - {}", status, error_text);
            return Err(anyhow!("Failed to delete file: {}", status));
        }

        info!("Deleted file: {}", path);
        Ok(())
    }

    pub async fn get_default_branch(&self) -> Result<String> {
        let url = self.get_base_url();

        let response = self.client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await
            .context("Failed to get repository info")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get repository: {}", response.status()));
        }

        let repo: serde_json::Value = response.json().await?;

        repo["default_branch"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow!("No default branch found"))
    }

    pub async fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let url = format!("{}/branches", self.get_base_url());

        let response = self.client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await
            .context("Failed to list branches")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list branches: {}", response.status()));
        }

        let branches: Vec<BranchInfo> = response.json().await
            .context("Failed to parse branches response")?;

        Ok(branches)
    }
}
