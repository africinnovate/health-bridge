use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::{AppState, error::AppError};

