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
use crate::Calendar;

// External library imports.
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

// Standard Library imports.
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;



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

	pub fn parse_format(
		text: &str,
		format: &C::FormatReq,
		re: &Regex,
		capture_map: &HashMap<usize, usize>)
		-> Result<Self, C::ParseErr>
	{
		Ok(Self::from(C::parse_format(text, format, re, capture_map)?.into()))
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
