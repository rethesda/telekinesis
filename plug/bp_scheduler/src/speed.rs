use std::fmt::{Display, self};

use funscript::FSPoint;

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    pub value: u16,
}

impl Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Speed {
    pub fn new(mut percentage: i64) -> Speed {
        if percentage < 0 {
            percentage = 0;
        }
        if percentage > 100 {
            percentage = 100;
        }
        Speed {
            value: percentage as u16,
        }
    }
    pub fn from_fs(point: &FSPoint) -> Speed {
        Speed::new(point.pos.into())
    }
    pub fn min() -> Speed {
        Speed { value: 0 }
    }
    pub fn max() -> Speed {
        Speed { value: 100 }
    }
    pub fn as_float(self) -> f64 {
        self.value as f64 / 100.0
    }
}