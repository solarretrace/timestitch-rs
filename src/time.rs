////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Record time types.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::EntryId;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard Library imports.
use std::str::FromStr;



/// An interval of time.
#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub struct TimeInterval {
    /// The start of the interval.
    pub start: TimeBound,
    /// The end of the interval.
    pub end: TimeBound,
}

impl TimeInterval {
    pub fn unknown() -> Self {
        Self {
            start: TimeBound::Unbounded,
            end: TimeBound::Unbounded,
        }
    }
}

impl FromStr for TimeInterval {
    type Err = TimeIntervalParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Err(TimeIntervalParseError {})
    }
}


#[derive(Debug, Clone, Copy)]
pub struct TimeIntervalParseError {}
    
impl std::fmt::Display for TimeIntervalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("TimeIntervalParseError").fmt(f)
    }
}

impl std::error::Error for TimeIntervalParseError {}


#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub enum TimeBound {
    /// An unknwon time bound.
    Unbounded,
    /// A timebound coincident with the start or end of another entry.
    BoundRef(EntryId, Endpoint),
    /// A timebound coincident with the another entrie's interval.
    IntervalRef(EntryId),
    /// An time bound estimated to lie within the given period.
    Est(Period)
}


#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub enum Endpoint {
    Start,
    End,
}


#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub struct Period {
    earliest: Time,
    latest: Time,
}

pub type Time = u128;


// Zeller's congruence computes days of the week:
// zeller() {
//     local y=$1 m=$2 d=$3
//     if (( m < 3 )); then
//         m=$((m + 12))
//         y=$((y - 1))
//     fi
//     local K=$((y % 100))
//     local J=$((y / 100))
//     local h=$(( (d + (13*(m+1))/5 + K + K/4 + J/4 + 5*J) % 7 ))
//     local days=(Saturday Sunday Monday Tuesday Wednesday Thursday Friday)
//     echo "${days[$h]}"
// }
