use crate::application::{auth_service::AuthService, blog_service::BlogService};
use crate::domain::{
    error::DomainError, post::CreatePost, post::UpdatePost, user::LoginUser, user::RegisterUser,
};
use crate::infrastructure::jwt::JwtService;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

pub mod blog {
    tonic::include_proto!("blog");
}

use blog::{
    blog_service_server::{BlogService as GrpcBlogService, BlogServiceServer},
    AuthResponse, CreatePostRequest, DeletePostRequest, Empty, GetPostRequest, ListPostsRequest,
    ListPostsResponse, LoginRequest, PostResponse, RegisterRequest, UpdatePostRequest, User,
};

pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

#[tonic::async_trait]
impl GrpcBlogService for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let input = RegisterUser {
            username: req.username,
            email: req.email,
            password: req.password,
        };
        match self.auth_service.register(input).await {
            Ok((token, user)) => Ok(Response::new(AuthResponse {
                token,
                user: Some(User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::UserAlreadyExists) => {
                Err(Status::already_exists("User already exists"))
            }
            Err(e) => Err(Status::internal(format!("Registration error: {}", e))),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let input = LoginUser {
            username: req.username,
            password: req.password,
        };
        match self.auth_service.login(input).await {
            Ok((token, user)) => Ok(Response::new(AuthResponse {
                token,
                user: Some(User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::InvalidCredentials) => {
                Err(Status::unauthenticated("Invalid credentials"))
            }
            Err(DomainError::UserNotFound) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(format!("Login error: {}", e))),
        }
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self
            .extract_user_id(&request)
            .ok_or_else(|| Status::unauthenticated("Missing or invalid token"))?;
        let req = request.into_inner();
        let input = CreatePost {
            title: req.title,
            content: req.content,
        };
        match self.blog_service.create_post(user_id, input).await {
            Ok(post) => Ok(Response::new(PostResponse {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            })),
            Err(DomainError::UserNotFound) => Err(Status::not_found("Author not found")),
            Err(e) => Err(Status::internal(format!("Create post error: {}", e))),
        }
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let id = request.into_inner().id;
        match self.blog_service.get_post(id).await {
            Ok(post) => Ok(Response::new(PostResponse {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            })),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(e) => Err(Status::internal(format!("Get post error: {}", e))),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self
            .extract_user_id(&request)
            .ok_or_else(|| Status::unauthenticated("Missing or invalid token"))?;
        let req = request.into_inner();
        let input = UpdatePost {
            title: req.title,
            content: req.content,
        };
        match self.blog_service.update_post(req.id, user_id, input).await {
            Ok(post) => Ok(Response::new(PostResponse {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            })),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(DomainError::Forbidden) => Err(Status::permission_denied("Not the author")),
            Err(e) => Err(Status::internal(format!("Update error: {}", e))),
        }
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<Empty>, Status> {
        let user_id = self
            .extract_user_id(&request)
            .ok_or_else(|| Status::unauthenticated("Missing or invalid token"))?;
        let id = request.into_inner().id;
        match self.blog_service.delete_post(id, user_id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(DomainError::Forbidden) => Err(Status::permission_denied("Not the author")),
            Err(e) => Err(Status::internal(format!("Delete error: {}", e))),
        }
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        let limit = req.limit;
        let offset = req.offset;
        match self.blog_service.list_posts(limit, offset).await {
            Ok((posts, total)) => {
                let grpc_posts = posts
                    .into_iter()
                    .map(|p| PostResponse {
                        id: p.id,
                        title: p.title,
                        content: p.content,
                        author_id: p.author_id,
                        created_at: p.created_at.to_rfc3339(),
                        updated_at: p.updated_at.to_rfc3339(),
                    })
                    .collect();
                Ok(Response::new(ListPostsResponse {
                    posts: grpc_posts,
                    total: total as i32,
                    limit,
                    offset,
                }))
            }
            Err(e) => Err(Status::internal(format!("List error: {}", e))),
        }
    }
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    /// extract_user_id извлекает user_id из JWT-токена, переданного в gRPC метаданных.
    /// Ожидается заголовок `authorization: Bearer <token>`.
    /// Возвращает `Some(user_id)` или `None` при отсутствии/невалидности токена.
    fn extract_user_id<T>(&self, request: &Request<T>) -> Option<i64> {
        let metadata = request.metadata();
        let auth_header = metadata.get("authorization")?;
        let auth_str = auth_header.to_str().ok()?;
        if !auth_str.starts_with("Bearer ") {
            return None;
        }
        let token = &auth_str[7..];
        let claims = self.jwt_service.verify_token(token).ok()?;
        Some(claims.sub)
    }
}

pub fn run_grpc_server(
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
    port: u16,
) -> Result<
    impl std::future::Future<Output = Result<(), tonic::transport::Error>>,
    Box<dyn std::error::Error>,
> {
    let grpc_service = BlogGrpcService::new(auth_service, blog_service, jwt_service);
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let server = async move {
        Server::builder()
            .add_service(BlogServiceServer::new(grpc_service))
            .serve(addr)
            .await
    };
    Ok(server)
}
