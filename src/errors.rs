use log::error;
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub enum AppError {
    AssetNotFound,
    FailedToGetAsset,
    CacheMiss,
    Serde(String),
    Client(String),
    Asset(String),
}

impl warp::reject::Reject for AppError {}

impl std::error::Error for AppError {}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        error!("err: {:?}", err);
        AppError::Serde(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        error!("err: {:?}", err);
        AppError::Client(err.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        error!("err: {:?}", err);
        AppError::Asset(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        error!("err: {:?}", err);
        AppError::Asset(err.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::CacheMiss => write!(f, "Failed to fetch asset from cache"),
            AppError::AssetNotFound => write!(f, "Asset not found"),
            AppError::FailedToGetAsset => write!(f, "Failed to fetch asset"),
            AppError::Serde(_) => write!(f, "Failed to decode response"),
            AppError::Client(_) => write!(f, "Failed to make request"),
            AppError::Asset(_) => write!(f, "Failed to fetch asset"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
    code: i8,
    status_code: u16,
    message: String,
}

pub async fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err.find::<AppError>() {
        let (code, status_code) = match err {
            AppError::AssetNotFound => (-4, StatusCode::NOT_FOUND),

            _ => (-5, StatusCode::INTERNAL_SERVER_ERROR),
        };
        let message = err.to_string();

        let json = warp::reply::json(&ErrorMessage {
            code,
            message,
            status_code: status_code.as_u16(),
        });
        Ok(warp::reply::with_status(json, status_code))
    } else {
        Err(err)
    }
}
