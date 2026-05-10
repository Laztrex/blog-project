use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP transport error: {0}")]
    Http(reqwest::Error),
    #[error("gRPC status error: {0}")]
    Grpc(Box<tonic::Status>),
    #[error("gRPC transport error: {0}")]
    GrpcTransport(tonic::transport::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for BlogClientError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_status() {
            if let Some(status) = e.status() {
                if status == 401 {
                    return BlogClientError::Unauthorized("Invalid token".into());
                }
                if status == 404 {
                    return BlogClientError::NotFound("Resource not found".into());
                }
            }
        }
        BlogClientError::Http(e)
    }
}

impl From<tonic::Status> for BlogClientError {
    fn from(s: tonic::Status) -> Self {
        match s.code() {
            tonic::Code::Unauthenticated => BlogClientError::Unauthorized(s.message().into()),
            tonic::Code::NotFound => BlogClientError::NotFound(s.message().into()),
            _ => BlogClientError::Grpc(Box::new(s)),
        }
    }
}
