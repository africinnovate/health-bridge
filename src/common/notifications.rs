use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::admin::dtos::NotificationFilters;
use crate::error::AppError;
use crate::models::{NewNotification, Notification, User};
use crate::schema::notifications;
use crate::utils::enums::{NotificationCategoryEnum, Role};

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category: NotificationCategoryEnum,
    pub title: String,
    pub message: String,
    pub related_id: Option<Uuid>,
    pub related_type: Option<String>,
    pub metadata: Option<String>,
    pub is_read: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationListResponse {
    pub data: Vec<NotificationResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarkAsReadResponse {
    pub success: bool,
    pub marked_count: usize,
}

// Create a new notification
pub fn create_notification(
    conn: &mut PgConnection,
    new_notification: NewNotification,
) -> Result<Notification, AppError> {
    let notification = diesel::insert_into(notifications::table)
        .values(&new_notification)
        .returning(Notification::as_returning())
        .get_result::<Notification>(conn)?;

    Ok(notification)
}


// Get paginated notifications
pub fn get_notifications_paginated(
    conn: &mut PgConnection,
    user: &User,
    filters: NotificationFilters,
) -> Result<NotificationListResponse, AppError> {
    use crate::schema::notifications::dsl::*;

    // Build query for counting
    let mut count_query = notifications.into_boxed();

    // Non-admins can only see their own notifications
    if !matches!(user.role, Role::Admin) {
        count_query = count_query.filter(user_id.eq(user.id));
    }

    // Admins can filter by category
    if matches!(user.role, Role::Admin) {
        if let Some(ref filter_category) = filters.category {
            count_query = count_query.filter(category.eq(filter_category));
        }
    }

    // Filter by read status
    if let Some(read_status) = filters.is_read {
        count_query = count_query.filter(is_read.eq(read_status));
    }

    // Search in title or message
    if let Some(ref search_term) = filters.search {
        let search_pattern = format!("%{}%", search_term);
        count_query = count_query.filter(
            title.ilike(search_pattern.clone())
                .or(message.ilike(search_pattern))
        );
    }

    // Get total count
    let total_items = count_query.count().get_result::<i64>(conn)?;

    // Get unread count for the user
    let unread_count = notifications
        .filter(user_id.eq(user.id))
        .filter(is_read.eq(false))
        .count()
        .get_result::<i64>(conn)?;

    // Build query again for fetching data
    let mut data_query = notifications.into_boxed();

    // Apply same filters
    if !matches!(user.role, Role::Admin) {
        data_query = data_query.filter(user_id.eq(user.id));
    }

    if matches!(user.role, Role::Admin) {
        if let Some(filter_category) = filters.category {
            data_query = data_query.filter(category.eq(filter_category));
        }
    }

    if let Some(read_status) = filters.is_read {
        data_query = data_query.filter(is_read.eq(read_status));
    }

    if let Some(search_term) = filters.search {
        let search_pattern = format!("%{}%", search_term);
        data_query = data_query.filter(
            title.ilike(search_pattern.clone())
                .or(message.ilike(search_pattern))
        );
    }

    // Calculate pagination
    let total_pages = (total_items as f64 / filters.page_size as f64).ceil() as i64;
    let offset = (filters.page - 1) * filters.page_size;

    // Get paginated results
    let notification_list = data_query
        .order(created_at.desc())
        .limit(filters.page_size)
        .offset(offset)
        .select(Notification::as_select())
        .load::<Notification>(conn)?;

    // Transform to response format
    let data = notification_list
        .into_iter()
        .map(|n| NotificationResponse {
            id: n.id,
            user_id: n.user_id,
            category: n.category,
            title: n.title,
            message: n.message,
            related_id: n.related_id,
            related_type: n.related_type,
            metadata: n.metadata,
            is_read: n.is_read,
            created_by: n.created_by,
            created_at: n.created_at,
            read_at: n.read_at,
        })
        .collect();

    Ok(NotificationListResponse {
        data,
        pagination: PaginationMeta {
            page: filters.page,
            page_size: filters.page_size,
            total_items,
            total_pages,
            unread_count,
        },
    })
}

// Mark a single notification as read
pub fn mark_notification_as_read(
    conn: &mut PgConnection,
    notification_id: Uuid,
    user: &User,
) -> Result<NotificationResponse, AppError> {
    use crate::schema::notifications::dsl::*;

    // First, check if the notification exists and belongs to the user
    let notification_check = notifications
        .find(notification_id)
        .select(Notification::as_select())
        .first::<Notification>(conn)
        .map_err(|_| AppError::NotFound("Notification not found".into()))?;

    // Non-admins can only mark their own notifications as read
    if !matches!(user.role, Role::Admin) && notification_check.user_id != user.id {
        return Err(AppError::Unauthorized(
            "You can only mark your own notifications as read".into(),
        ));
    }

    // Update the notification
    let updated = diesel::update(notifications.find(notification_id))
        .set((is_read.eq(true), read_at.eq(Some(Utc::now()))))
        .returning(Notification::as_returning())
        .get_result::<Notification>(conn)?;

    Ok(NotificationResponse {
        id: updated.id,
        user_id: updated.user_id,
        category: updated.category,
        title: updated.title,
        message: updated.message,
        related_id: updated.related_id,
        related_type: updated.related_type,
        metadata: updated.metadata,
        is_read: updated.is_read,
        created_by: updated.created_by,
        created_at: updated.created_at,
        read_at: updated.read_at,
    })
}

// Mark all notifications as read for a user
pub fn mark_all_notifications_as_read(
    conn: &mut PgConnection,
    user: &User,
) -> Result<MarkAsReadResponse, AppError> {
    use crate::schema::notifications::dsl::*;

    let marked_count = diesel::update(
        notifications
            .filter(user_id.eq(user.id))
            .filter(is_read.eq(false))
    )
    .set((is_read.eq(true), read_at.eq(Some(Utc::now()))))
    .execute(conn)?;

    Ok(MarkAsReadResponse {
        success: true,
        marked_count,
    })
}

// Helper function to create notification (to be called from other services)
pub fn notify_user(
    conn: &mut PgConnection,
    target_user_id: Uuid,
    category: NotificationCategoryEnum,
    title: String,
    message: String,
    related_id: Option<Uuid>,
    related_type: Option<String>,
    metadata: Option<String>,
    triggered_by: Option<Uuid>,
) -> Result<Notification, AppError> {
    let new_notification = NewNotification {
        user_id: target_user_id,
        category,
        title,
        message,
        related_id,
        related_type,
        metadata,
        created_by: triggered_by,
    };

    create_notification(conn, new_notification)
}