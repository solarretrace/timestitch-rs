////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File source and modification tracking.
////////////////////////////////////////////////////////////////////////////////


// Standard library imports.
use std::path::Path;
use std::path::PathBuf;



////////////////////////////////////////////////////////////////////////////////
// Format
////////////////////////////////////////////////////////////////////////////////
/// The loaded file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Format {
	/// RON format.
	Ron,
	/// TOML format.
	Toml,
}


////////////////////////////////////////////////////////////////////////////////
// LoadStatus
////////////////////////////////////////////////////////////////////////////////
/// Structure for tracking a file's load status.
#[derive(Debug, Clone)]
pub struct LoadStatus {
	/// The path the data was initially loaded from.
	load_path: Option<PathBuf>,
	/// Whether the data has been modified since last save.
	modified: bool,
	/// The loaded file format.
	format: Option<Format>,
}

impl Default for LoadStatus {
	fn default() -> Self {
		Self::new()   
	}
}

impl LoadStatus {
	/// Constructs a new `LoadStatus`.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			load_path: None,
			modified: false,
			format: None,
		}
	}

	/// Sets the load path and returns the `LoadStatus`.
	#[must_use]
	pub fn with_load_path<P>(mut self, path: P) -> Self
		where P: AsRef<Path>
	{
		self.set_load_path(path);
		self
	}

	/// Sets the modification flag and returns the `LoadStatus`.
	#[must_use]
	pub fn with_modified(mut self, modified: bool) -> Self
	{
		self.set_modified(modified);
		self
	}

	/// Sets the format and returns the `LoadStatus`
	#[must_use]
	pub fn with_format(mut self, format: Format) -> Self
	{
		self.set_format(format);
		self
	}

	/// Returns the `LoadStatus`'s load path.
	#[must_use]
	pub fn load_path(&self) -> Option<&Path> {
		self.load_path.as_ref().map(AsRef::as_ref)
	}

	/// Returns the value of the modification flag.
	#[must_use]
	pub const fn modified(&self) -> bool {
		self.modified
	}

	/// Returns the file format.
	#[must_use]
	pub const fn format(&self) -> Option<Format> {
		self.format
	}

	/// Clears the `LoadStatus`'s load path.
	pub fn clear_load_path<P>(&mut self) {
		self.load_path = None;
	}

	/// Sets the `LoadStatus`'s load path.
	pub fn set_load_path<P>(&mut self, path: P)
		where P: AsRef<Path>
	{
		self.load_path = Some(path.as_ref().to_owned());
	}

	/// Sets the modification flag.
	pub fn set_modified(&mut self, modified: bool) {
		self.modified = modified;
	}

	/// Sets file format.
	pub fn set_format(&mut self, format: Format) {
		self.format = Some(format);
	}
}
