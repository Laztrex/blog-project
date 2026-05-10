use crate::error::BlogClientError;

use std::time::Duration;

use tonic::metadata::MetadataValue;
use tonic::Request;

tonic::include_proto!("blog");

const GRPC_TIMEOUT_SECS: u64 = 30;

pub struct GrpcClient {
    client: blog_service_client::BlogServiceClient<tonic::transport::Channel>,
    token: Option<String>,
}

impl GrpcClient {
    /// new создаёт нового gRPC-клиента, подключается к серверу с таймаутом.
    pub async fn new(addr: String) -> Result<Self, BlogClientError> {
        let channel = tonic::transport::Channel::from_shared(addr)
            .map_err(|e| BlogClientError::Other(e.to_string()))?
            .timeout(Duration::from_secs(GRPC_TIMEOUT_SECS))
            .connect()
            .await
            .map_err(BlogClientError::GrpcTransport)?;
        let client = blog_service_client::BlogServiceClient::new(channel);
        Ok(Self {
            client,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    /// add_auth добавляет JWT-токен в метаданные gRPC-запроса, если он установлен.
    fn add_auth<T>(&self, mut req: Request<T>) -> Request<T> {
        if let Some(token) = &self.token {
            let bearer = format!("Bearer {}", token);
            if let Ok(val) = MetadataValue::try_from(&bearer) {
                req.metadata_mut().insert("authorization", val);
            }
        }
        req
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(String, User), BlogClientError> {
        let req = RegisterRequest {
            username,
            email,
            password,
        };
        let resp = self.client.register(req).await?.into_inner();
        self.token = Some(resp.token.clone());
        Ok((resp.token, resp.user.unwrap()))
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<(String, User), BlogClientError> {
        let req = LoginRequest { username, password };
        let resp = self.client.login(req).await?.into_inner();
        self.token = Some(resp.token.clone());
        Ok((resp.token, resp.user.unwrap()))
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<PostResponse, BlogClientError> {
        let req = CreatePostRequest { title, content };
        let req = self.add_auth(Request::new(req));
        let resp = self.client.create_post(req).await?.into_inner();
        Ok(resp)
    }

    pub async fn get_post(&mut self, id: i64) -> Result<PostResponse, BlogClientError> {
        let req = GetPostRequest { id };
        let resp = self.client.get_post(req).await?.into_inner();
        Ok(resp)
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<PostResponse, BlogClientError> {
        let req = UpdatePostRequest { id, title, content };
        let req = self.add_auth(Request::new(req));
        let resp = self.client.update_post(req).await?.into_inner();
        Ok(resp)
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let req = DeletePostRequest { id };
        let req = self.add_auth(Request::new(req));
        self.client.delete_post(req).await?;
        Ok(())
    }

    pub async fn list_posts(
        &mut self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<PostResponse>, i32), BlogClientError> {
        let req = ListPostsRequest { limit, offset };
        let resp = self.client.list_posts(req).await?.into_inner();
        Ok((resp.posts, resp.total))
    }
}
