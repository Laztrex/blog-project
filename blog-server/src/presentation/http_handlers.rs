use crate::application::{auth_service::AuthService, blog_service::BlogService};
use crate::domain::{
    error::DomainError, post::CreatePost, post::UpdatePost, user::LoginUser, user::RegisterUser,
};
use crate::presentation::middleware::{jwt_validator, AuthenticatedUser};
use actix_cors::Cors;
use actix_web::{web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreatePostForm {
    title: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct UpdatePostForm {
    title: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct PostResponse {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct ListPostsResponse {
    posts: Vec<PostResponse>,
    total: i64,
    limit: i32,
    offset: i32,
}

async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    form: web::Json<RegisterForm>,
) -> impl Responder {
    let input = RegisterUser {
        username: form.username.clone(),
        email: form.email.clone(),
        password: form.password.clone(),
    };
    match auth_service.register(input).await {
        Ok((token, user)) => HttpResponse::Created().json(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            },
        }),
        Err(DomainError::UserAlreadyExists) => HttpResponse::Conflict().body("User already exists"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Registration error: {}", e)),
    }
}

async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    form: web::Json<LoginForm>,
) -> impl Responder {
    let input = LoginUser {
        username: form.username.clone(),
        password: form.password.clone(),
    };
    match auth_service.login(input).await {
        Ok((token, user)) => HttpResponse::Ok().json(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            },
        }),
        Err(DomainError::InvalidCredentials) => {
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
        Err(DomainError::UserNotFound) => HttpResponse::Unauthorized().body("User not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Login error: {}", e)),
    }
}

async fn create_post(
    blog_service: web::Data<Arc<BlogService>>,
    req: HttpRequest,
    form: web::Json<CreatePostForm>,
) -> impl Responder {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .expect("AuthenticatedUser missing – middleware should guarantee this")
        .clone();
    let input = CreatePost {
        title: form.title.clone(),
        content: form.content.clone(),
    };
    match blog_service.create_post(user.user_id, input).await {
        Ok(post) => HttpResponse::Created().json(PostResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }),
        Err(DomainError::UserNotFound) => HttpResponse::NotFound().body("Author not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Create post error: {}", e)),
    }
}

async fn get_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();
    match blog_service.get_post(id).await {
        Ok(post) => HttpResponse::Ok().json(PostResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }),
        Err(DomainError::PostNotFound) => HttpResponse::NotFound().body("Post not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Get post error: {}", e)),
    }
}

async fn update_post(
    blog_service: web::Data<Arc<BlogService>>,
    req: HttpRequest,
    path: web::Path<i64>,
    form: web::Json<UpdatePostForm>,
) -> impl Responder {
    let id = path.into_inner();
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .expect("AuthenticatedUser missing – middleware should guarantee this")
        .clone();
    let input = UpdatePost {
        title: form.title.clone(),
        content: form.content.clone(),
    };
    match blog_service.update_post(id, user.user_id, input).await {
        Ok(post) => HttpResponse::Ok().json(PostResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }),
        Err(DomainError::PostNotFound) => HttpResponse::NotFound().body("Post not found"),
        Err(DomainError::Forbidden) => HttpResponse::Forbidden().body("You are not the author"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Update error: {}", e)),
    }
}

async fn delete_post(
    blog_service: web::Data<Arc<BlogService>>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .expect("AuthenticatedUser missing – middleware should guarantee this")
        .clone();
    match blog_service.delete_post(id, user.user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(DomainError::PostNotFound) => HttpResponse::NotFound().body("Post not found"),
        Err(DomainError::Forbidden) => HttpResponse::Forbidden().body("You are not the author"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Delete error: {}", e)),
    }
}

async fn list_posts(
    blog_service: web::Data<Arc<BlogService>>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    match blog_service.list_posts(limit, offset).await {
        Ok((posts, total)) => {
            let post_responses: Vec<PostResponse> = posts
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
            HttpResponse::Ok().json(ListPostsResponse {
                posts: post_responses,
                total,
                limit,
                offset,
            })
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("List error: {}", e)),
    }
}

pub fn run_http_server(
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<crate::infrastructure::jwt::JwtService>,
    port: u16,
) -> std::io::Result<actix_web::dev::Server> {
    let auth_service = web::Data::new(auth_service);
    let blog_service = web::Data::new(blog_service);
    let jwt_service = web::Data::new(jwt_service);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(auth_service.clone())
            .app_data(blog_service.clone())
            .app_data(jwt_service.clone())
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login)),
            )
            .service(
                web::scope("/api/posts")
                    .route("", web::get().to(list_posts))
                    .route("/{id}", web::get().to(get_post))
                    .service(
                        web::scope("")
                            .wrap(HttpAuthentication::bearer(jwt_validator))
                            .route("", web::post().to(create_post))
                            .route("/{id}", web::put().to(update_post))
                            .route("/{id}", web::delete().to(delete_post)),
                    ),
            )
    })
    .bind(("0.0.0.0", port))?
    .run();
    Ok(server)
}
