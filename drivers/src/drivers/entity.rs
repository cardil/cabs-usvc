use chrono::{
    DateTime,
    Local,
};
use std::collections::HashMap;
use std::fmt::Display;

use crate::support::clock::{
    Clock,
    Now,
};
use crate::support::money::Money;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDriver {
    pub name:       String,
    pub surname:    String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license:    Option<License>,
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub attributes: HashMap<Attribute, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee:        Option<Fee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub name:       String,
    pub surname:    String,
    #[serde(skip_serializing_if = "default", default)]
    pub status:     Status,
    #[serde(skip_serializing_if = "default", default)]
    pub r#type:     Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license:    Option<License>,
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::new"
    )]
    pub attributes: HashMap<Attribute, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee:        Option<Fee>,
}

impl Driver {
    pub fn calculate_fee(&self, transit_price: &Money) -> Money {
        let company_fee = self.fee.clone().unwrap_or_default();

        let fee = match company_fee.r#type {
            FeeType::Percentage => {
                let percentage = company_fee.amount as f64 / 100.0;
                transit_price.percentage(percentage)
            }
            FeeType::Flat => Money::new(company_fee.amount as i64),
        };

        let fee = transit_price.subtract(&fee);

        let fee = match company_fee.min {
            Some(min) => {
                let mut min = Money::new(min as i64);
                let leftover = transit_price.subtract(&min);
                if leftover.less_then(&Money::ZERO) {
                    min = transit_price.clone();
                }
                if fee.less_then(&min) {
                    min
                } else {
                    fee
                }
            }
            None => fee,
        };

        return fee;
    }

    pub(crate) fn with_type(&self, typ: Type) -> Driver {
        let mut driver = self.clone();
        driver.r#type = typ;

        driver
    }

    pub(crate) fn activate(&self, clk: &Clock) -> Result<Driver, Error> {
        let now = clk.now();
        let mut driver = self.clone();
        driver.status = Status::Active;
        if !driver
            .attributes
            .contains_key(&Attribute::YearsOfExperience)
        {
            driver.attributes.insert(
                Attribute::YearsOfExperience,
                format!("{}", now.to_rfc3339()),
            );
        }

        match self.license {
            Some(ref license) => license.validate(clk),
            None => {
                Err(Error::InvalidLicense("Driver has no license".to_string()))
            }
        }?;

        Ok(driver)
    }

    pub(crate) fn deactivate(&self) -> Driver {
        let mut driver = self.clone();
        driver.status = Status::Inactive;

        driver
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Attribute {
    PenaltyPoints,
    Nationality,
    YearsOfExperience,
    MedicalExaminationExpirationDate,
    MedicalExaminationRemarks,
    Email,
    Birthplace,
    CompanyName,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FeeType {
    Flat,
    Percentage,
}

impl Display for FeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", repr)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Fee {
    pub r#type: FeeType,
    pub amount: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min:    Option<usize>,
}

impl Fee {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        let ft = self.r#type.clone();
        match self.r#type {
            FeeType::Flat => {
                if self.amount <= 0 {
                    return Err(Error::InvalidFeeAmount(self.amount, ft));
                }

                if let Some(min) = self.min {
                    if min > self.amount {
                        return Err(Error::InvalidFeeMin(min, ft));
                    }
                }
            }
            FeeType::Percentage => {
                if self.amount > 10000 {
                    return Err(Error::InvalidFeeAmount(self.amount, ft));
                }
            }
        }

        Ok(())
    }
}

impl Default for Fee {
    fn default() -> Self {
        Self {
            r#type: FeeType::Percentage,
            amount: 200,
            min:    None,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidName(String),
    InvalidSurname(String),
    InvalidLicense(String),
    InvalidFeeAmount(usize, FeeType),
    InvalidFeeMin(usize, FeeType),
}

impl Default for Driver {
    fn default() -> Self {
        Self {
            name:       String::new(),
            surname:    String::new(),
            status:     Status::default(),
            r#type:     Type::default(),
            photo:      None,
            license:    None,
            attributes: HashMap::new(),
            fee:        None,
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
            Error::InvalidFeeAmount(a, ft) => {
                write!(f, "Invalid fee amount: {} for type {}", a, ft)
            }
            Error::InvalidFeeMin(m, ft) => {
                write!(f, "Invalid fee minimum: {} for type {}", m, ft)
            }
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

        match self.fee {
            None => Ok(()),
            Some(ref fee) => fee.validate(),
        }?;

        match self.license {
            None => Ok(()),
            Some(ref license) => license.validate(clock),
        }
    }

    pub(crate) fn onto(self, defaults: &Driver) -> Driver {
        Driver {
            name:       self.name,
            surname:    self.surname,
            status:     defaults.status.clone(),
            r#type:     defaults.r#type.clone(),
            photo:      self.photo,
            license:    self.license,
            attributes: defaults
                .attributes
                .iter()
                .chain(self.attributes.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            fee:        self.fee,
        }
    }
}

lazy_static! {
    static ref LICENSE_NUMBER_REGEX: regex::Regex =
        regex::Regex::new(r"(?i)^[a-z9]{5}\d{6}[a-z9]{2}\d[a-z]{2}$").unwrap();
}

fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
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
