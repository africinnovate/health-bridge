use diesel::deserialize::{self, FromSql};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::FromSqlRow;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use utoipa::ToSchema;
use crate::schema::sql_types::{
    GenderType, 
    RoleType, 
    TimelineType,
    BloodType,
    UrgencyType,
    BloodRequestStatusType,
    HospitalType,
    AppointmentTypeType,
    AppointmentStatusType,
    CancelledByType,
};
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
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = AppointmentTypeType)]
pub enum AppointmentTypeEnum {
    Donor,
    Recipient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = AppointmentStatusType)]
pub enum AppointmentStatusEnum {
    Created,
    Confirmed,
    Rescheduled,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = CancelledByType)]
pub enum CancelledByEnum {
    Hospital,
    Donor,
    Patient,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = BloodType)]
#[serde(rename_all = "lowercase")]
pub enum BloodTypeEnum {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = UrgencyType)]
#[serde(rename_all = "lowercase")]
pub enum UrgencyTypeEnum {
    Standard,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = TimelineType)]
#[serde(rename_all = "lowercase")]
pub enum TimelineTypeEnum {
    RequestCreated,
    VisibleToDonors,
    DonationAppointmentScheduled,
    DonationCompleted,
    RequestFulfilled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, ToSchema)]
#[diesel(sql_type = BloodRequestStatusType)]
#[serde(rename_all = "lowercase")]
pub enum RequestStatusTypeEnum {
    Confirmed,
    Accepted,
    Completed,
    Cancelled,
}

impl ToSql<AppointmentTypeType, Pg> for AppointmentTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            AppointmentTypeEnum::Donor => out.write_all(b"donor")?,
            AppointmentTypeEnum::Recipient => out.write_all(b"recipient")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<AppointmentTypeType, Pg> for AppointmentTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"donor" => Ok(AppointmentTypeEnum::Donor),
            b"recipient" => Ok(AppointmentTypeEnum::Recipient),
            _ => Err("Unrecognized enum variant for AppointmentTypeEnum".into()),
        }
    }
}

impl ToSql<AppointmentStatusType, Pg> for AppointmentStatusEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            AppointmentStatusEnum::Created => out.write_all(b"created")?,
            AppointmentStatusEnum::Confirmed => out.write_all(b"confirmed")?,
            AppointmentStatusEnum::Rescheduled => out.write_all(b"rescheduled")?,
            AppointmentStatusEnum::Cancelled => out.write_all(b"cancelled")?,
            AppointmentStatusEnum::Completed => out.write_all(b"completed")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<AppointmentStatusType, Pg> for AppointmentStatusEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"created" => Ok(AppointmentStatusEnum::Created),
            b"confirmed" => Ok(AppointmentStatusEnum::Confirmed),
            b"rescheduled" => Ok(AppointmentStatusEnum::Rescheduled),
            b"cancelled" => Ok(AppointmentStatusEnum::Cancelled),
            b"completed" => Ok(AppointmentStatusEnum::Completed),
            _ => Err("Unrecognized enum variant for AppointmentStatusEnum".into()),
        }
    }
}

impl ToSql<AppointmentStatusType, Pg> for CancelledByEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            CancelledByEnum::Hospital => out.write_all(b"hospital")?,
            CancelledByEnum::Donor => out.write_all(b"donor")?,
            CancelledByEnum::Patient => out.write_all(b"patient")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<CancelledByType, Pg> for CancelledByEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"hospital" => Ok(CancelledByEnum::Hospital),
            b"donor" => Ok(CancelledByEnum::Donor),
            b"cancelled" => Ok(CancelledByEnum::Patient),
            _ => Err("Unrecognized enum variant for CancelledByEnum".into()),
        }
    }
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
            Role::Admin => out.write_all(b"admin")?,
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
            b"admin" => Ok(Role::Admin),
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
            "admin" => Ok(Role::Admin),
            _ => Err(AppError::BadRequest("Invalid role string".to_string())),
        }
    }
}

impl FromStr for BloodTypeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BloodTypeEnum::*;
        match s {
            "apositive" => Ok(APositive),
            "anegative" => Ok(ANegative),
            "bpositive" => Ok(BPositive),
            "bnegative" => Ok(BNegative),
            "abpositive" => Ok(ABPositive),
            "abnegative" => Ok(ABNegative),
            "opositive" => Ok(OPositive),
            "onegative" => Ok(ONegative),
            _ => Err("Invalid blood type".into()),
        }
    }
}

impl ToSql<BloodType, Pg> for BloodTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            BloodTypeEnum::APositive => out.write_all(b"apositive")?,
            BloodTypeEnum::ANegative => out.write_all(b"anegative")?,
            BloodTypeEnum::BPositive => out.write_all(b"bpositive")?,
            BloodTypeEnum::BNegative => out.write_all(b"bnegative")?,
            BloodTypeEnum::ABPositive => out.write_all(b"abpositive")?,
            BloodTypeEnum::ABNegative => out.write_all(b"abnegative")?,
            BloodTypeEnum::OPositive => out.write_all(b"opositive")?,
            BloodTypeEnum::ONegative => out.write_all(b"onegative")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<BloodType, Pg> for BloodTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"apositive" => Ok(BloodTypeEnum::APositive),
            b"anegative" => Ok(BloodTypeEnum::ANegative),
            b"bpositive" => Ok(BloodTypeEnum::BPositive),
            b"bnegative" => Ok(BloodTypeEnum::BNegative),
            b"abpositive" => Ok(BloodTypeEnum::ABPositive),
            b"abnegative" => Ok(BloodTypeEnum::ABNegative),
            b"opositive" => Ok(BloodTypeEnum::OPositive),
            b"onegative" => Ok(BloodTypeEnum::ONegative),
            _ => Err("Unrecognized enum variant for BloodType".into()),
        }
    }
}


impl FromStr for UrgencyTypeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UrgencyTypeEnum::*;
        match s {
            "standard" => Ok(Standard),
            "urgent" => Ok(Urgent),
            _ => Err("Invalid urgency type".into()),
        }
    }
}

impl ToSql<UrgencyType, Pg> for UrgencyTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            UrgencyTypeEnum::Standard => out.write_all(b"standard")?,
            UrgencyTypeEnum::Urgent => out.write_all(b"urgent")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<UrgencyType, Pg> for UrgencyTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"standard" => Ok(UrgencyTypeEnum::Standard),
            b"urgent" => Ok(UrgencyTypeEnum::Urgent),
            _ => Err("Unrecognized enum variant for UrgencyType".into()),
        }
    }
}


impl FromStr for TimelineTypeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TimelineTypeEnum::*;
        match s {
            "request_created" => Ok(RequestCreated),
            "visible_to_donors" => Ok(VisibleToDonors),
            "donation_appointment_scheduled" => Ok(DonationAppointmentScheduled),
            "donation_completed" => Ok(DonationCompleted),
            "request_fulfilled" => Ok(RequestFulfilled),
            _ => Err("Invalid timeline type".into()),
        }
    }
}

impl ToSql<TimelineType, Pg> for TimelineTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            TimelineTypeEnum::RequestCreated => out.write_all(b"request_created")?,
            TimelineTypeEnum::VisibleToDonors => out.write_all(b"visible_to_donors")?,
            TimelineTypeEnum::DonationAppointmentScheduled => {
                out.write_all(b"donation_appointment_scheduled")?
            }
            TimelineTypeEnum::DonationCompleted => out.write_all(b"donation_completed")?,
            TimelineTypeEnum::RequestFulfilled => out.write_all(b"request_fulfilled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<TimelineType, Pg> for TimelineTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"request_created" => Ok(TimelineTypeEnum::RequestCreated),
            b"visible_to_donors" => Ok(TimelineTypeEnum::VisibleToDonors),
            b"donation_appointment_scheduled" => {
                Ok(TimelineTypeEnum::DonationAppointmentScheduled)
            }
            b"donation_completed" => Ok(TimelineTypeEnum::DonationCompleted),
            b"request_fulfilled" => Ok(TimelineTypeEnum::RequestFulfilled),
            _ => Err("Unrecognized enum variant for TimelineType".into()),
        }
    }
}


impl FromStr for RequestStatusTypeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RequestStatusTypeEnum::*;
        match s {
            "confirmed" => Ok(Confirmed),
            "accepted" => Ok(Accepted),
            "completed" => Ok(Completed),
            "cancelled" => Ok(Cancelled),
            _ => Err("Invalid blood request status type".into()),
        }
    }
}

impl ToSql<BloodRequestStatusType, Pg> for RequestStatusTypeEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            RequestStatusTypeEnum::Confirmed => out.write_all(b"confirmed")?,
            RequestStatusTypeEnum::Accepted => out.write_all(b"accepted")?,
            RequestStatusTypeEnum::Completed => out.write_all(b"completed")?,
            RequestStatusTypeEnum::Cancelled => out.write_all(b"cancelled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<BloodRequestStatusType, Pg> for RequestStatusTypeEnum {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"confirmed" => Ok(RequestStatusTypeEnum::Confirmed),
            b"accepted" => Ok(RequestStatusTypeEnum::Accepted),
            b"completed" => Ok(RequestStatusTypeEnum::Completed),
            b"cancelled" => Ok(RequestStatusTypeEnum::Cancelled),
            _ => Err("Unrecognized enum variant for BloodRequestStatusType".into()),
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
            Role::Admin => write!(f, "admin"),
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