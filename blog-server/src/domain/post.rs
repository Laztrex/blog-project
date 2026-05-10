use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    pub fn new(title: String, content: String, author_id: i64) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            title,
            content,
            author_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_post_new() {
        let post = Post::new("Title".to_string(), "Content".to_string(), 42);
        assert_eq!(post.title, "Title");
        assert_eq!(post.content, "Content");
        assert_eq!(post.author_id, 42);
        assert_eq!(post.id, 0);
        assert!(post.created_at <= Utc::now());
        assert!(post.updated_at <= Utc::now());
    }
}
