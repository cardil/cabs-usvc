use chrono::{
    DateTime,
    Local,
};
use std::fmt::Display;

use crate::support::clock::{
    Clock,
    Now,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDriver {
    pub name:    String,
    pub surname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub name:    String,
    pub surname: String,
    #[serde(default)]
    pub status:  Status,
    #[serde(default)]
    pub r#type:  Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub number:  String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_dt",
        deserialize_with = "deserialize_dt"
    )]
    pub expires: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Type {
    Candidate,
    Regular,
}

#[derive(Debug)]
pub enum Error {
    InvalidName(String),
    InvalidSurname(String),
    InvalidLicense(String),
}

impl Default for Driver {
    fn default() -> Self {
        Self {
            name:    String::new(),
            surname: String::new(),
            status:  Status::default(),
            r#type:  Type::default(),
            photo:   None,
            license: None,
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Candidate
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Inactive
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidName(n) => write!(f, "Invalid name: {}", n),
            Error::InvalidSurname(n) => write!(f, "Invalid surname: {}", n),
            Error::InvalidLicense(l) => write!(f, "Invalid license: {}", l),
        }
    }
}

impl License {
    pub fn validate(&self, clock: &Clock) -> Result<(), Error> {
        if !LICENSE_NUMBER_REGEX.is_match(&self.number) {
            return Err(Error::InvalidLicense(format!(
                "number {} does not match regex {}",
                self.number,
                LICENSE_NUMBER_REGEX.as_str()
            )));
        }

        match self.expires {
            None => Ok(()),
            Some(dt) => {
                if dt < clock.now() {
                    return Err(Error::InvalidLicense(format!(
                        "license expired on {}",
                        dt.to_rfc3339()
                    )));
                }
                Ok(())
            }
        }
    }
}

impl NewDriver {
    pub fn validate(&self, clock: &Clock) -> Result<(), Error> {
        if self.name.is_empty() {
            return Err(Error::InvalidName(self.name.clone()));
        }

        if self.surname.is_empty() {
            return Err(Error::InvalidSurname(self.surname.clone()));
        }

        match self.license {
            None => Ok(()),
            Some(ref license) => license.validate(clock),
        }
    }

    pub(crate) fn onto(self, defaults: &Driver) -> Driver {
        Driver {
            name:    self.name,
            surname: self.surname,
            status:  defaults.status.clone(),
            r#type:  defaults.r#type.clone(),
            photo:   self.photo,
            license: self.license,
        }
    }
}

lazy_static! {
    static ref LICENSE_NUMBER_REGEX: regex::Regex =
        regex::Regex::new(r"(?i)^[a-z9]{5}\d{6}[a-z9]{2}\d[a-z]{2}$").unwrap();
}

fn serialize_dt<S>(
    dt: &Option<DateTime<Local>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match dt {
        Some(dt) => s.serialize_str(&dt.to_rfc3339()),
        None => s.serialize_none(),
    }
}

fn deserialize_dt<'de, D>(d: D) -> Result<Option<DateTime<Local>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = match String::deserialize(d).ok() {
        Some(s) => s,
        None => return Ok(None),
    };

    let dt =
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
    Ok(Some(dt.into()))
}
