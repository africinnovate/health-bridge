use chrono::Utc;
use diesel::prelude::*;
use sha1::{Digest, Sha1};
use uuid::Uuid;

use crate::{
    error::AppError,
    handlers::common::UpdateUserSettingsRequest,
    models::{MedicalInfo, NewUserSettings, Patient, User, UserSettings},
    schema::{
        patients::dsl as patients_dsl, user_settings::dsl as settings_dsl, users::dsl as users_dsl,
    },
};

pub struct CloudinaryService {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

impl CloudinaryService {
    pub fn new(config: &crate::config::Config) -> Self {
        Self {
            cloud_name: config.cloudinary_cloud_name.clone(),
            api_key: config.cloudinary_api_key.clone(),
            api_secret: config.cloudinary_api_secret.clone(),
        }
    }

    pub async fn upload_file(
        &self,
        file_bytes: Vec<u8>,
        file_name: &str,
        mime_type: &str,
    ) -> Result<String, AppError> {
        let timestamp = Utc::now().timestamp();

        let params_to_sign = format!("timestamp={}{}", timestamp, self.api_secret);

        let mut hasher = Sha1::new();
        hasher.update(params_to_sign.as_bytes());
        let signature = hex::encode(hasher.finalize());

        let resource_type = if mime_type == "application/pdf" {
            "raw"
        } else {
            "image"
        };

        let url = format!(
            "https://api.cloudinary.com/v1_1/{}/{}/upload",
            self.cloud_name, resource_type
        );

        let client = reqwest::Client::new();
        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .text("api_key", self.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .part("file", part);

        if resource_type == "raw" {
            form = form.text("resource_type", "raw");
        }

        let response = client.post(url).multipart(form).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "Cloudinary upload failed: {}",
                err_text
            )));
        }

        let json: serde_json::Value = response.json().await?;
        let secure_url = json["secure_url"].as_str().ok_or_else(|| {
            AppError::ExternalService("Missing secure_url in Cloudinary response".to_string())
        })?;

        Ok(secure_url.to_string())
    }

    pub async fn upload_image(&self, image_bytes: Vec<u8>) -> Result<String, AppError> {
        self.upload_file(image_bytes, "upload.jpg", "image/jpeg")
            .await
    }
}

pub fn update_user_image(
    conn: &mut PgConnection,
    user_id_: Uuid,
    image_url_: &str,
) -> Result<User, AppError> {
    diesel::update(users_dsl::users.filter(users_dsl::id.eq(user_id_)))
        .set(users_dsl::image_url.eq(Some(image_url_.to_string())))
        .get_result::<User>(conn)
        .map_err(AppError::from)
}

pub fn update_consultation_preference(
    conn: &mut PgConnection,
    user_id_: Uuid,
    preference: crate::utils::enums::ConsultationTypeEnum,
) -> Result<User, AppError> {
    diesel::update(users_dsl::users.filter(users_dsl::id.eq(user_id_)))
        .set(users_dsl::consultation_preference.eq(Some(preference)))
        .get_result::<User>(conn)
        .map_err(AppError::from)
}

pub fn upsert_medical_info(
    conn: &mut PgConnection,
    user_id_: Uuid,
    data: MedicalInfo<'_>,
) -> Result<Patient, AppError> {
    let existing = patients_dsl::patients
        .filter(patients_dsl::user_id.eq(user_id_))
        .first::<Patient>(conn)
        .optional()?;

    let result = match existing {
        Some(_) => {
            diesel::update(patients_dsl::patients.filter(patients_dsl::user_id.eq(user_id_)))
                .set((
                    patients_dsl::blood_type.eq(data.blood_type),
                    patients_dsl::chronic_illnesses.eq(data.chronic_illnesses),
                    patients_dsl::allergies.eq(data.allergies),
                    patients_dsl::medications.eq(data.medications),
                    patients_dsl::existing_conditions.eq(data.existing_conditions),
                    patients_dsl::primary_physician.eq(data.primary_physician),
                    patients_dsl::hmo_number.eq(data.hmo_number),
                    patients_dsl::emergency_contact_name.eq(data.emergency_contact_name),
                    patients_dsl::emergency_contact_phone.eq(data.emergency_contact_phone),
                    patients_dsl::medical_notes.eq(data.medical_notes),
                ))
                .get_result::<Patient>(conn)?
        }
        None => diesel::insert_into(patients_dsl::patients)
            .values(&data)
            .get_result::<Patient>(conn)?,
    };

    Ok(result)
}

pub fn get_or_create_user_settings(
    conn: &mut PgConnection,
    user: &User,
) -> Result<UserSettings, AppError> {
    match settings_dsl::user_settings
        .filter(settings_dsl::user_id.eq(user.id))
        .first::<UserSettings>(conn)
    {
        Ok(settings) => Ok(settings),
        Err(diesel::result::Error::NotFound) => {
            let new_settings = NewUserSettings { user_id: user.id };

            diesel::insert_into(settings_dsl::user_settings)
                .values(&new_settings)
                .get_result::<UserSettings>(conn)
                .map_err(AppError::from)
        }
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn update_user_settings(
    conn: &mut PgConnection,
    user: &User,
    payload: UpdateUserSettingsRequest,
) -> Result<UserSettings, AppError> {
    diesel::update(settings_dsl::user_settings.filter(settings_dsl::user_id.eq(user.id)))
        .set((
            payload
                .appointment_reminders
                .map(|v| settings_dsl::appointment_reminders.eq(v)),
            payload
                .specialist_recommendations
                .map(|v| settings_dsl::specialist_recommendations.eq(v)),
            payload
                .donation_alerts
                .map(|v| settings_dsl::donation_alerts.eq(v)),
            payload
                .account_notifications
                .map(|v| settings_dsl::account_notifications.eq(v)),
            payload
                .email_notifications
                .map(|v| settings_dsl::email_notifications.eq(v)),
            payload
                .sms_notifications
                .map(|v| settings_dsl::sms_notifications.eq(v)),
            payload
                .push_notifications
                .map(|v| settings_dsl::push_notifications.eq(v)),
            payload
                .medical_profile_visibility
                .map(|v| settings_dsl::medical_profile_visibility.eq(v)),
            payload
                .allow_specialists_view_history
                .map(|v| settings_dsl::allow_specialists_view_history.eq(v)),
            payload
                .allow_app_analytics
                .map(|v| settings_dsl::allow_app_analytics.eq(v)),
            payload
                .allow_marketing_notifications
                .map(|v| settings_dsl::allow_marketing_notifications.eq(v)),
            settings_dsl::updated_at.eq(diesel::dsl::now),
        ))
        .get_result::<UserSettings>(conn)
        .map_err(AppError::from)
}
