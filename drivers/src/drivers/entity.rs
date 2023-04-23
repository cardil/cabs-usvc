use std::fmt::Display;

use redis_derive::{
    FromRedisValue,
    ToRedisArgs,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
pub struct Driver {
    pub name:    String,
    pub surname: String,
}

#[derive(Debug)]
pub enum Error {
    EmptyName,
    EmptySurname,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyName => write!(f, "Empty name"),
            Error::EmptySurname => write!(f, "Empty surname"),
        }
    }
}

impl Driver {
    pub fn validate(&self) -> Result<(), Error> {
        if self.name.is_empty() {
            return Err(Error::EmptyName);
        }

        if self.surname.is_empty() {
            return Err(Error::EmptySurname);
        }

        Ok(())
    }
}
