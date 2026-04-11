use axum::extract::{Path, State};
use axum::{Extension, Json};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    ConsultationBenefit, ConsultationPackage, ConsultationPackageBenefit,
    ConsultationPackageResponse, CreatePackageRequest, NewConsultationPackage,
    NewConsultationPackageBenefit, ResolvedBenefit, User, Specialist,
    ConsultationType
};
use crate::schema::{
    consultation_benefits, consultation_package_benefits, consultation_packages, specialists,
    consultation_types
};
use crate::utils::response::ApiResponse;
use crate::AppState;

/// Create a new consultation package
#[utoipa::path(
    post,
    path = "/api/specialists/packages",
    request_body = CreatePackageRequest,
    responses(
        (status = 201, body = ApiResponse<ConsultationPackageResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn create_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreatePackageRequest>,
) -> Result<ApiResponse<ConsultationPackageResponse>, AppError> {
    let mut conn = state.pool.get()?;

    // Get specialist record
    let specialist = specialists::table
        .filter(specialists::user_id.eq(user.id))
        .first::<Specialist>(&mut conn)
        .map_err(|_| AppError::Forbidden("Specialist profile not found".into()))?;

    conn.transaction::<_, AppError, _>(|conn| {
        // 1. Create the package
        let new_package = NewConsultationPackage {
            specialist_id: specialist.id,
            consultation_type_id: payload.consultation_type_id,
            name: payload.name,
            description: payload.description,
            custom_price: payload.custom_price,
            custom_duration_minutes: payload.custom_duration_minutes,
        };

        let package = diesel::insert_into(consultation_packages::table)
            .values(&new_package)
            .get_result::<ConsultationPackage>(conn)?;

        // 2. Add benefits
        for benefit_input in payload.benefits {
            let new_benefit = NewConsultationPackageBenefit {
                package_id: package.id,
                consultation_benefit_id: benefit_input.consultation_benefit_id,
                custom_title: benefit_input.custom_title,
                custom_description: benefit_input.custom_description,
            };

            diesel::insert_into(consultation_package_benefits::table)
                .values(&new_benefit)
                .execute(conn)?;
        }

        let response = get_resolved_package(conn, package)?;
        Ok(ApiResponse::success(response))
    })
}

/// List specialist's consultation packages
#[utoipa::path(
    get,
    path = "/api/specialists/packages",
    responses(
        (status = 200, body = ApiResponse<Vec<ConsultationPackageResponse>>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn list_packages(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<Vec<ConsultationPackageResponse>>, AppError> {
    let mut conn = state.pool.get()?;

    let specialist = specialists::table
        .filter(specialists::user_id.eq(user.id))
        .first::<Specialist>(&mut conn)
        .map_err(|_| AppError::Forbidden("Specialist profile not found".into()))?;

    let packages = consultation_packages::table
        .filter(consultation_packages::specialist_id.eq(specialist.id))
        .load::<ConsultationPackage>(&mut conn)?;

    let mut response = Vec::new();
    for package in packages {
        response.push(get_resolved_package(&mut conn, package)?);
    }

    Ok(ApiResponse::success(response))
}

/// Get a specific consultation package
#[utoipa::path(
    get,
    path = "/api/specialists/packages/{id}",
    params(
        ("id" = Uuid, Path, description = "Package ID")
    ),
    responses(
        (status = 200, body = ApiResponse<ConsultationPackageResponse>),
        (status = 404, description = "Package not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn get_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(package_id): Path<Uuid>,
) -> Result<ApiResponse<ConsultationPackageResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let specialist = specialists::table
        .filter(specialists::user_id.eq(user.id))
        .first::<Specialist>(&mut conn)
        .map_err(|_| AppError::Forbidden("Specialist profile not found".into()))?;

    let package = consultation_packages::table
        .filter(consultation_packages::id.eq(package_id))
        .filter(consultation_packages::specialist_id.eq(specialist.id))
        .first::<ConsultationPackage>(&mut conn)
        .map_err(|_| AppError::NotFound("Package not found".into()))?;

    let response = get_resolved_package(&mut conn, package)?;
    Ok(ApiResponse::success(response))
}

/// Update a consultation package
#[utoipa::path(
    put,
    path = "/api/specialists/packages/{id}",
    request_body = CreatePackageRequest,
    params(
        ("id" = Uuid, Path, description = "Package ID")
    ),
    responses(
        (status = 200, body = ApiResponse<ConsultationPackageResponse>),
        (status = 404, description = "Package not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn update_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(package_id): Path<Uuid>,
    Json(payload): Json<CreatePackageRequest>,
) -> Result<ApiResponse<ConsultationPackageResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let specialist = specialists::table
        .filter(specialists::user_id.eq(user.id))
        .first::<Specialist>(&mut conn)
        .map_err(|_| AppError::Forbidden("Specialist profile not found".into()))?;

    conn.transaction::<_, AppError, _>(|conn| {
        let package = diesel::update(consultation_packages::table)
            .filter(consultation_packages::id.eq(package_id))
            .filter(consultation_packages::specialist_id.eq(specialist.id))
            .set((
                consultation_packages::consultation_type_id.eq(payload.consultation_type_id),
                consultation_packages::name.eq(payload.name),
                consultation_packages::description.eq(payload.description),
                consultation_packages::custom_price.eq(payload.custom_price),
                consultation_packages::custom_duration_minutes.eq(payload.custom_duration_minutes),
                consultation_packages::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<ConsultationPackage>(conn)
            .map_err(|_| AppError::NotFound("Package not found".into()))?;

        // Replace benefits
        diesel::delete(consultation_package_benefits::table.filter(consultation_package_benefits::package_id.eq(package.id)))
            .execute(conn)?;

        for benefit_input in payload.benefits {
            let new_benefit = NewConsultationPackageBenefit {
                package_id: package.id,
                consultation_benefit_id: benefit_input.consultation_benefit_id,
                custom_title: benefit_input.custom_title,
                custom_description: benefit_input.custom_description,
            };

            diesel::insert_into(consultation_package_benefits::table)
                .values(&new_benefit)
                .execute(conn)?;
        }

        let response = get_resolved_package(conn, package)?;
        Ok(ApiResponse::success(response))
    })
}

/// Delete a consultation package
#[utoipa::path(
    delete,
    path = "/api/specialists/packages/{id}",
    params(
        ("id" = Uuid, Path, description = "Package ID")
    ),
    responses(
        (status = 200, body = ApiResponse<bool>),
        (status = 404, description = "Package not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn delete_package(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(package_id): Path<Uuid>,
) -> Result<ApiResponse<bool>, AppError> {
    let mut conn = state.pool.get()?;

    let specialist = specialists::table
        .filter(specialists::user_id.eq(user.id))
        .first::<Specialist>(&mut conn)
        .map_err(|_| AppError::Forbidden("Specialist profile not found".into()))?;

    let deleted = diesel::delete(consultation_packages::table)
        .filter(consultation_packages::id.eq(package_id))
        .filter(consultation_packages::specialist_id.eq(specialist.id))
        .execute(&mut conn)?;

    if deleted == 0 {
        return Err(AppError::NotFound("Package not found".into()));
    }

    Ok(ApiResponse::success(true))
}

// Helper to resolve benefits and fallback price/duration
fn get_resolved_package(
    conn: &mut PgConnection,
    mut package: ConsultationPackage,
) -> Result<ConsultationPackageResponse, AppError> {
    // 1. Resolve Fallbacks
    if package.custom_price.is_none() || package.custom_duration_minutes.is_none() {
        let base_type = consultation_types::table
            .find(package.consultation_type_id)
            .first::<ConsultationType>(conn)
            .map_err(|_| AppError::NotFound("Consultation type not found".into()))?;

        if package.custom_price.is_none() {
            package.custom_price = Some(base_type.base_price);
        }
        if package.custom_duration_minutes.is_none() {
            package.custom_duration_minutes = Some(base_type.duration_minutes);
        }
    }

    // 2. Resolve Benefits
    let package_benefits = consultation_package_benefits::table
        .filter(consultation_package_benefits::package_id.eq(package.id))
        .load::<ConsultationPackageBenefit>(conn)?;

    let mut resolved_benefits = Vec::new();

    for pb in package_benefits {
        if let Some(global_id) = pb.consultation_benefit_id {
            let global_benefit = consultation_benefits::table
                .find(global_id)
                .first::<ConsultationBenefit>(conn)?;
            
            resolved_benefits.push(ResolvedBenefit {
                id: pb.id,
                title: global_benefit.title,
                description: global_benefit.description,
                is_custom: false,
                global_benefit_id: Some(global_id),
            });
        } else {
            resolved_benefits.push(ResolvedBenefit {
                id: pb.id,
                title: pb.custom_title.unwrap_or_else(|| "Untitled Benefit".into()),
                description: pb.custom_description,
                is_custom: true,
                global_benefit_id: None,
            });
        }
    }

    Ok(ConsultationPackageResponse {
        package,
        benefits: resolved_benefits,
    })
}
