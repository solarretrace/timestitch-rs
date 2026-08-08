////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Runtime application configuration.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::application::LoadStatus;
use crate::application::Format;
use crate::application::TraceConfig;

// External library imports.
use anyhow::Context as _;
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::convert::TryInto as _;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration data. Configures the logger, window, renderer,
/// application limits, and behaviors.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	/// The Config file's load status.
	#[serde(skip)]
	load_status: LoadStatus,

	/// The trace configuration.
	#[serde(default = "Config::default_trace_config")]
	pub trace_config: TraceConfig,
	
	/// The default format for the prefs file.
	#[serde(default = "Config::default_prefs_format")]
	pub prefs_format: Format,

	/// The path for the prefs file.
	#[serde(default = "Config::default_prefs_path")]
	pub prefs_path: PathBuf,
}


impl Default for Config {
	fn default() -> Self {
		Self::new()
	}
}

impl Config {
	/// The default path to look for the [`Config`] file, relative to the app root.
	///
	/// [`Config`]: crate::application::Config
	pub const DEFAULT_CONFIG_PATH: &'static str = ".stall-config";

	/// The default path to look for the [`Prefs`] file, relative to the app
	/// root.
	///
	/// [`Prefs`]: crate::application::Prefs
	pub const DEFAULT_PREFS_PATH: &'static str = ".stall-preferences";

	/// The default format for the config file.
	///
	/// [`Prefs`]: crate::application::Prefs
	pub const DEFAULT_CONFIG_FORMAT: Format = Format::Toml;

	/// The default format for the prefs file.
	///
	/// [`Prefs`]: crate::application::Prefs
	pub const DEFAULT_PREFS_FORMAT: Format = Format::Toml;

	/// Constructs a new `Config` with the default options.
	#[must_use]
	pub fn new() -> Self {
		Self {
			load_status: LoadStatus::default()
				.with_format(Self::DEFAULT_CONFIG_FORMAT),
			trace_config: Self::default_trace_config(),
			prefs_format: Self::default_prefs_format(),
			prefs_path: Self::default_prefs_path(),
		}
	}

	////////////////////////////////////////////////////////////////////////////
	// Accessors
	////////////////////////////////////////////////////////////////////////////

	/// Sets the load path and returns the `Config`.
	#[must_use]
	pub fn with_load_path<P>(mut self, path: P) -> Self
		where P: AsRef<Path>
	{
		self.set_load_path(path);
		self
	}

	/// Sets the file format and returns the `Config`.
	#[must_use]
	pub fn with_format(mut self, format: Format) -> Self	{
		self.set_format(format);
		self
	}

	/// Returns the load path.
	#[must_use]
	pub fn load_path(&self) -> Option<&Path> {
		self.load_status.load_path()
	}

	/// Returns the modification flag.
	#[must_use]
	pub const fn modified(&self) -> bool {
		self.load_status.modified()
	}

	/// Returns the file format.
	#[must_use]
	pub const fn format(&self) -> Option<Format> {
		self.load_status.format()
	}

	/// Sets the load path.
	pub fn set_load_path<P>(&mut self, path: P)
		where P: AsRef<Path>
	{
		self.load_status.set_load_path(path);
	}

	/// Sets the modification flag.
	pub fn set_modified(&mut self, modified: bool) {
		self.load_status.set_modified(modified);
	}

	/// Sets the file format.
	pub fn set_format(&mut self, format: Format) {
		self.load_status.set_format(format);
	}

	////////////////////////////////////////////////////////////////////////////
	// File and serialization methods.
	////////////////////////////////////////////////////////////////////////////

	/// Constructs a new `Config` with options read from the given file path.
	pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
		where P: AsRef<Path>
	{
		let path = path.as_ref();
		let file = File::open(path)
			.with_context(|| format!(
				"Failed to open config file for reading: {}",
				path.display()))?;
		let mut config = Self::read_from_file(file)?;
		config.set_load_path(path);
		Ok(config)
	}

	/// Open a file at the given path and write the `Config` into it.
	pub fn write_to_path<P>(&self, path: P) -> Result<(), Error>
		where P: AsRef<Path>
	{
		let path = path.as_ref();
		let file = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(path)
			.with_context(|| format!(
				"Failed to create/open config file for writing: {}",
				path.display()))?;
		self.write_to_file(file)
			.context("Failed to write config file")?;
		Ok(())
	}
	
	/// Create a new file at the given path and write the `Config` into it.
	pub fn write_to_path_if_new<P>(&self, path: P) -> Result<(), Error>
		where P: AsRef<Path>
	{
		let path = path.as_ref();
		let file = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create_new(true)
			.open(path)
			.with_context(|| format!(
				"Failed to create config file: {}",
				path.display()))?;
		self.write_to_file(file)
			.context("Failed to write config file")?;
		Ok(())
	}

	/// Write the `Config` into the file is was loaded from. Returns true if the
	/// data was written.
	pub fn write_to_load_path(&self) -> Result<bool, Error> {
		match self.load_status.load_path() {
			Some(path) => {
				self.write_to_path(path)?;
				Ok(true)
			},
			None => Ok(false)    
		}
	}

	/// Write the `Config` into a new file using the load path. Returns true
	/// if the data was written.
	pub fn write_to_load_path_if_new(&self) -> Result<bool, Error> {
		match self.load_status.load_path() {
			Some(path) => {
				self.write_to_path_if_new(path)?;
				Ok(true)
			},
			None => Ok(false)
		}
	}

	/// Constructs a new `Config` with options parsed from the given file.
	///
	/// Each file format will be tried in order, and the first to succed will be
	/// returned.
	pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
		let res = Self::parse_toml_from_file(&mut file);
		if res.is_ok() { return res; }

		let next = Self::parse_ron_from_file(&mut file);
		if next.is_ok() { return next; }

		res
	}

	/// Constructs a new `Config` with options parsed from the given file and
	/// format.
	pub fn read_from_file_format(mut file: File, format: Format)
		-> Result<Self, Error> 
	{
		match format {
			Format::Ron  => Self::parse_ron_from_file(&mut file),
			Format::Toml => Self::parse_toml_from_file(&mut file),
		}
	}

	/// Write the `Config` into the given file.
	pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
		self.generate_ron_into_file(&mut file)
	}

	/// Write the `Config` into the given file and format.
	pub fn write_to_file_format(&self, mut file: File, format: Format)
		-> Result<(), Error>
	{
		match format {
			Format::Ron  => self.generate_ron_into_file(&mut file),
			Format::Toml => self.generate_toml_into_file(&mut file),
		}
	}

	////////////////////////////////////////////////////////////////////////////
	// RON format processing.
	////////////////////////////////////////////////////////////////////////////

	/// Parses a `Config` from a file using the RON format.
	fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
		let len = file.metadata()
			.context("Failed to recover file metadata.")?
			.len();
		let mut buf = Vec::with_capacity(len.try_into()?);
		let _ = file.read_to_end(&mut buf)
			.context("Failed to read config file")?;

		Self::parse_ron_from_bytes(&buf[..])
	}

	/// Parses a `Config` from a buffer using the RON format.
	fn parse_ron_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
		use ron::de::Deserializer;
		let mut d = Deserializer::from_bytes(bytes)
			.context("Failed deserializing RON file")?;
		let mut config = Self::deserialize(&mut d)
			.context("Failed parsing RON file")?;
		d.end()
			.context("Failed parsing RON file")?;
		config.load_status.set_format(Format::Ron);
		Ok(config) 
	}

	/// Parses a `Config` from a file using the RON format.
	fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
		tracing::debug!("Serializing & writing Config file (RON).");
		let pretty = ron::ser::PrettyConfig::new()
			.depth_limit(2)
			.separate_tuple_members(true)
			.enumerate_arrays(true)
			.extensions(ron::extensions::Extensions::IMPLICIT_SOME);
		let s = ron::ser::to_string_pretty(&self, pretty)
			.context("Failed to serialize RON file")?;
		let mut writer = BufWriter::new(file);
		writer.write_all(s.as_bytes())
			.context("Failed to write RON file")?;
		writer.flush()
			.context("Failed to flush file buffer")
	}

	////////////////////////////////////////////////////////////////////////////
	// TOML format processing.
	////////////////////////////////////////////////////////////////////////////

	/// Parses a `Config` from a file using the TOML format.
	fn parse_toml_from_file(file: &mut File) -> Result<Self, Error> {
		let len = file.metadata()
			.context("Failed to recover file metadata.")?
			.len();
		let mut buf = Vec::with_capacity(len.try_into()?);
		let _ = file.read_to_end(&mut buf)
			.context("Failed to read config file")?;

		Self::parse_toml_from_bytes(&buf[..])
	}

	/// Parses a `Config` from a buffer using the TOML format.
	fn parse_toml_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
		let mut config: Self = toml::from_slice(bytes)
			.context("Failed deserializing TOML file")?;
		config.load_status.set_format(Format::Toml);

		Ok(config) 
	}

	/// Parses a `Config` from a file using the TOML format.
	fn generate_toml_into_file(&self, file: &mut File) -> Result<(), Error> {
		tracing::debug!("Serializing & writing Config file (TOML).");
		let s = toml::to_string(&self)
			.context("Failed to serialize TOML file")?;
		let mut writer = BufWriter::new(file);
		writer.write_all(s.as_bytes())
			.context("Failed to write TOML file")?;
		writer.flush()
			.context("Failed to flush file buffer")
	}

	////////////////////////////////////////////////////////////////////////////
	// Default constructors for serde.
	////////////////////////////////////////////////////////////////////////////

	/// Returns the default [`TraceConfig`].
	///
	/// [`TraceConfig`]: crate::application::TraceConfig
	fn default_trace_config() -> TraceConfig {
		TraceConfig::default()
	}
	
	/// Returns the default prefs format.
	const fn default_prefs_format() -> Format {
		Self::DEFAULT_PREFS_FORMAT
	}

	/// Returns the default prefs file path.
	fn default_prefs_path() -> PathBuf {
		PathBuf::from(Self::DEFAULT_PREFS_PATH)
	}
}

impl std::fmt::Display for Config {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(fmt, "\ttrace_config.trace_output_path: {:?}",
			self.trace_config.trace_output_path)?;
		writeln!(fmt, "\ttrace_config.ansi_colors: {:?}",
			self.trace_config.ansi_colors)?;
		writeln!(fmt, "\ttrace_config.output_stdout: {:?}",
			self.trace_config.output_stdout)?;
		writeln!(fmt, "\ttrace_config.filters:")?;
		for filter in &self.trace_config.filters {
			writeln!(fmt, "\t\t{:?}", filter)?;
		}
		writeln!(fmt, "\tprefs_format: {:?}", 
			self.prefs_format)?;
		writeln!(fmt, "\tprefs_path: {:?}", 
			self.prefs_path)?;

		Ok(())
	}
}
