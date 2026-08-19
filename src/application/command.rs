////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface module.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use clap::ArgAction;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// CliOpts
////////////////////////////////////////////////////////////////////////////////
/// Command line interface options.
#[deny(missing_docs)]
#[deny(clippy::missing_docs_in_private_items)]
#[derive(Debug, Clone)]
#[derive(Parser)]
#[clap(name = "timestitch")]
#[clap(author, version, about, long_about = None)]
pub struct CliOpts {
	/// The application configuration file to load.
	#[clap(
		long = "config",
		value_parser,
		hide(true))]
	pub config: Option<PathBuf>,

	/// The user preferences file to load.
	#[clap(
		long = "prefs",
		value_parser,
		hide(true))]
	pub prefs: Option<PathBuf>,

	/// When to color output.
	#[clap(
		long = "color",
		default_value = "auto",
		value_enum)]
	pub color: ColorOption,
	
	/// Provide more detailed messages.
	#[clap(
		short = 'v',
		long = "verbose",
		group = "verbosity")]
	pub verbose: bool,

	/// Silence all non-error program output.
	#[clap(
		short = 'q',
		long = "quiet",
		alias = "silent",
		group = "verbosity")]
	pub quiet: bool,

	/// Print trace messages.
	#[clap(
		long = "ztrace",
		hide(true))]
	pub trace: bool,

	/// Fail immediately when an input source cannot be processed.
	#[clap(long = "fast-fail")]
	pub fail_on_error: bool,

	/// Skip processing any non-normal files. (Use --no-dirs to also skip
	/// directories.)
	#[clap(long = "normal_only")]
	pub normal_only: bool,

	/// Scan the contents of provided directories for additional input sources.
	/// (Enabled by default, but can be used to override --no-dirs.)
	// NOTE: This must be manually disabled if no_expand_directories is true.
	#[clap(
		long = "dirs",
		overrides_with = "no_expand_directories",
		default_value_t = true)]
	pub expand_directories: bool,

	/// Skip processing of provided directories. (Disabled by default, but can
	/// be used to override --dirs.)
	#[clap(
		long = "no-dirs",
		overrides_with = "expand_directories")]
	pub no_expand_directories: bool,

	/// Scan the contents of discovered directories recursively for additional
	/// input sources. (Has no effect if --no-dirs is used.)
	// NOTE: This must be manually disabled if expand_directories is false.
	#[clap(
		short = 'r',
		long = "recursive")]
	pub expand_recursive: bool,

	/// The input data sources.
	#[clap(value_parser)]
	pub files: Vec<PathBuf>
}



////////////////////////////////////////////////////////////////////////////////
// ColorOption
////////////////////////////////////////////////////////////////////////////////
/// Color ouput options.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(clap::ValueEnum)]
pub enum ColorOption {
	/// Color usage is automatically determined based on environment variables
	/// and TTY usage.
	Auto,
	/// Color output should always be generated.
	Always,
	/// Color output should never be generated.
	Never,
}

impl ColorOption {
	/// Returns true if colored output should be used.
	#[must_use]
	pub fn enabled(&self) -> bool {
		match self {
			Self::Auto => {
				// Defer to `colored` for enviroment vars and TTY detection.
				colored::control::SHOULD_COLORIZE.should_colorize()
			},
			Self::Always => true,
			Self::Never => false,
		}
	}
}

impl std::str::FromStr for ColorOption {
	type Err = ColorOptionParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.eq_ignore_ascii_case("auto") {
			Ok(Self::Auto)
		} else if s.eq_ignore_ascii_case("always") {
			Ok(Self::Always)
		} else if s.eq_ignore_ascii_case("never") {
			Ok(Self::Never)
		} else {
			Err(ColorOptionParseError)
		}
	}
}

/// An error indicating a failure to parse a [`ColorOption`].
///
/// [`ColorOption`]: ColorOption 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorOptionParseError;

impl std::error::Error for ColorOptionParseError {}

impl std::fmt::Display for ColorOptionParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "failure to parse ColorOption")
	}
}

