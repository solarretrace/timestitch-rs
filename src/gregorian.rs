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
use crate::ClockTime;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use regex::Regex;
pub use chrono::Weekday;
pub use chrono::NaiveDate;
pub use chrono::ParseWeekdayError;
pub use chrono::Datelike as _;

// Standard Library imports.
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
	/// The year, month, and day.
	Ymd {
		/// The Gregorian proleptic calendar year.
		year: i32,
		/// The index of the month within the year.
		month: Option<u32>,
		/// The index of the day within the month.
		day: Option<u32>,
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
	/// The ISO week date: year, week number (0-indexed), day of week.
	Ywd {
		/// The Gregorian proleptic calendar year.
		year: i32,
		/// The index of the week.
		week: u32,
		/// The day of the week.
		weekday: Option<Weekday>,
		/// An optional clock time resolution.
		time: Option<ClockTime>,
	},
	/// The number of days since 0001-01-01 in the Gregorian proleptic calendar.
	CeDay {
		/// The index of the day relative to the start of the current era.
		days: i32,
		/// An optional clock time resolution.
		time: Option<ClockTime>,
	},
	/// The number of days since 1970-01-01 in the Gregorian proleptic calendar.
	EpochDay {
		/// The index of the day relative to the start of the UNIX epoch.
		days: i32,
		/// An optional clock time resolution.
		time: Option<ClockTime>,
	},
	/// The year, month, weekday, and index of that weekday within the month.
	Ymwn {
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


impl GregorianProleptic {
	fn try_from_raw(raw: GregorianProlepticRaw)
		-> Result<Self, GregorianProlepticParseError> 
	{
		use GregorianProleptic::*;
		raw.format.validate_capture_group_count(&raw.re)?;
		let cap = raw.re.captures(&raw.text)
			.ok_or_else(|| GregorianProlepticParseError::CaptureMatchFailure(
				raw.text.clone()))?;
		match raw.format {
			Format::Ymd => {
				let year: i32 = cap.get(1).unwrap().as_str().parse()?;
				let month: Option<u32> = cap.get(2).map(|g| g.as_str().parse())
					.transpose()?;
				let day: Option<u32> = cap.get(3).map(|g| g.as_str().parse())
					.transpose()?;
				let h = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(6).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(Ymd { year, month, day, time })
			},
			Format::Yo => {
				let year: i32 = cap.get(1).unwrap().as_str().parse()?;
				let ordinal: Option<u32> = cap.get(2)
					.map(|g| g.as_str().parse())
					.transpose()?;
				if ordinal.is_none() {
					return Ok(GregorianProleptic::Ymd {
						year,
						month: None,
						day: None,
						time: None,
					});
				}
				let ordinal = ordinal.unwrap();
				let h = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(Yo { year, ordinal, time })
			},
			Format::Ywd => {
				let year: i32 = cap.get(1).unwrap().as_str().parse()?;
				let week: Option<u32> = cap.get(2).map(|g| g.as_str().parse())
					.transpose()?;
				if week.is_none() {
					return Ok(GregorianProleptic::Ymd {
						year,
						month: None,
						day: None,
						time: None,
					});
				}
				let week = week.unwrap();
				let weekday: Option<Weekday> = cap.get(3)
					.map(|g| g.as_str().parse())
					.transpose()?;
				let h = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(6).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(Ywd { year, week, weekday, time })
			},
			Format::CeDay => {
				let days: i32 = cap.get(1).unwrap().as_str().parse()?;
				let h = cap.get(2).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(CeDay { days, time })
			},
			Format::EpochDay => {
				let days: i32 = cap.get(1).unwrap().as_str().parse()?;
				let h = cap.get(2).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(3).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(4).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(EpochDay { days, time })
			},
			Format::Ymwn => {
				let year: i32 = cap.get(1).unwrap().as_str().parse()?;
				let month: Option<u32> = cap.get(2).map(|g| g.as_str().parse())
					.transpose()?;
				if month.is_none() {
					return Ok(GregorianProleptic::Ymd {
						year,
						month: None,
						day: None,
						time: None,
					});
				}
				let weekday: Option<Weekday> = cap.get(3)
					.map(|g| g.as_str().parse())
					.transpose()?;
				if weekday.is_none() {
					return Ok(GregorianProleptic::Ymd {
						year,
						month,
						day: None,
						time: None,
					});
				}
				let month = month.unwrap();
				let weekday = weekday.unwrap();
				let n: u8 = cap.get(4).unwrap().as_str().parse()?;
				let h = cap.get(5).map(|g| g.as_str().parse()).transpose()?;
				let m = cap.get(6).map(|g| g.as_str().parse()).transpose()?;
				let s = cap.get(7).map(|g| g.as_str().parse()).transpose()?;
				let time = ClockTime::from_hms_opt(h, m, s);

				Ok(Ymwn { year, month, weekday, n, time })
			},
		}
	}
}

impl From<GregorianProleptic> for TimeInterval<GregorianProleptic> {
	fn from(period: GregorianProleptic) -> Self {
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
	type ParseErr = GregorianProlepticParseError;

	fn from_str(s: &str) -> Result<Self, Self::ParseErr> {
		todo!()
	}

	fn earliest(&self) -> Self {
		use GregorianProleptic::*;
		match *self {
			Ymd { year, month, day, time } => {
				Ymd {
					year,
					month: month.or(Some(0)),
					day: day.or(Some(0)),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
			Yo { year, ordinal, time } => {
				let dt = NaiveDate::from_yo_opt(year, ordinal)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
			Ywd { year, week, weekday, time } => {
				let weekday = weekday.unwrap_or(Weekday::Mon);
				let dt = NaiveDate::from_isoywd_opt(year, week + 1, weekday)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
			CeDay { days, time } => {
				let dt = NaiveDate::from_num_days_from_ce_opt(days)
					.expect("convert period to NaiveDate");
				Ymd {
					year: dt.year(),
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
			EpochDay { days, time } => {
				let dt = NaiveDate::from_epoch_days(days)
					.expect("convert period to NaiveDate");
				Ymd {
					year: dt.year(),
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
			Ymwn { year, month, weekday, n, time } => {
				let dt = NaiveDate::from_weekday_of_month_opt(
						year,
						month,
						weekday,
						n)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(month),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MIN, |t| t.extend(ClockTime::MIN))),
				}
			},
		}
	} 

	fn latest(&self) -> Self {
		use GregorianProleptic::*;
		match *self {
			Ymd { year, month, day, time } => {
				let month = month.unwrap_or(11);
				let day = day
					.or_else(||
						Some(NaiveDate::from_ymd_opt(year, month, 1)
					.expect("get NaiveDate for year & month")
					.num_days_in_month() as u32));
				Ymd {
					year,
					month: Some(month),
					day,
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
			Yo { year, ordinal, time } => {
				let dt = NaiveDate::from_yo_opt(year, ordinal)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
			Ywd { year, week, weekday, time } => {
				let weekday = weekday.unwrap_or(Weekday::Sun);
				let dt = NaiveDate::from_isoywd_opt(year, week + 1, weekday)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
			CeDay { days, time } => {
				let dt = NaiveDate::from_num_days_from_ce_opt(days)
					.expect("convert period to NaiveDate");
				Ymd {
					year: dt.year(),
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
			EpochDay { days, time } => {
				let dt = NaiveDate::from_epoch_days(days)
					.expect("convert period to NaiveDate");
				Ymd {
					year: dt.year(),
					month: Some(dt.month0()),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
			Ymwn { year, month, weekday, n, time } => {
				let dt = NaiveDate::from_weekday_of_month_opt(
						year,
						month,
						weekday,
						n)
					.expect("convert period to NaiveDate");
				Ymd {
					year,
					month: Some(month),
					day: Some(dt.day0()),
					time: Some(time
						.map_or(ClockTime::MAX, |t| t.extend(ClockTime::MAX))),
				}
			},
		}
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
/// The parsed time format.
pub enum Format {
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
	Ymwn,
}

impl Format {
	fn validate_capture_group_count(&self, re: &Regex)
		-> Result<usize, CaptureGroupCountError>
	{
		let len = re.captures_len();
		let (min, max) = match self {
			Format::Ymd          => (2, 7),
			Format::Yo           => (2, 6),
			Format::Ywd          => (2, 7),
			Format::CeDay        => (2, 5),
			Format::EpochDay     => (2, 5),
			Format::Ymwn => if len == 4 {
				// 4 is specifically disallowed here, recommend adding more.
				return Err(CaptureGroupCountError {
					format: *self,
					min: 5,
					max: 8,
				});
			} else {
				(2, 8)
			},
		};
		if !(min..=max).contains(&len) {
			let min: u8 = min.try_into().unwrap();
			let max: u8 = max.try_into().unwrap();
			Err(CaptureGroupCountError { format: *self, min, max })
		} else {
			Ok(len)
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




////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn earliest_ymd_y() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = Ymd { year: 2000, month: None, day: None, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(0), day: Some(0), time: Some(t0) });
	}

	#[test]
	fn earliest_ymd_ym() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = Ymd { year: 2000, month: Some(4), day: None, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(4), day: Some(0), time: Some(t0) });
	}

	#[test]
	fn earliest_ymd_ymd() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time: Some(t0) });
	}

	#[test]
	fn earliest_ymd_ymdh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_h(18));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn earliest_ymd_ymdhm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hm(18, 8));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn earliest_ymd_ymdhms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hms(18, 8, 59));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn latest_ymd_y() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = Ymd { year: 2000, month: None, day: None, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(11), day: Some(30), time: Some(t1) });
	}

	#[test]
	fn latest_ymd_ym() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = Ymd { year: 2000, month: Some(4), day: None, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(4), day: Some(30), time: Some(t1) });
	}

	#[test]
	fn latest_ymd_ymd() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time: Some(t1) });
	}

	#[test]
	fn latest_ymd_ymdh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_h(18));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn latest_ymd_ymdhm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hm(18, 8));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn latest_ymd_ymdhms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hms(18, 8, 59));
		let dt = Ymd { year: 2000, month: Some(4), day: Some(15), time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(4), day: Some(15), time });
	}

	#[test]
	fn earliest_yo_yo() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = Yo { year: 2000, ordinal: 65, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time: Some(t0) });
	}

	#[test]
	fn earliest_yo_yoh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_h(18));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn earliest_yo_yohm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hm(18, 50));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn earliest_yo_yohms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hms(18, 50, 1));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn latest_yo_yo() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = Yo { year: 2000, ordinal: 65, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time: Some(t1) });
	}

	#[test]
	fn latest_yo_yoh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_h(18));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn latest_yo_yohm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hm(18, 50));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn latest_yo_yohms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hms(18, 50, 1));
		let dt = Yo { year: 2000, ordinal: 65, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(2), day: Some(4), time });
	}

	#[test]
	fn earliest_ywd_yw() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = Ywd { year: 2000, week: 40, weekday: None, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(9), day: Some(8), time: Some(t0) });
	}

	#[test]
	fn earliest_ywd_ywd() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Some(Weekday::Tue);
		let dt = Ywd { year: 2000, week: 40, weekday, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time: Some(t0) });
	}

	#[test]
	fn earliest_ywd_ywdh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_h(2));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn earliest_ywd_ywdhm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_hm(2, 0));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn earliest_ywd_ywdhms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_hms(2, 0, 59));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn latest_ywd_yw() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = Ywd { year: 2000, week: 40, weekday: None, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(9), day: Some(14), time: Some(t1) });
	}

	#[test]
	fn latest_ywd_ywd() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Some(Weekday::Tue);
		let dt = Ywd { year: 2000, week: 40, weekday, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time: Some(t1) });
	}

	#[test]
	fn latest_ywd_ywdh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_h(2));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn latest_ywd_ywdhm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_hm(2, 0));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn latest_ywd_ywdhms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Some(Weekday::Tue);
		let t = Some(ClockTime::from_hms(2, 0, 59));
		let dt = Ywd { year: 2000, week: 40, weekday, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2000, month: Some(9), day: Some(9), time });
	}

	#[test]
	fn earliest_ce_d() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = CeDay { days: 741832, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time: Some(t0) });
	}

	#[test]
	fn earliest_ce_dh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_h(23));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn earliest_ce_dhm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hm(23, 15));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn earliest_ce_dhms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hms(23, 15, 59));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn latest_ce_d() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = CeDay { days: 741832, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time: Some(t1) });
	}

	#[test]
	fn latest_ce_dh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_h(23));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn latest_ce_dhm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hm(23, 15));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn latest_ce_dhms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hms(23, 15, 59));
		let dt = CeDay { days: 741832, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 2032, month: Some(0), day: Some(24), time });
	}

	#[test]
	fn earliest_epoch_d() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let dt = EpochDay { days: -2189, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time: Some(t0) });
	}

	#[test]
	fn earliest_epoch_dh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_h(23));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn earliest_epoch_dhm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hm(23, 15));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn earliest_epoch_dhms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let t = Some(ClockTime::from_hms(23, 15, 59));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn latest_epoch_d() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let dt = EpochDay { days: -2189, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time: Some(t1) });
	}

	#[test]
	fn latest_epoch_dh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_h(23));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn latest_epoch_dhm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hm(23, 15));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn latest_epoch_dhms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let t = Some(ClockTime::from_hms(23, 15, 59));
		let dt = EpochDay { days: -2189, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 1964, month: Some(0), day: Some(3), time });
	}

	#[test]
	fn earliest_ymwn_ymwn() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Weekday::Fri;
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: None };
		assert_eq!(
			dt.earliest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time: Some(t0) });
	}

	#[test]
	fn earliest_ymwn_ymwnh() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_h(4));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}

	#[test]
	fn earliest_ymwn_ymwnhm() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_hm(4, 5));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}

	#[test]
	fn earliest_ymwn_ymwnhms() {
		use GregorianProleptic::*;
		let t0 = ClockTime::MIN;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_hms(4, 5, 6));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t0));
		assert_eq!(
			dt.earliest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}

	#[test]
	fn latest_ymwn_ymwn() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Weekday::Fri;
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: None };
		assert_eq!(
			dt.latest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time: Some(t1) });
	}

	#[test]
	fn latest_ymwn_ymwnh() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_h(4));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}

	#[test]
	fn latest_ymwn_ymwnhm() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_hm(4, 5));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}

	#[test]
	fn latest_ymwn_ymwnhms() {
		use GregorianProleptic::*;
		let t1 = ClockTime::MAX;

		let weekday = Weekday::Fri;
		let t = Some(ClockTime::from_hms(4, 5, 6));
		let dt = Ymwn { year: 80, month: 6, weekday, n: 2, time: t };
		let time = t.map(|t| t.extend(t1));
		assert_eq!(
			dt.latest(),
			Ymd { year: 80, month: Some(6), day: Some(13), time });
	}
}
