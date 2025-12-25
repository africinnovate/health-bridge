use diesel::deserialize::{self, FromSql};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::FromSqlRow;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use utoipa::ToSchema;
use crate::schema::sql_types::{GenderType, RoleType, HospitalType};
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = GenderType)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = RoleType)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Patient,
    Donor,
    Specialist,
    Hospital,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = HospitalType)]
#[serde(rename_all = "lowercase")]
pub enum HospitalTypeEnum {
    Clinic,
    General,
    Teaching,
    Specialist,
    Diagnostic,
}


impl ToSql<GenderType, Pg> for Gender {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Gender::Male => out.write_all(b"male")?,
            Gender::Female => out.write_all(b"female")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<GenderType, Pg> for Gender {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"male" => Ok(Gender::Male),
            b"female" => Ok(Gender::Female),
            _ => Err("Unrecognized enum variant for Gender".into()),
        }
    }
}

impl ToSql<HospitalType, Pg> for HospitalTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            HospitalTypeEnum::Clinic => out.write_all(b"clinic")?,
            HospitalTypeEnum::General => out.write_all(b"general")?,
            HospitalTypeEnum::Teaching => out.write_all(b"teaching")?,
            HospitalTypeEnum::Specialist => out.write_all(b"specialist")?,
            HospitalTypeEnum::Diagnostic => out.write_all(b"diagnostic")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<HospitalType, Pg> for HospitalTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"clinic" => Ok(HospitalTypeEnum::Clinic),
            b"general" => Ok(HospitalTypeEnum::General),
            b"teaching" => Ok(HospitalTypeEnum::Teaching),
            b"specialist" => Ok(HospitalTypeEnum::Specialist),
            b"diagnostic" => Ok(HospitalTypeEnum::Diagnostic),
            _ => Err("Unrecognized enum variant for HospitalType".into()),
        }
    }
}



impl ToSql<RoleType, Pg> for Role {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Role::Patient => out.write_all(b"patient")?,
            Role::Donor => out.write_all(b"donor")?,
            Role::Specialist => out.write_all(b"specialist")?,
            Role::Hospital => out.write_all(b"hospital")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<RoleType, Pg> for Role {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"patient" => Ok(Role::Patient),
            b"donor" => Ok(Role::Donor),
            b"specialist" => Ok(Role::Specialist),
            b"hospital" => Ok(Role::Hospital),
            _ => Err("Unrecognized enum variant for Role".into()),
        }
    }
}

impl FromStr for Role {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "patient" => Ok(Role::Patient),
            "donor" => Ok(Role::Donor),
            "specialist" => Ok(Role::Specialist),
            "hospital" => Ok(Role::Hospital),
            _ => Err(AppError::BadRequest("Invalid role string".to_string())),
        }
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Patient => write!(f, "patient"),
            Role::Donor => write!(f, "donor"),
            Role::Specialist => write!(f, "specialist"),
            Role::Hospital => write!(f, "hospital"),
        }
    }
}

impl std::fmt::Display for HospitalTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HospitalTypeEnum::Clinic => write!(f, "clinic"),
            HospitalTypeEnum::General => write!(f, "general"),
            HospitalTypeEnum::Teaching => write!(f, "teaching"),
            HospitalTypeEnum::Specialist => write!(f, "specialist"),
            HospitalTypeEnum::Diagnostic => write!(f, "diagnostic"),
        }
    }
}