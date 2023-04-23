use chrono::{
    DateTime,
    Local,
};

pub trait Now {
    fn now(&self) -> DateTime<Local>;
}

#[derive(Clone, Debug)]
pub struct Fixed {
    pub time: DateTime<Local>,
}

impl Now for Fixed {
    fn now(&self) -> DateTime<Local> {
        self.time
    }
}

#[derive(Clone, Debug)]
pub struct System;

impl Now for System {
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
}

#[derive(Clone, Debug)]
pub enum Clock {
    SystemClock(System),

    #[allow(dead_code)] // This is only used for testing.
    FixedClock(Fixed),
}

impl Now for Clock {
    fn now(&self) -> DateTime<Local> {
        match self {
            Clock::SystemClock(c) => c.now(),
            Clock::FixedClock(f) => f.now(),
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Clock::SystemClock(System)
    }
}
