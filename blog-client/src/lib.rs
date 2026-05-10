pub mod error;
pub mod grpc_client;
pub mod http_client;

pub use error::BlogClientError;
pub use grpc_client::GrpcClient;
pub use http_client::{AuthResponse, HttpClient, Post, User};

#[derive(Debug, Clone)]
pub enum Transport {
    Http(Option<String>),
    Grpc(String),
}

pub struct BlogClient {
    pub http: Option<HttpClient>,
    pub grpc: Option<GrpcClient>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let (http, grpc) = match &transport {
            Transport::Http(base_url) => {
                let client = HttpClient::new(base_url.clone());
                (Some(client), None)
            }
            Transport::Grpc(addr) => {
                let client = GrpcClient::new(addr.clone()).await?;
                (None, Some(client))
            }
        };
        Ok(Self { http, grpc })
    }

    fn ensure_http(&self) -> Result<&HttpClient, BlogClientError> {
        self.http
            .as_ref()
            .ok_or_else(|| BlogClientError::Other("Not an HTTP transport".into()))
    }

    fn ensure_http_mut(&mut self) -> Result<&mut HttpClient, BlogClientError> {
        self.http
            .as_mut()
            .ok_or_else(|| BlogClientError::Other("Not an HTTP transport".into()))
    }

    fn ensure_grpc_mut(&mut self) -> Result<&mut GrpcClient, BlogClientError> {
        self.grpc
            .as_mut()
            .ok_or_else(|| BlogClientError::Other("Not a gRPC transport".into()))
    }

    pub fn set_token(&mut self, token: String) {
        if let Some(http) = &mut self.http {
            http.set_token(token.clone());
        }
        if let Some(grpc) = &mut self.grpc {
            grpc.set_token(token);
        }
    }

    pub fn get_token(&self) -> Option<String> {
        if let Some(http) = &self.http {
            http.get_token().cloned()
        } else if let Some(grpc) = &self.grpc {
            grpc.get_token().cloned()
        } else {
            None
        }
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        if let Ok(http) = self.ensure_http_mut() {
            http.register(username, email, password).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let (token, user) = grpc.register(username, email, password).await?;
            Ok(AuthResponse {
                token,
                user: user.into(),
            })
        } else {
            unreachable!()
        }
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        if let Ok(http) = self.ensure_http_mut() {
            http.login(username, password).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let (token, user) = grpc.login(username, password).await?;
            Ok(AuthResponse {
                token,
                user: user.into(),
            })
        } else {
            unreachable!()
        }
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        if let Ok(http) = self.ensure_http() {
            http.create_post(title, content).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let resp = grpc.create_post(title, content).await?;
            Ok(Post {
                id: resp.id,
                title: resp.title,
                content: resp.content,
                author_id: resp.author_id,
                created_at: resp.created_at,
                updated_at: resp.updated_at,
            })
        } else {
            unreachable!()
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        if let Ok(http) = self.ensure_http() {
            http.get_post(id).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let resp = grpc.get_post(id).await?;
            Ok(Post {
                id: resp.id,
                title: resp.title,
                content: resp.content,
                author_id: resp.author_id,
                created_at: resp.created_at,
                updated_at: resp.updated_at,
            })
        } else {
            unreachable!()
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        if let Ok(http) = self.ensure_http() {
            http.update_post(id, title, content).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let resp = grpc.update_post(id, title, content).await?;
            Ok(Post {
                id: resp.id,
                title: resp.title,
                content: resp.content,
                author_id: resp.author_id,
                created_at: resp.created_at,
                updated_at: resp.updated_at,
            })
        } else {
            unreachable!()
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        if let Ok(http) = self.ensure_http() {
            http.delete_post(id).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            grpc.delete_post(id).await
        } else {
            unreachable!()
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Post>, i64), BlogClientError> {
        if let Ok(http) = self.ensure_http() {
            http.list_posts(limit, offset).await
        } else if let Ok(grpc) = self.ensure_grpc_mut() {
            let (posts, total) = grpc.list_posts(limit, offset).await?;
            let mapped = posts
                .into_iter()
                .map(|p| Post {
                    id: p.id,
                    title: p.title,
                    content: p.content,
                    author_id: p.author_id,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                })
                .collect();
            Ok((mapped, total as i64))
        } else {
            unreachable!()
        }
    }
}

impl From<grpc_client::User> for http_client::User {
    fn from(u: grpc_client::User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }
    }
}
