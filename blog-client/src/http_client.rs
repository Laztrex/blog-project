use crate::error::BlogClientError;

use std::time::Duration;

use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{Deserialize, Serialize};

const DEFAULT_API_BASE: &str = "http://localhost:3000";
const REQUEST_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct CreatePostForm {
    title: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct UpdatePostForm {
    title: String,
    content: String,
}

pub struct HttpClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl HttpClient {
    pub fn new(base_url: Option<String>) -> Self {
        // Создаём клиент с тайм-аутом и пулом соединений
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_BASE.to_string()),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    /// auth_headers формирует заголовки с JWT-токеном для авторизованных запросов
    fn auth_headers(&self) -> Result<HeaderMap, BlogClientError> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            let value = format!("Bearer {}", token);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&value)
                    .map_err(|e| BlogClientError::Other(e.to_string()))?,
            );
        }
        Ok(headers)
    }

    // ----- Аутентификация -----
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let url = format!("{}/api/auth/register", self.base_url);
        let form = RegisterForm {
            username,
            email,
            password,
        };
        let resp = self.client.post(&url).json(&form).send().await?;
        let status = resp.status();
        let auth_resp: AuthResponse = resp.json().await?;
        if status == StatusCode::CREATED {
            self.token = Some(auth_resp.token.clone());
            Ok(auth_resp)
        } else {
            Err(BlogClientError::Other("Registration failed".into()))
        }
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let url = format!("{}/api/auth/login", self.base_url);
        let form = LoginForm { username, password };
        let resp = self.client.post(&url).json(&form).send().await?;
        let status = resp.status();
        let auth_resp: AuthResponse = resp.json().await?;
        if status == StatusCode::OK {
            self.token = Some(auth_resp.token.clone());
            Ok(auth_resp)
        } else {
            Err(BlogClientError::Other("Login failed".into()))
        }
    }

    // ----- CRUD постов -----
    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts", self.base_url);
        let form = CreatePostForm { title, content };
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers()?)
            .json(&form)
            .send()
            .await?;
        if resp.status() == StatusCode::CREATED {
            Ok(resp.json().await?)
        } else {
            Err(BlogClientError::Other("Create post failed".into()))
        }
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let resp = self.client.get(&url).send().await?;
        if resp.status() == StatusCode::OK {
            Ok(resp.json().await?)
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(BlogClientError::NotFound("Post not found".into()))
        } else {
            Err(BlogClientError::Other("Get post failed".into()))
        }
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let form = UpdatePostForm { title, content };
        let resp = self
            .client
            .put(&url)
            .headers(self.auth_headers()?)
            .json(&form)
            .send()
            .await?;
        if resp.status() == StatusCode::OK {
            Ok(resp.json().await?)
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(BlogClientError::NotFound("Post not found".into()))
        } else if resp.status() == StatusCode::FORBIDDEN {
            Err(BlogClientError::Unauthorized("Not the author".into()))
        } else {
            Err(BlogClientError::Other("Update failed".into()))
        }
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let resp = self
            .client
            .delete(&url)
            .headers(self.auth_headers()?)
            .send()
            .await?;
        if resp.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(BlogClientError::NotFound("Post not found".into()))
        } else if resp.status() == StatusCode::FORBIDDEN {
            Err(BlogClientError::Unauthorized("Not the author".into()))
        } else {
            Err(BlogClientError::Other("Delete failed".into()))
        }
    }

    pub async fn list_posts(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Post>, i64), BlogClientError> {
        let url = format!(
            "{}/api/posts?limit={}&offset={}",
            self.base_url, limit, offset
        );
        let resp = self.client.get(&url).send().await?;
        #[derive(Deserialize)]
        struct ListResponse {
            posts: Vec<Post>,
            total: i64,
        }
        let list: ListResponse = resp.json().await?;
        Ok((list.posts, list.total))
    }
}
