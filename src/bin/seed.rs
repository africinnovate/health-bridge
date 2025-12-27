use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};


use anyhow::{anyhow, Result};


use health_bridge::models::{
    User,
    NewUser,
    MedicalInfo,
};

use health_bridge::schema::{
    users,
    patients,
};

use health_bridge::utils::enums::{
    Gender,
    Role,
};

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("failed to hash password: {}", e))?
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

    // Hash password once for all users
    let password_hash = hash_password("password")?;
    // let password_hash = hash_password("password")?;

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
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("hospital@mail.com")))
        .set(users::id.eq(hospital_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(hospital_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    println!("✓ Created Hospital user");

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
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::email.eq("specialist@mail.com")))
        .set(users::id.eq(specialist_user_id))
        .execute(&mut conn)?;

    diesel::update(users::table.filter(users::id.eq(specialist_user_id)))
        .set(users::email_verified.eq(true))
        .execute(&mut conn)?;

    println!("✓ Created Specialist user");

    println!("\n=== Seeding Complete ===");
    println!("\nLogin credentials for all users:");
    println!("Password: password\n");
    println!("Patient:    patient@mail.com");
    println!("Donor:      donor@mail.com");
    println!("Hospital:   hospital@mail.com");
    println!("Admin:      admin@mail.com");
    println!("Specialist: specialist@mail.com");
    println!("\nAll users have verified emails and are ready to use.");

    Ok(())
}