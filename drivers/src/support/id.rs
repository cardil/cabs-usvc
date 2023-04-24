use std::ops::Sub;

use chrono::{
    DateTime,
    Local,
    TimeZone,
};
use data_encoding::Encoding;
use num::{
    BigInt,
    ToPrimitive,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::support::clock::{
    Clock,
    Now,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ID<T> {
    pub id:     Identifier,
    pub entity: T,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Identifier(i64);

impl Identifier {
    pub fn new(clock: &Clock) -> Self {
        let n = clock.now();
        let diff = n.sub(EPOCH.clone());
        let ms = diff.num_milliseconds();
        Identifier(ms)
    }

    pub fn int(&self) -> i64 {
        self.0
    }
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        let ts_bi = BigInt::from(self.0);
        let input = ts_bi.to_bytes_be().1;
        B32.encode(input.as_slice())
    }
}

impl From<&String> for Identifier {
    fn from(id: &String) -> Self {
        let decoded = B32.decode(id.as_bytes()).unwrap();
        let sign = num::bigint::Sign::Plus;
        let bi = BigInt::from_bytes_be(sign, decoded.as_slice());
        Identifier(bi.to_i64().unwrap())
    }
}

impl From<i64> for Identifier {
    fn from(id: i64) -> Self {
        Identifier(id)
    }
}

lazy_static! {
    static ref B32: Encoding = {
        let mut spec = data_encoding::Specification::new();
        spec.symbols.push_str("234567abcdefghijklmnopqrstuvwxyz");
        spec.encoding().unwrap()
    };

    /// Epoch designed for app to have as shorter IDs.
    static ref EPOCH: DateTime<Local> = {
        Local.with_ymd_and_hms(2023, 4, 12, 12, 43, 56).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::clock::Fixed;

    #[test]
    fn test_interoperation_with_lexicoid() {
        let cases = [
            (0, "22"),
            (100, "gk"),
            (1700000, "5bse2"),
            (550000000, "6567f22"),
            (1550000000, "flllz22"),
            (1654401676, "gei4p52"),
            (1674301677, "gj7x3vc"),
        ];
        cases.iter().for_each(|(ts, repr)| {
            let id = Identifier(*ts);
            assert_eq!(id.to_string(), *repr);
        });
    }

    #[test]
    fn test_epoch() {
        let clk = Clock::FixedClock(Fixed {
            time: EPOCH.clone(),
        });
        let id = Identifier::new(&clk);
        assert_eq!(id.int(), 0);
        assert_eq!(id.to_string(), "22");
    }

    #[test]
    fn test_from_string() {
        let id = Identifier(1);
        assert_eq!(id.to_string(), "26");
        let id = Identifier::from(&"26".to_string());
        assert_eq!(id.int(), 1);
        assert_eq!(id.to_string(), "26");

        let id = Identifier::from(&"gk".to_string());
        assert_eq!(id.int(), 100);
        assert_eq!(id.to_string(), "gk");

        let id = Identifier::from(&"5bse2".to_string());
        assert_eq!(id.int(), 1700000);
        assert_eq!(id.to_string(), "5bse2");

        let id = Identifier::from(&"biykars".to_string());
        assert_eq!(id.to_string(), "biykars");
    }
}
