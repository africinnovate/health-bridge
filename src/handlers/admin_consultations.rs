use axum::{extract::{Path, State}, Extension, Json};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        ConsultationType, NewConsultationType, UpdateConsultationType,
        ConsultationBenefit, NewConsultationBenefit, UpdateConsultationBenefit,
        ConsultationTypeBenefit, User,
    },
    schema::{consultation_types, consultation_benefits, consultation_type_benefits},
    utils::{enums::Role, response::{ApiResponse, EmptyData}},
    AppState,
};

use axum::http::StatusCode;

use crate::consultations::service as consultation_service;

// ---- Consultation types CRUD ----

/// List all consultation types
#[utoipa::path(
    get,
    path = "/api/admin/consultation-types",
    responses(
        (status = 200, body = ApiResponse<Vec<ConsultationType>>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn list_consultation_types(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<Vec<ConsultationType>>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let types = consultation_service::list_consultation_types(&mut conn)?;

    Ok(ApiResponse::success(types))
}

/// Create a new consultation type
#[utoipa::path(
    post,
    path = "/api/admin/consultation-types",
    request_body = NewConsultationType,
    responses(
        (status = 201, body = ApiResponse<ConsultationType>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn create_consultation_type(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<NewConsultationType>,
) -> Result<ApiResponse<ConsultationType>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let consultation_type = consultation_service::create_consultation_type(&mut conn, payload)?;

    Ok(ApiResponse::success(consultation_type))
}

/// Update a consultation type
#[utoipa::path(
    put,
    path = "/api/admin/consultation-types/{id}",
    request_body = UpdateConsultationType,
    responses(
        (status = 200, body = ApiResponse<ConsultationType>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn update_consultation_type(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConsultationType>,
) -> Result<ApiResponse<ConsultationType>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let consultation_type = consultation_service::update_consultation_type(&mut conn, id, payload)?;

    Ok(ApiResponse::success(consultation_type))
}

/// Delete a consultation type
#[utoipa::path(
    delete,
    path = "/api/admin/consultation-types/{id}",
    responses(
        (status = 200, body = ApiResponse<EmptyData>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn delete_consultation_type(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    consultation_service::delete_consultation_type(&mut conn, id)?;

    Ok(ApiResponse::message_only(StatusCode::OK, "Consultation type deleted successfully"))
}

// ---- Consultation benefits CRUD ----

/// List all consultation benefits
#[utoipa::path(
    get,
    path = "/api/admin/consultation-benefits",
    responses(
        (status = 200, body = ApiResponse<Vec<ConsultationBenefit>>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn list_consultation_benefits(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<Vec<ConsultationBenefit>>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let benefits = consultation_service::list_consultation_benefits(&mut conn)?;

    Ok(ApiResponse::success(benefits))
}

/// List benefits for a specific consultation type
#[utoipa::path(
    get,
    path = "/api/admin/consultation-types/{id}/benefits",
    responses(
        (status = 200, body = ApiResponse<Vec<ConsultationBenefit>>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn list_type_benefits(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Vec<ConsultationBenefit>>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let benefits = consultation_service::list_type_benefits(&mut conn, id)?;

    Ok(ApiResponse::success(benefits))
}

/// Create a new consultation benefit
#[utoipa::path(
    post,
    path = "/api/admin/consultation-benefits",
    request_body = NewConsultationBenefit,
    responses(
        (status = 201, body = ApiResponse<ConsultationBenefit>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn create_consultation_benefit(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<NewConsultationBenefit>,
) -> Result<ApiResponse<ConsultationBenefit>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let benefit = consultation_service::create_consultation_benefit(&mut conn, payload)?;

    Ok(ApiResponse::success(benefit))
}

/// Update a consultation benefit
#[utoipa::path(
    put,
    path = "/api/admin/consultation-benefits/{id}",
    request_body = UpdateConsultationBenefit,
    responses(
        (status = 200, body = ApiResponse<ConsultationBenefit>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn update_consultation_benefit(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConsultationBenefit>,
) -> Result<ApiResponse<ConsultationBenefit>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let benefit = consultation_service::update_consultation_benefit(&mut conn, id, payload)?;

    Ok(ApiResponse::success(benefit))
}

/// Delete a consultation benefit
#[utoipa::path(
    delete,
    path = "/api/admin/consultation-benefits/{id}",
    responses(
        (status = 200, body = ApiResponse<EmptyData>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn delete_consultation_benefit(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    consultation_service::delete_consultation_benefit(&mut conn, id)?;

    Ok(ApiResponse::message_only(StatusCode::OK, "Consultation benefit deleted successfully"))
}

// ---- Type-Benefit Linking ----

/// Link a benefit to a consultation type
#[utoipa::path(
    post,
    path = "/api/admin/consultation-types/{type_id}/benefits/{benefit_id}",
    responses(
        (status = 200, body = ApiResponse<ConsultationTypeBenefit>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn link_benefit_to_type(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((type_id, benefit_id)): Path<(Uuid, Uuid)>,
) -> Result<ApiResponse<ConsultationTypeBenefit>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let join_record = consultation_service::link_benefit_to_type(&mut conn, type_id, benefit_id)?;

    Ok(ApiResponse::success(join_record))
}

/// Unlink a benefit from a consultation type
#[utoipa::path(
    delete,
    path = "/api/admin/consultation-types/{type_id}/benefits/{benefit_id}",
    responses(
        (status = 200, body = ApiResponse<EmptyData>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn unlink_benefit_from_type(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((type_id, benefit_id)): Path<(Uuid, Uuid)>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    consultation_service::unlink_benefit_from_type(&mut conn, type_id, benefit_id)?;

    Ok(ApiResponse::message_only(StatusCode::OK, "Benefit unlinked successfully"))
}
