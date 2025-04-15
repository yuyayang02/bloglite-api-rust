use axum::http::{HeaderMap, HeaderValue};
use reqwest::header;
use serde::Serialize;

use crate::domain::articles;

#[derive(Clone)]
pub struct GithubArticleContentRender {
    client: reqwest::Client,
}

impl Default for GithubArticleContentRender {
    fn default() -> Self {
        Self::new(std::env::var("MARKDOWN_RENDER_GITHUB_KEY").unwrap())
    }
}

impl GithubArticleContentRender {
    pub fn new<T: AsRef<str>>(token: T) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("bloglite-markdown-render")
            .default_headers({
                let mut header = HeaderMap::new();
                header.insert(
                    header::ACCEPT,
                    HeaderValue::from_static("application/vnd.github+json"),
                );
                header.insert(
                    "X-GitHub-Api-Version",
                    HeaderValue::from_static("2022-11-28"),
                );
                header.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token.as_ref())).unwrap(),
                );
                header
            })
            .build()
            .unwrap();

        Self { client }
    }
}

#[derive(Serialize)]
struct RequestBody {
    text: String,
    mode: String,
}

impl articles::content::ContentRender for GithubArticleContentRender {
    async fn render<T: AsRef<str>>(&self, content: T) -> Result<String, articles::content::Error> {
        let resp = self
            .client
            .post("https://api.github.com/markdown")
            .json(&RequestBody {
                text: content.as_ref().to_string(),
                mode: "gfm".to_string(),
            })
            .send()
            .await
            .map_err(|e| articles::content::Error::RenderError(e.to_string()))?;

        resp.text()
            .await
            .map_err(|e| articles::content::Error::RenderError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {

    use crate::domain::articles::content::ContentRender;

    use super::*;

    #[tokio::test]
    async fn test_render() {
        let render = GithubArticleContentRender::default();

        println!("{:?}", render.render("content").await);
    }
}
