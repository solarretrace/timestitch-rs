////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Calendar types.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::TimeInterval;
use crate::Calendar;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use regex::Regex;
pub use chrono::Weekday;
pub use chrono::ParseWeekdayError;

// Standard Library imports.
use std::str::FromStr;
use std::fmt::Display;
use std::fmt::Debug;
use std::rc::Rc;
use std::convert::TryFrom;
use std::num::ParseIntError;


////////////////////////////////////////////////////////////////////////////////
// GregorianProleptic
////////////////////////////////////////////////////////////////////////////////
/// A parsed real-world time period based on the Gregorian proleptic calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum GregorianProleptic {
    /// The year.
    Y {
        /// The Gregorian proleptic calendar year.
        year: i32,
    },
    /// The year and season.
    Yx {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// A calendar season.
        season: Season,
    },
    /// The year and month.
    Ym {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the month within the year.
        month: u32,
    },
    /// The ISO week: year and week number.
    Yw {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the week.
        week: u32,
    },
    /// The year, month, and day.
    Ymd {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the month within the year.
        month: u32,
        /// The index of the day within the month.
        day: u32,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
    /// The year and ordinal.
    Yo {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the day within the year.
        ordinal: u32,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
    /// The ISO week date: year, week number, day of week.
    Ywd {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the week.
        week: u32,
        /// The day of the week.
        weekday: Weekday,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
    /// The number of days since 00010101 in the Gregorian proleptic calendar.
    CeDay {
        /// The index of the day relative to the start of the current era.
        days: i32,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
    /// The number of days since 19700101 in the Gregorian proleptic calendar.
    EpochDay {
        /// The index of the day relative to the start of the UNIX epoch.
        days: i32,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
    /// The `n`th weekday of the month of the given year.
    MonthWeekday {
        /// The Gregorian proleptic calendar year.
        year: i32,
        /// The index of the month within the year.
        month: u32,
        /// The day of the week.
        weekday: Weekday,
        /// The index of the week within the month.
        n: u8,
        /// An optional clock time resolution.
        time: Option<ClockTime>,
    },
}

impl From<GregorianProleptic> for TimeInterval<GregorianProleptic> {
    fn from(c: GregorianProleptic) -> Self {
        todo!()
    }
}

impl FromStr for GregorianProleptic {
    type Err = GregorianProlepticParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for GregorianProleptic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("<TIME>"), f)
    }
}

impl PartialOrd for GregorianProleptic {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Calendar for GregorianProleptic {
    fn earliest(&self) -> Self {
        todo!()
    } 

    fn latest(&self) -> Self {
        todo!()
    } 
}


////////////////////////////////////////////////////////////////////////////////
// Formats
////////////////////////////////////////////////////////////////////////////////
/// Unparsed `GregorianProleptic` calendar values with data needed to parse
/// them.
#[derive(Debug, Clone)]
pub struct GregorianProlepticRaw {
    /// The calendar format to parse.
    pub format: Format,
    /// The calendar period text.
    pub text: Box<str>,
    /// The regular expression providing captures for the calendar data.
    pub re: Rc<Regex>,
    // TODO: Capture group mapping.
}

impl TryFrom<GregorianProlepticRaw> for GregorianProleptic {
    type Error = GregorianProlepticParseError;
    fn try_from(raw: GregorianProlepticRaw) -> Result<Self, Self::Error> {
        use GregorianProleptic::*;
        raw.format.validate_capture_group_count(&raw.re)?;
        let cap = raw.re.captures(&raw.text)
            .ok_or_else(|| GregorianProlepticParseError::CaptureMatchFailure(
                raw.text.clone()))?;
        match raw.format {
            Format::Y => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                Ok(Y { year })
            },
            Format::Yx => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                todo!()

                //Ok(Yx { year, season })
            },
            Format::Ym => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let month: u32 = cap.get(2).unwrap().as_str().parse()?;
                Ok(Ym { year, month })
            },
            Format::Ymd => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let month: u32 = cap.get(2).unwrap().as_str().parse()?;
                let day: u32 = cap.get(3).unwrap().as_str().parse()?;
                let h = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(6).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(Ymd { year, month, day, time })
            },
            Format::Yo => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let ordinal: u32 = cap.get(2).unwrap().as_str().parse()?;
                let h = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(Yo { year, ordinal, time })
            },
            Format::Yw => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let week: u32 = cap.get(2).unwrap().as_str().parse()?;
                Ok(Yw { year, week })
            },
            Format::Ywd => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let week: u32 = cap.get(2).unwrap().as_str().parse()?;
                let weekday: Weekday = cap.get(3).unwrap().as_str().parse()?;
                let h = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(Ywd { year, week, weekday, time })
            },
            Format::CeDay => {
                let days: i32 = cap.get(1).unwrap().as_str().parse()?;
                let h = cap.get(2).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(CeDay { days, time })
            },
            Format::EpochDay => {
                let days: i32 = cap.get(1).unwrap().as_str().parse()?;
                let h = cap.get(2).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(EpochDay { days, time })
            },
            Format::MonthWeekday => {
                let year: i32 = cap.get(1).unwrap().as_str().parse()?;
                let month: u32 = cap.get(2).unwrap().as_str().parse()?;
                let weekday: Weekday = cap.get(3).unwrap().as_str().parse()?;
                let n: u8 = cap.get(4).unwrap().as_str().parse()?;
                let h = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
                let m = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
                let s = cap.get(6).map(|g| g.as_str().parse()).transpose()?;
                let time = ClockTime::new(h, m, s);

                Ok(MonthWeekday { year, month, weekday, n, time })
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
/// The parsed time format.
pub enum Format {
    /// The year.
    Y,
    /// The year and season.
    Yx,
    /// The year and month.
    Ym,
    /// The ISO week: year and week number.
    Yw,
    /// The year, month, and day.
    Ymd,
    /// The year and ordinal.
    Yo,
    /// The ISO week date: year, week number, day of week.
    Ywd,
    /// The number of days since 00010101 in the Gregorian proleptic calendar.
    CeDay,
    /// The number of days since 19700101 in the Gregorian proleptic calendar.
    EpochDay,
    /// The `n`th weekday of the month of the given year.
    MonthWeekday,
}

impl Format {
    fn validate_capture_group_count(&self, re: &Regex)
        -> Result<usize, CaptureGroupCountError>
    {
        let (min, max) = match self {
            Format::Y            => (2, 2),
            Format::Yx           => (3, 3),
            Format::Ym           => (3, 3),
            Format::Yw           => (3, 6),
            Format::Ymd          => (4, 7),
            Format::Yo           => (3, 6),
            Format::Ywd          => (4, 7),
            Format::CeDay        => (2, 5),
            Format::EpochDay     => (2, 5),
            Format::MonthWeekday => (5, 8),
        };
        let len = re.captures_len();
        if !(min..=max).contains(&len) {
            let min: u8 = min.try_into().unwrap();
            let max: u8 = max.try_into().unwrap();
            Err(CaptureGroupCountError { format: *self, min, max })
        } else {
            Ok(len)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}


////////////////////////////////////////////////////////////////////////////////
// ClockTime
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum ClockTime {
    /// A time resolved to hour resolution.
    H {
        /// The hour.
        hour: u8,
    },
    /// A time resolved to minute resolution.
    Hm {
        /// The hour.
        hour: u8,
        /// The minute.
        minute: u8
    },
    /// A time resolved to second resolution.
    Hms {
        /// The hour.
        hour: u8,
        /// The minute.
        minute: u8,
        /// The second.
        second: u8
    },
}

impl ClockTime {
    /// Constructs a new `ClockTime` from the given hour, minute, second values.
    ///
    /// # Panics
    ///
    /// Panics if a minute value is provided without an hour, or if a second
    /// value is provided without a minute and hour.
    #[must_use]
    pub fn new(hour: Option<u8>, minute: Option<u8>, second: Option<u8>)
        -> Option<Self>
    {
        match (hour, minute, second) {
            (None, None, None) => None,
            (Some(hour), None, None) => Some(Self::H { hour }),
            (Some(hour), Some(minute), None) => Some(Self::Hm { hour, minute }),
            (Some(hour), Some(minute), Some(second))
                => Some(Self::Hms { hour, minute, second }),

            _ => panic!("invalid clock time resolution"),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Errors
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum GregorianProlepticParseError {
    CaptureMatchFailure(Box<str>),
    CaptureGroupCountError(CaptureGroupCountError),
    ParseIntError(ParseIntError),
    ParseWeekdayError(ParseWeekdayError),
}

impl From<CaptureGroupCountError> for GregorianProlepticParseError {
    fn from(e: CaptureGroupCountError) -> Self {
        GregorianProlepticParseError::CaptureGroupCountError(e)
    }
}

impl From<ParseIntError> for GregorianProlepticParseError {
    fn from(e: ParseIntError) -> Self {
        GregorianProlepticParseError::ParseIntError(e)
    }
}

impl From<ParseWeekdayError> for GregorianProlepticParseError {
    fn from(e: ParseWeekdayError) -> Self {
        GregorianProlepticParseError::ParseWeekdayError(e)
    }
}

impl Display for GregorianProlepticParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR")
    }
}

impl std::error::Error for GregorianProlepticParseError {}


/// An incorrect number of capture groups have been provided for the calendar
/// parser.
#[derive(Debug, Clone, Copy)]
pub struct CaptureGroupCountError {
    /// The time format.
    pub format: Format,
    /// The minimum number of capture groups to use.
    pub min: u8,
    /// The maximum number of capture groups to use.
    pub max: u8,
}

impl Display for CaptureGroupCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min == self.max {
            write!(f, "invalid capture group count for format {:?}: \
                expected {} groups", self.format, self.min)
        } else {
            write!(f, "invalid capture group count for format {:?}: \
                expected {} to {} groups", self.format, self.min, self.max)
        }
    }
}

impl std::error::Error for CaptureGroupCountError {}
