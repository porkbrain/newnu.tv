use crate::prelude::*;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;
use std::error::Error as StdError;

#[derive(Debug)]
pub(crate) struct AppError {
    pub(crate) message: Cow<'static, str>,
    pub(crate) kind: AppErrorKind,
    pub(crate) status: StatusCode,
}

#[derive(Default, Debug, Serialize)]
pub(crate) enum AppErrorKind {
    /// Something wrong with the user request, see message for more info.
    BadRequest,
    AlreadyExists,
    NotFound,
    /// Internal server error.
    ///
    /// We don't track the error kind for that error as it's too specific.
    /// Will result in console log.
    #[default]
    Other,
}

impl AppError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::BadRequest,
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub(crate) fn already_exists(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into(),
            kind: AppErrorKind::AlreadyExists,
            status: StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let Self {
            message,
            kind,
            status,
        } = self;

        if matches!(kind, AppErrorKind::Other) {
            error!("500: {message}");
        }

        let body = Json(json!({
            "kind": kind,
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> StdResult<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl StdError for AppError {}

impl From<AnyError> for AppError {
    fn from(err: AnyError) -> Self {
        Self {
            message: format!("{err}").into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<handlebars::RenderError> for AppError {
    fn from(err: handlebars::RenderError) -> Self {
        Self {
            message: format!("handlebars: {err}").into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<tonic::Status> for AppError {
    fn from(err: tonic::Status) -> Self {
        let (kind, status) = match err.code() {
            tonic::Code::InvalidArgument => {
                (AppErrorKind::BadRequest, StatusCode::BAD_REQUEST)
            }
            tonic::Code::NotFound => {
                (AppErrorKind::NotFound, StatusCode::NOT_FOUND)
            }
            _ => (AppErrorKind::Other, StatusCode::INTERNAL_SERVER_ERROR),
        };

        Self {
            message: err.message().to_string().into(),
            kind,
            status,
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self {
            message: format!("rusqlite: {err}").into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            message: format!("serde_json: {err}").into(),
            kind: AppErrorKind::Other,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
