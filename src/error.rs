use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;


#[derive(Debug)]
pub enum AppError {
    Rpc(solana_client::client_error::ClientError),
}


impl From<solana_client::client_error::ClientError> for AppError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        AppError::Rpc(err)
    }
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response { 
        error!("Internal Server Error: {:?}", self);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
            .into_response()
    }
}