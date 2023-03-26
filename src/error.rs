use reqwest::header::InvalidHeaderValue;
use std::panic::Location;
use thiserror::Error;

pub type AppResult<T = ()> = anyhow::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Reqwest: {error}; Location: {location}")]
    Reqwest {
        error: reqwest::Error,
        location: &'static Location<'static>,
    },
    #[error("Dotenvy: {error}; Location: {location}")]
    Dotenvy {
        error: dotenvy::Error,
        location: &'static Location<'static>,
    },
    #[error("ParseError: {error}; Location: {location}")]
    ParseError {
        error: chrono::format::ParseError,
        location: &'static Location<'static>,
    },
    #[error("SerdeJsonError: {error}; Location: {location}")]
    SerdeJsonError {
        error: serde_json::Error,
        location: &'static Location<'static>,
    },
    #[error("InvalidHeaderValue: {error}; Location: {location}")]
    InvalidHeaderValue {
        error: InvalidHeaderValue,
        location: &'static Location<'static>,
    },
}

impl From<reqwest::Error> for AppError {
    #[track_caller]
    fn from(error: reqwest::Error) -> Self {
        AppError::Reqwest {
            error,
            location: Location::caller(),
        }
    }
}
impl From<dotenvy::Error> for AppError {
    #[track_caller]
    fn from(error: dotenvy::Error) -> Self {
        AppError::Dotenvy {
            error,
            location: Location::caller(),
        }
    }
}
impl From<chrono::format::ParseError> for AppError {
    #[track_caller]
    fn from(error: chrono::format::ParseError) -> Self {
        AppError::ParseError {
            error,
            location: Location::caller(),
        }
    }
}
impl From<serde_json::Error> for AppError {
    #[track_caller]
    fn from(error: serde_json::Error) -> Self {
        AppError::SerdeJsonError {
            error,
            location: Location::caller(),
        }
    }
}
impl From<InvalidHeaderValue> for AppError {
    #[track_caller]
    fn from(error: InvalidHeaderValue) -> Self {
        AppError::InvalidHeaderValue {
            error,
            location: Location::caller(),
        }
    }
}
