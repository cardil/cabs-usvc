use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Self = Money(0);

    pub fn new(amount: i64) -> Self {
        Self(amount)
    }

    pub fn subtract(&self, other: &Self) -> Self {
        Self(self.0 - other.0)
    }

    pub fn percentage(&self, percentage: f64) -> Self {
        Self((percentage * self.0 as f64 / 100.0) as i64)
    }

    pub fn less_then(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.0 as f64 / 100.0;
        write!(f, "{:.2}", d)
    }
}
