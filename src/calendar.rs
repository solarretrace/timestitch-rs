////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Record time types.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::clock::TimeFormat;
use crate::gregorian::DateFormat;
use crate::gregorian::GregorianProleptic;
use crate::gregorian::ParseFormat;

// External library imports.
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

// Standard Library imports.
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;


////////////////////////////////////////////////////////////////////////////////
// Calendar
////////////////////////////////////////////////////////////////////////////////
/// A representation of a time period suitable for potentially imprecise moments
/// in time.
pub trait Calendar: Debug + Display + Clone + PartialOrd + PartialEq + 'static {
	/// Data provided for specifying the format requirements of the calendar
	/// value.
	type FormatReq: Clone;

	/// The associated error which can be returned from parsing.
	type ParseErr: std::error::Error;


	/// Parse a calendar value from a string.
	///
	/// The user-provided `FormatReq`, `Regex`, and capture map are provided to
	/// determine the expected format of the input text.
	fn parse_format(
		s: &str,
		req: &Self::FormatReq,
		regex: &Regex,
		capture_map: &HashMap<usize, usize>)
		-> Result<Self, Self::ParseErr>;

	/// The earliest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn earliest(&self) -> Self;

	/// The latest point of the calendar period, resolved to the highest
	/// granularity supported by the calendar.
	fn latest(&self) -> Self;
}


////////////////////////////////////////////////////////////////////////////////
// CalendarSystem
////////////////////////////////////////////////////////////////////////////////
/// Supported parseable calendar types.
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CalendarSystem {
	/// The gregorian proleptic calendar in the given format.
	GregorianProleptic {
		pattern: Box<str>,
		format: <GregorianProleptic as Calendar>::FormatReq
	},
}

impl Default for CalendarSystem {
	fn default() -> Self {
		Self::GregorianProleptic {
			pattern: ".*".to_string().into_boxed_str(),
			format: ParseFormat {
				date: DateFormat::Ymd,
				time: TimeFormat::Hours24,
			},
		}
	}
}
