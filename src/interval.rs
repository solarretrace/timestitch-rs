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
use std::fmt::Display;
use std::fmt::Debug;


////////////////////////////////////////////////////////////////////////////////
// CalendarSystem
////////////////////////////////////////////////////////////////////////////////
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
// Calendar
////////////////////////////////////////////////////////////////////////////////
/// A representation of a time period suitable for potentially imprecise moments
/// in time.
pub trait Calendar: Debug + Display + Clone + PartialOrd + PartialEq + 'static {
	/// The associated error which can be returned from parsing.
	type ParseErr: std::error::Error;

	/// Parse a calendar value from a string.
	fn from_str(s: &str) -> Result<Self, Self::ParseErr>;

	/// The earliest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn earliest(&self) -> Self;

	/// The latest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn latest(&self) -> Self;
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

impl<C> Default for TimeInterval<C> 
	where C: Calendar
{
	fn default() -> Self {
		Self::unknown()
	}
}

impl<C> TimeInterval<C>
	where C: Calendar
{
	/// The time interval starting and ending at the given calendar periods.
	pub fn new(start: C, end: C) -> Self {
		Self {
			start: start.into(),
			end: end.into(),
		}
	}

	/// An unspecified time interval.
	pub const fn unknown() -> Self {
		Self {
			start: TimeBound::Unbounded,
			end: TimeBound::Unbounded,
		}
	}

	/// The time interval coinciding with the given calendar period.
	pub fn at(time: C) -> Self {
		Self {
			start: time.clone().into(),
			end: time.into(),
		}
	}

	/// The unbounded time interval starting at the given calendar period.
	pub fn from(start: C) -> Self {
		Self {
			start: start.into(),
			end: TimeBound::Unbounded,
		}
	}

	/// The unbounded time interval ending at the given calendar period.
	pub fn to(end: C) -> Self {
		Self {
			start: TimeBound::Unbounded,
			end: end.into(),
		}
	}

}

impl<C> FromStr for TimeInterval<C> 
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	type Err = C::ParseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(C::from_str(s)?.into()))
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
		todo!()
	}
}

impl<C> PartialOrd for TimeInterval<C> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		todo!()
	}
}


/// The endpoint type of a `TimeInterval`.
#[derive(Deserialize, Serialize)]
#[derive(Debug, Clone)]
pub enum Endpoint {
	/// The starting period of the interval.
	Start,
	/// The ending period of the interval.
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
	/// A time bound lying within a single calendar period.
	At { period: C },
	/// A timebound coincident with the another interval.
	IntervalRef { id: EntryId },
	/// A timebound coincident with the start or end of another interval.
	BoundRef { id: EntryId, bound: Endpoint },
}

impl<C> Display for TimeBound<C>
	where C: Calendar
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

impl<C> From<C> for TimeBound<C> {
	fn from(period: C) -> Self {
		Self::At { period }
	}
}
