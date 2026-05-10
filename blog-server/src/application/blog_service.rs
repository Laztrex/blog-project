use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::domain::error::DomainError;
use crate::domain::post::{CreatePost, Post, UpdatePost};

use std::sync::Arc;

pub struct BlogService {
    post_repo: Arc<dyn PostRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl BlogService {
    pub fn new(post_repo: Arc<dyn PostRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            post_repo,
            user_repo,
        }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        input: CreatePost,
    ) -> Result<Post, DomainError> {
        self.user_repo.find_by_id(author_id).await?;
        let post = Post::new(input.title, input.content, author_id);
        self.post_repo.create(&post).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repo.find_by_id(id).await
    }

    pub async fn update_post(
        &self,
        id: i64,
        author_id: i64,
        input: UpdatePost,
    ) -> Result<Post, DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }
        self.post_repo
            .update(id, &input.title, &input.content)
            .await
    }

    pub async fn delete_post(&self, id: i64, author_id: i64) -> Result<(), DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }
        self.post_repo.delete(id).await
    }

    pub async fn list_posts(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        self.post_repo.list(limit, offset).await
    }
}
