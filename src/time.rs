////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Record time types.
//!
//! 
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::EntryId;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard Library imports.
use std::str::FromStr;
use std::fmt::Display;
use std::fmt::Debug;


////////////////////////////////////////////////////////////////////////////////
// Calendar
////////////////////////////////////////////////////////////////////////////////
/// A representation of a time period suitable for potentially imprecise moments
/// in time.
pub trait Calendar: Debug + Display 
    + PartialOrd + PartialEq
    + FromStr + Into<TimeInterval<Self>>
    + 'static
{
	/// The earliest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn earliest(&self) -> Self;

	/// The latest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn latest(&self) -> Self;
}

/// Supported parseable calendar types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum CalendarSystem {
	/// The gregorian proleptic calendar.
	GregorianProleptic,
}

impl Default for CalendarSystem {
	fn default() -> Self {
		Self::GregorianProleptic
	}
}

////////////////////////////////////////////////////////////////////////////////
// TimeInterval
////////////////////////////////////////////////////////////////////////////////
/// An interval of time represented in some calendar.
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub struct TimeInterval<C> {
	/// The start of the interval.
	pub start: TimeBound<C>,
	/// The end of the interval.
	pub end: TimeBound<C>,
}

impl<C> TimeInterval<C>
	where C: Calendar
{
	

	pub const fn unknown() -> Self {
		Self {
			start: TimeBound::Unbounded,
			end: TimeBound::Unbounded,
		}
	}
}

impl<C> FromStr for TimeInterval<C> 
	where
		C: Calendar,
		<C as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static
{
	type Err = <C as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(<C as FromStr>::from_str(s)?.into()))
	}
}

impl<C> Display for TimeInterval<C>
	where C: Calendar
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&format!("{} - {}", self.start, self.end), f)
	}
}

impl<C> PartialEq for TimeInterval<C> {
	fn eq(&self, other: &Self) -> bool {
		false
	}
}

impl<C> PartialOrd for TimeInterval<C> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		None
	}
}


/// The endpoint type of a `TimeInterval`.
#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub enum Endpoint {
	Start,
	End,
}

////////////////////////////////////////////////////////////////////////////////
// TimeBound
////////////////////////////////////////////////////////////////////////////////
/// A `TimeInterval` bound represented in some calendar.
#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub enum TimeBound<C> {
	/// An unknwon time bound.
	Unbounded,
	/// An time bound estimated to lie within the given period.
	Between { earliest: C, latest: C },
	/// A timebound coincident with the start or end of another entry.
	BoundRef { id: EntryId, bound: Endpoint },
	/// A timebound coincident with the another entrie's interval.
	IntervalRef { id: EntryId },
}

impl<C> Display for TimeBound<C>
	where C: Calendar
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}



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



