use diesel::prelude::*;
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

// Use the crate name with underscores replaced by hyphens
use health_bridge::{
    models::{MedicalInfo, NewUser}, 
    utils::enums::ConsultationTypeEnum,
};
use health_bridge::schema::{
    users, 
    patients, 
    hospitals,
    specialties,
    specialists,
    specialist_availabilities,
    appointments,
    blood_requests
};
use health_bridge::utils::enums::{Gender, Role};

fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("failed to hash password: {}", e))?
        .to_string();
    Ok(password_hash)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let mut conn = PgConnection::establish(&database_url)
        .expect("Error connecting to database");

    println!("Starting database seeding...");

    diesel::delete(appointments::table).execute(&mut conn)?;
    diesel::delete(patients::table).execute(&mut conn)?;
    diesel::delete(users::table).execute(&mut conn)?;
    println!("✓ Cleared existing data");

    // Hash password once for all users
    let password_hash = hash_password("password")?;

    // 1. Create Patient User
    let patient_user_id = Uuid::new_v4();
    let patient_user = NewUser {
        first_name: "John",
        last_name: "Patient",
        email: "patient@mail.com",
        phone: Some("+2348012345671"),
        gender: Some(Gender::Male),
        dob: Some(chrono::NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
        password_hash: &password_hash,
        role: Role::Patient,
    };

    diesel::insert_into(users::table)
        .values(&patient_user)
        .on_conflict(users::email)
        .do_nothing()
        .execute(&mut conn)?;

    // Update with correct ID
    diesel::update(users::table.filter(users::email.eq("patient@mail.com")))
        .set(users::id.eq(patient_user_id))
        .execute(&mut conn)?;

    // Set email as verified
    diesel::update(users::table.filter(users::id.eq(patient_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    // Add patient medical info
    let patient_medical = MedicalInfo {
        user_id: patient_user_id,
        blood_type: Some("O+"),
        chronic_illnesses: Some("None"),
        allergies: Some("Penicillin"),
        hmo_number: Some("HMO123456"),
        emergency_contact_name: Some("Jane Patient"),
        emergency_contact_phone: Some("+2348012345672"),
        medical_notes: Some("Regular checkups needed"),
    };

    diesel::insert_into(patients::table)
        .values(&patient_medical)
        .execute(&mut conn)?;

    println!("✓ Created Patient user");

    // 2. Create Donor User
    let donor_user_id = Uuid::new_v4();
    let donor_user = NewUser {
        first_name: "Mary",
        last_name: "Donor",
        email: "donor@mail.com",
        phone: Some("+2348012345673"),
        gender: Some(Gender::Female),
        dob: Some(chrono::NaiveDate::from_ymd_opt(1988, 8, 20).unwrap()),
        password_hash: &password_hash,
        role: Role::Donor,
    };

    diesel::insert_into(users::table)
        .values(&donor_user)
        .on_conflict(users::email)
        .do_nothing()
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("donor@mail.com")))
        .set(users::id.eq(donor_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(donor_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    // Add donor medical info
    let donor_medical = MedicalInfo {
        user_id: donor_user_id,
        blood_type: Some("A+"),
        chronic_illnesses: Some("None"),
        allergies: Some("None"),
        hmo_number: None,
        emergency_contact_name: Some("John Donor"),
        emergency_contact_phone: Some("+2348012345674"),
        medical_notes: Some("Regular blood donor"),
    };

    diesel::insert_into(patients::table)
        .values(&donor_medical)
        .execute(&mut conn)?;

    println!("✓ Created Donor user");

    // 3. Create Hospital User
    let hospital_user_id = Uuid::new_v4();
    let hospital_user = NewUser {
        first_name: "General",
        last_name: "Hospital",
        email: "hospital@mail.com",
        phone: Some("+2348012345675"),
        gender: None,
        dob: None,
        password_hash: &password_hash,
        role: Role::Hospital,
    };

    diesel::insert_into(users::table)
        .values(&hospital_user)
        .on_conflict(users::email)
        .do_nothing()
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("hospital@mail.com")))
        .set(users::id.eq(hospital_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(hospital_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    println!("✓ Created Hospital user");

    // 7. Create Specialty
let specialty_id = Uuid::new_v4();

#[derive(Insertable)]
#[diesel(table_name = specialties)]
struct NewSpecialty<'a> {
    id: Uuid,
    name: &'a str,
    description: Option<&'a str>,
}

let new_specialty = NewSpecialty {
    id: specialty_id,
    name: "Hematology",
    description: Some("Blood disorders and diseases"),
};

diesel::insert_into(specialties::table)
    .values(&new_specialty)
    .execute(&mut conn)?;

println!("✓ Created Specialty");

// 8. Link Specialist to Hospital
// ... your existing specialist code, but remove the line:
// let specialty_id = Uuid::new_v4();  // DELETE THIS LINE

    // 4. Create Admin User
    let admin_user_id = Uuid::new_v4();
    let admin_user = NewUser {
        first_name: "System",
        last_name: "Admin",
        email: "admin@mail.com",
        phone: Some("+2348012345676"),
        gender: Some(Gender::Male),
        dob: Some(chrono::NaiveDate::from_ymd_opt(1985, 3, 10).unwrap()),
        password_hash: &password_hash,
        role: Role::Admin,
    };

    diesel::insert_into(users::table)
        .values(&admin_user)
        .on_conflict(users::email)
        .do_nothing()
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("admin@mail.com")))
        .set(users::id.eq(admin_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(admin_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    println!("✓ Created Admin user");

    // 5. Create Specialist User
    let specialist_user_id = Uuid::new_v4();
    let specialist_user = NewUser {
        first_name: "Dr. Sarah",
        last_name: "Specialist",
        email: "specialist@mail.com",
        phone: Some("+2348012345677"),
        gender: Some(Gender::Female),
        dob: Some(chrono::NaiveDate::from_ymd_opt(1982, 11, 25).unwrap()),
        password_hash: &password_hash,
        role: Role::Specialist,
    };

    diesel::insert_into(users::table)
        .values(&specialist_user)
        .on_conflict(users::email)
        .do_nothing()
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("specialist@mail.com")))
        .set(users::id.eq(specialist_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(specialist_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    println!("✓ Created Specialist user");

    // 6. Create Hospital Record
    
    let hospital_id = Uuid::new_v4();
    
    #[derive(Insertable)]
    #[diesel(table_name = hospitals)]
    struct NewHospital<'a> {
        id: Uuid,
        user_id: Uuid,
        name: &'a str,
        hospital_type: Option<health_bridge::utils::enums::HospitalTypeEnum>,
        address: &'a str,
        city: &'a str,
        country: &'a str,
        primary_phone: &'a str,
        emergency_phone: Option<&'a str>,
        email: Option<&'a str>,
        license_number: &'a str,
        accreditation_doc_url: &'a str,
        license_status: bool,
        has_blood_bank: bool,
        accepting_donors: bool,
        donating_operating_hours: Option<&'a str>,
    }

    let new_hospital = NewHospital {
        id: hospital_id,
        user_id: hospital_user_id,
        name: "Lagos General Hospital",
        hospital_type: Some(health_bridge::utils::enums::HospitalTypeEnum::General),
        address: "123 Medical Center Drive",
        city: "Lagos",
        country: "Nigeria",
        primary_phone: "+2348012345675",
        emergency_phone: Some("+2348099999999"),
        email: Some("contact@lagosgeneral.com"),
        license_number: "LIC-2024-001",
        accreditation_doc_url: "https://example.com/accreditation.pdf",
        license_status: true,
        has_blood_bank: true,
        accepting_donors: true,
        donating_operating_hours: Some("Mon-Fri: 8AM-6PM, Sat: 9AM-2PM"),
    };

    diesel::insert_into(hospitals::table)
        .values(&new_hospital)
        .execute(&mut conn)?;

    println!("✓ Created Hospital record");

    // 7. Link Specialist to Hospital
    
    #[derive(Insertable)]
    #[diesel(table_name = specialists)]
    struct NewSpecialist {
        user_id: Uuid,
        hospital_id: Option<Uuid>,
        specialty_id: Uuid,
        bio: Option<&'static str>,
        years_of_experience: Option<i32>,
        consultation_type: ConsultationTypeEnum,
        session_duration_minutes: Option<i32>,
        primary_phone: Option<&'static str>,
        languages_spoken: Option<&'static str>,
    }

    let new_specialist = NewSpecialist {
        user_id: specialist_user_id,
        hospital_id: Some(hospital_id),
        specialty_id,
        bio: Some("Board-certified hematologist with 15 years of experience."),
        years_of_experience: Some(15),
        consultation_type: ConsultationTypeEnum::InPerson,
        session_duration_minutes: Some(30),
        primary_phone: Some("+2348012345677"),
        languages_spoken: Some("English"),
    };

    diesel::insert_into(specialists::table)
        .values(&new_specialist)
        .on_conflict(specialists::specialty_id)
        .do_nothing()
        .execute(&mut conn)?;

    println!("✓ Linked Specialist to Hospital");

    // 7b. Create Specialist Availability
    use health_bridge::utils::enums::DaysOfWeekEnum;

    #[derive(Insertable)]
    #[diesel(table_name = specialist_availabilities)]
    struct NewSpecialistAvailability {
        specialist_id: Uuid,
        day_of_week: DaysOfWeekEnum,
        opens_at: chrono::NaiveTime,
        closes_at: chrono::NaiveTime,
    }

    let availabilities = vec![
        NewSpecialistAvailability {
            specialist_id: specialist_user_id,
            day_of_week: DaysOfWeekEnum::Monday,
            opens_at: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            closes_at: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        },
        NewSpecialistAvailability {
            specialist_id: specialist_user_id,
            day_of_week: DaysOfWeekEnum::Wednesday,
            opens_at: chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            closes_at: chrono::NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        },
    ];

    diesel::insert_into(specialist_availabilities::table)
        .values(&availabilities)
        .execute(&mut conn)?;

    println!("✓ Created Specialist Availability");

    // 8. Create Blood Requests
    
    #[derive(Insertable)]
    #[diesel(table_name = blood_requests)]
    struct NewBloodRequest<'a> {
        id: Uuid,
        hospital_id: Uuid,
        donor_id: Option<Uuid>,
        recipient_id: Option<Uuid>,
        ref_id: &'a str,
        units: Option<i32>,
        blood_type: Option<health_bridge::utils::enums::BloodTypeEnum>,
        urgency: Option<health_bridge::utils::enums::UrgencyTypeEnum>,
        timeline_status: Option<health_bridge::utils::enums::TimelineTypeEnum>,
        request_status: Option<health_bridge::utils::enums::RequestStatusTypeEnum>,
        request_reason: Option<&'a str>,
        note: Option<&'a str>,
        cancelled_by: Option<Uuid>,
        cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
        cancelled_reason: Option<&'a str>,
        preferred_time: Option<chrono::DateTime<chrono::Utc>>,
        donated_at: Option<chrono::DateTime<chrono::Utc>>,
        administered_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    // Blood Request 1: Urgent request for patient
    let blood_request_1_id = Uuid::new_v4();
    let blood_request_1 = NewBloodRequest {
        id: blood_request_1_id,
        hospital_id: hospital_id,
        donor_id: None,
        recipient_id: Some(patient_user_id),
        ref_id: "BR-2024-001",
        units: Some(2),
        blood_type: Some(health_bridge::utils::enums::BloodTypeEnum::OPositive),
        urgency: Some(health_bridge::utils::enums::UrgencyTypeEnum::Standard),
        timeline_status: Some(health_bridge::utils::enums::TimelineTypeEnum::DonationAppointmentScheduled),
        request_status: Some(health_bridge::utils::enums::RequestStatusTypeEnum::Confirmed),
        request_reason: Some("Surgery preparation"),
        note: Some("Patient scheduled for surgery next week"),
        cancelled_by: None,
        cancelled_at: None,
        cancelled_reason: None,
        preferred_time: Some(chrono::Utc::now() + chrono::Duration::days(5)),
        donated_at: None,
        administered_at: None,
    };

    diesel::insert_into(blood_requests::table)
        .values(&blood_request_1)
        .execute(&mut conn)?;

    // Blood Request 2: Completed donation
    let blood_request_2_id = Uuid::new_v4();
    let blood_request_2 = NewBloodRequest {
        id: blood_request_2_id,
        hospital_id: hospital_id,
        donor_id: Some(donor_user_id),
        recipient_id: None,
        ref_id: "BR-2024-002",
        units: Some(1),
        blood_type: Some(health_bridge::utils::enums::BloodTypeEnum::APositive),
        urgency: Some(health_bridge::utils::enums::UrgencyTypeEnum::Standard),
        timeline_status: Some(health_bridge::utils::enums::TimelineTypeEnum::DonationCompleted),
        request_status: Some(health_bridge::utils::enums::RequestStatusTypeEnum::Completed),
        request_reason: Some("Voluntary donation"),
        note: Some("Regular donor, no issues"),
        cancelled_by: None,
        cancelled_at: None,
        cancelled_reason: None,
        preferred_time: Some(chrono::Utc::now() - chrono::Duration::days(2)),
        donated_at: Some(chrono::Utc::now() - chrono::Duration::days(2)),
        administered_at: None,
    };

    diesel::insert_into(blood_requests::table)
        .values(&blood_request_2)
        .execute(&mut conn)?;

    println!("✓ Created Blood Requests");

    // 9. Create Appointments
    
    #[derive(Insertable)]
    #[diesel(table_name = appointments)]
    struct NewAppointment<'a> {
        id: Uuid,
        blood_request_id: Uuid,
        hospital_id: Uuid,
        user_id: Uuid,
        appointment_type: health_bridge::utils::enums::AppointmentTypeEnum,
        status: health_bridge::utils::enums::AppointmentStatusEnum,
        scheduled_time: chrono::DateTime<chrono::Utc>,
        previous_time: Option<chrono::DateTime<chrono::Utc>>,
        cancelled_by: Option<health_bridge::utils::enums::CancelledByEnum>,
        cancelled_by_id: Option<Uuid>,
        cancelled_reason: Option<&'a str>,
        cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    // Appointment 1: Upcoming donation appointment for patient
    let appointment_1 = NewAppointment {
        id: Uuid::new_v4(),
        blood_request_id: blood_request_1_id,
        hospital_id: hospital_id,
        user_id: patient_user_id,
        appointment_type: health_bridge::utils::enums::AppointmentTypeEnum::Patient,
        status: health_bridge::utils::enums::AppointmentStatusEnum::Created,
        scheduled_time: chrono::Utc::now() + chrono::Duration::days(5),
        previous_time: None,
        cancelled_by: None,
        cancelled_by_id: None,
        cancelled_reason: None,
        cancelled_at: None,
    };

    diesel::insert_into(appointments::table)
        .values(&appointment_1)
        .execute(&mut conn)?;

    // Appointment 2: Completed appointment for donor
    let appointment_2 = NewAppointment {
        id: Uuid::new_v4(),
        blood_request_id: blood_request_2_id,
        hospital_id: hospital_id,
        user_id: donor_user_id,
        appointment_type: health_bridge::utils::enums::AppointmentTypeEnum::Donor,
        status: health_bridge::utils::enums::AppointmentStatusEnum::Completed,
        scheduled_time: chrono::Utc::now() - chrono::Duration::days(2),
        previous_time: None,
        cancelled_by: None,
        cancelled_by_id: None,
        cancelled_reason: None,
        cancelled_at: None,
    };

    diesel::insert_into(appointments::table)
        .values(&appointment_2)
        .execute(&mut conn)?;

    // Appointment 3: Cancelled appointment
    let appointment_3 = NewAppointment {
        id: Uuid::new_v4(),
        blood_request_id: blood_request_1_id,
        hospital_id: hospital_id,
        user_id: donor_user_id,
        appointment_type: health_bridge::utils::enums::AppointmentTypeEnum::Patient,
        status: health_bridge::utils::enums::AppointmentStatusEnum::Cancelled,
        scheduled_time: chrono::Utc::now() + chrono::Duration::days(3),
        previous_time: None,
        cancelled_by: Some(health_bridge::utils::enums::CancelledByEnum::Hospital),
        cancelled_by_id: Some(donor_user_id),
        cancelled_reason: Some("Schedule conflict"),
        cancelled_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
    };

    diesel::insert_into(appointments::table)
        .values(&appointment_3)
        .execute(&mut conn)?;

    println!("✓ Created Appointments");

    println!("\n=== Seeding Complete ===");
    println!("\nLogin credentials for all users:");
    println!("Password: password\n");
    println!("Patient:    patient@mail.com");
    println!("Donor:      donor@mail.com");
    println!("Hospital:   hospital@mail.com");
    println!("Admin:      admin@mail.com");
    println!("Specialist: specialist@mail.com");
    println!("\nSeeded data:");
    println!("- 5 users (all verified)");
    println!("- 2 patient medical records");
    println!("- 1 hospital (Lagos General Hospital)");
    println!("- 1 specialist linked to hospital");
    println!("- 2 blood requests (1 pending, 1 completed)");
    println!("- 3 appointments (1 scheduled, 1 completed, 1 cancelled)");

    Ok(())
}