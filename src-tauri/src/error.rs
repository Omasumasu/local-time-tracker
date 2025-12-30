use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn データベースエラーをシリアライズできる() {
        let error = AppError::NotFound("Task not found".to_string());
        let json = serde_json::to_string(&error);

        assert!(json.is_ok());
        assert!(json.unwrap().contains("Not found"));
    }

    #[test]
    fn 無効入力エラーを作成できる() {
        let error = AppError::InvalidInput("Invalid color format".to_string());

        assert!(error.to_string().contains("Invalid input"));
    }

    #[test]
    fn 既存エラーを作成できる() {
        let error = AppError::AlreadyExists("Entry already running".to_string());

        assert!(error.to_string().contains("Already exists"));
    }

    #[test]
    fn 操作失敗エラーを作成できる() {
        let error = AppError::OperationFailed("Cannot stop non-running entry".to_string());

        assert!(error.to_string().contains("Operation failed"));
    }
}
