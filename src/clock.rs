////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Clock-measurable time types.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Calendar;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::cmp::Ordering;
use std::num::ParseIntError;


////////////////////////////////////////////////////////////////////////////////
// ClockTime
////////////////////////////////////////////////////////////////////////////////
/// A time period resolvable to units of an hour, minute, or second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct ClockTime {
    /// The hour.
    hour: u8,
    /// The minute.
    minute: Option<u8>,
    /// The second.
    second: Option<u8>,
}

impl ClockTime {
    /// The time period of the first second of a day.
    pub const MIN: Self = Self { hour: 0, minute: Some(0), second: Some(0) };

    /// The time period of at last second of a day.
    pub const MAX: Self = Self { hour: 23, minute: Some(59), second: Some(59) };

    /// Constructs a new `ClockTime` from the given hour value.
    ///
    /// # Panics
    ///
    /// Panics if the value is outside of the valid range.
    #[must_use]
    pub fn from_h(hour: u8) -> Self  {
        assert!(hour < 24);
        Self {
            hour,
            minute: None,
            second: None,
        }
    }

    /// Constructs a new `ClockTime` from the given hour & minute values.
    ///
    /// # Panics
    ///
    /// Panics if any values are outside of the valid ranges.
    #[must_use]
    pub fn from_hm(hour: u8, minute: u8) -> Self  {
        assert!(hour < 24);
        assert!(minute < 60);
        Self {
            hour,
            minute: Some(minute),
            second: None,
        }
    }

    /// Constructs a new `ClockTime` from the given hour, minute, and second
    /// values.
    ///
    /// # Panics
    ///
    /// Panics if any values are outside of the valid ranges.
    #[must_use]
    pub fn from_hms(hour: u8, minute: u8, second: u8) -> Self  {
        assert!(hour < 24);
        assert!(minute < 60);
        assert!(second < 60);
        Self {
            hour,
            minute: Some(minute),
            second: Some(second),
        }
    }

    /// Constructs a new `ClockTime` from the given hour, minute, and second
    /// values.
    ///
    /// If the seconds is provided without a minute, or a minute without an
    /// hour, a value of `0` will be assumed for the less specific values.
    ///
    /// # Panics
    ///
    /// Panics if any values are outside of the valid ranges.
    #[must_use]
    pub fn from_hms_opt(
        hour: Option<u8>,
        minute: Option<u8>,
        second: Option<u8>)
        -> Option<Self>
    {
        if let Some(hour) = hour {
            assert!(hour < 24);
        }
        
        let hour = if let Some(minute) = minute {
            assert!(minute < 60);
            hour.or(Some(0))
        } else {
            None
        };
        
        let (hour, minute) = if let Some(second) = second {
            assert!(second < 60);
            (hour.or(Some(0)), minute.or(Some(0)))
        } else {
            (None, None)
        };

        hour.map(|hour| Self {
            hour,
            minute,
            second,
        })
    }

    /// Sets the hour and returns the `ClockTime`.
    #[must_use]
    pub fn with_hour(mut self, hour: u8) -> Self {
        self.hour = hour;
        self
    }

    /// Sets the minute and returns the `ClockTime`.
    #[must_use]
    pub fn with_minute_opt(mut self, minute: Option<u8>) -> Self {
        self.minute = minute;
        self
    }

    /// Sets the minute and returns the `ClockTime`.
    #[must_use]
    pub fn with_minute(mut self, minute: u8) -> Self {
        self.minute = Some(minute);
        self
    }

    /// Sets the second and returns the `ClockTime`.
    ///
    /// If the minute has not been set, it will be set to `0`.
    #[must_use]
    pub fn with_second_opt(mut self, second: Option<u8>) -> Self {
        self.second = second;
        self.minute = self.minute.or(Some(0));
        self

    }

    /// Sets the second and returns the `ClockTime`.
    ///
    /// If the minute has not been set, it will be set to `0`.
    #[must_use]
    pub fn with_second(mut self, second: u8) -> Self {
        self.second = Some(second);
        self.minute = self.minute.or(Some(0));
        self
    }

    /// Sets the values for any missing components to those of the provided
    /// value and returns the `ClockTime`.
    ///
    /// If the second value is provided without a minute set, the minute will
    /// be set to `0`.
    #[must_use]
    pub fn extend(mut self, time: ClockTime) -> Self {
        if self.minute.is_none() {
            self.minute = time.minute;
        }
        if self.second.is_none() {
            self.second = time.second;
            self.minute = self.minute.or(Some(0));
        }
        self
    }

}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self { hour, minute: None, second: None }
                => write!(f, "{:02}", hour),
            Self { hour, minute: Some(m), second: None }
                => write!(f, "{:02}:{:02}", hour, m),
            Self { hour, minute: Some(m), second: Some(s) }
                => write!(f, "{:02}:{:02}:{:02}", hour, m, s),
            Self { hour, minute: None, second: Some(s) }
                => write!(f, "{:02}:00:{:02}", hour, s),
        }
    }
}

// NOTE: We only compare times with equivalent resolutions. If unequivalent
// resolutions need to be compared, the values should be projected via
// `earliest` and `latest` first.
impl PartialOrd for ClockTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut res = self.hour.cmp(&other.hour);
        match (self.minute, other.minute) {
            (Some(a), Some(b)) => { res = res.then(a.cmp(&b)); },
            (None,    None)    => { /* Do nothing. */ },
            _ => return None,
        }
        match (self.second, other.second) {
            (Some(a), Some(b)) => { res = res.then(a.cmp(&b)); },
            (None,    None)    => { /* Do nothing. */ },
            _ => return None,
        }
        Some(res)
    }
}


impl Calendar for ClockTime {
    type ParseErr = ClockTimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::ParseErr> {
        todo!()
    }

    fn earliest(&self) -> Self {
        self.extend(Self::MIN)
    }

    fn latest(&self) -> Self {
        self.extend(Self::MAX)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Errors
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum ClockTimeParseError {
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ClockTimeParseError {
    fn from(e: ParseIntError) -> Self {
        ClockTimeParseError::ParseIntError(e)
    }
}

impl std::fmt::Display for ClockTimeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for ClockTimeParseError {}
