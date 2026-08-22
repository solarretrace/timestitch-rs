////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Data processing module.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::application::Config;
use crate::application::Prefs;
use crate::application::CliOpts;
use crate::Entry;
use crate::DataSource;
use crate::MatchSource;
use crate::MatchSourceAttribute;
use crate::TimeInterval;
use crate::Calendar;

// External library imports.
use anyhow::anyhow;
use anyhow::Context as _;
use anyhow::Error;
use regex::Regex;

// Standard library imports.
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// process_args
////////////////////////////////////////////////////////////////////////////////
/// Reads `Entry`s from an iterator of files according to the `Entry` data
/// schema provided by `Prefs`.
///
/// This is the main data extraction function of the application.
pub fn process_args<'a, I, C>(
	_config: &Config,
	prefs: &Prefs,
	opts: &CliOpts,
	errors: &mut Vec<Error>,
	calendar_regex: Regex,
	calendar_regex_map: HashMap<usize, usize>,
	calendar_parse_fmt: C::FormatReq,
	paths: I)
	-> Result<Vec<Entry<C>>, Error>
	where
		I: IntoIterator<Item=&'a Path>,
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	// Compile resources.
	let mut res = Resources {
		calendar_regex,
		calendar_regex_map,
		calendar_parse_fmt,
	};

	// Generate entries from the provided paths.
	let mut entries = Vec::new();
	for path in paths {
		process_path(
			path.as_ref(),
			prefs,
			opts,
			&mut entries,
			errors,
			&mut res,
			false)?;
	}

	Ok(entries)
}

/// Generates entries from a path.
fn process_path<C>(
	path: &Path,
	prefs: &Prefs,
	opts: &CliOpts,
	entries: &mut Vec<Entry<C>>,
	errors: &mut Vec<Error>,
	res: &mut Resources<C::FormatReq>,
	is_recursive_dir: bool)
	-> Result<(), Error>
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	let mut file = match File::open(path)
		.with_context(|| format!("Failed to open file for reading: {}",
			path.display()))
	{
		Ok(f) => f,
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	};
	let file_type = match file.metadata()
		.context("Failed to read file metadata.")
	{
		Ok(m) => m.file_type(),
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	};
	
	if file_type.is_dir() 
		&& ((opts.expand_recursive && is_recursive_dir)
			|| (opts.expand_directories && !is_recursive_dir))
	{
		process_directory(
			path,
			prefs,
			opts,
			entries,
			errors,
			res)
	} else if file_type.is_file() || !opts.normal_only {
		process_file(
			path,
			&mut file,
			prefs,
			opts,
			entries,
			errors,
			res)
	} else {
		// Skip processing this path.
		Ok(())
	}
}

/// Generates entries from a directory.
///
/// The `is_recursive_dir` parameter informs us if we are in a recursively
/// discovered directory or an explicitly provided directory.
fn process_directory<C>(
	path: &Path,
	prefs: &Prefs,
	opts: &CliOpts,
	entries: &mut Vec<Entry<C>>,
	errors: &mut Vec<Error>,
	res: &mut Resources<C::FormatReq>)
	-> Result<(), Error>
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	let read_dir = match std::fs::read_dir(path)
		.context("Failed to read directory.")
	{
		Ok(p) => p,
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	};
	for dir_entry in read_dir {
		let entry = match dir_entry
			.context("Failed to read directory entry.")
		{
			Ok(p) => p,
			Err(e) if opts.fail_on_error => return Err(e),
			Err(e) => {
				errors.push(e);
				return Ok(());
			},
		};

		process_path(
			entry.path().as_ref(),
			prefs,
			opts,
			entries,
			errors,
			res,
			true)?;
	}
	Ok(())
}

/// Generates entries from a non-directory file.
fn process_file<C>(
	path: &Path,
	file: &mut File,
	prefs: &Prefs,
	opts: &CliOpts,
	entries: &mut Vec<Entry<C>>,
	errors: &mut Vec<Error>,
	res: &mut Resources<C::FormatReq>)
	-> Result<(), Error>
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	// Setup matching for the file path.
	let path_str = path.to_string_lossy();

	// Setup matching for the file contents.
	let len = match file.metadata()
		.context("Failed to read file metadata.")
	{
		Ok(m) => m.len(),
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	};
	let mut buf = Vec::with_capacity(len.try_into()
		.expect("convert file length to usize"));
	match file.read_to_end(&mut buf)
		.context("Failed to read entry file")
	{
		Ok(_) => { /* Do nothing. */ },
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	}
	let text = match String::from_utf8(buf)
		.context("Failed to read file as valid UTF-8")
	{
		Ok(s) => s,
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			return Ok(());
		},
	};

	// Process entries.
	match process_entries(
			path_str.as_ref(),
			&text,
			prefs,
			entries,
			res)
		.context("Failed to read entry")
	{
		Ok(_) => { Ok(()) },
		Err(e) if opts.fail_on_error => return Err(e),
		Err(e) => {
			errors.push(e);
			Ok(())
		},
	}
}


fn process_entries<C>(
	path_str: &str,
	text: &str,
	prefs: &Prefs,
	entries: &mut Vec<Entry<C>>,
	res: &mut Resources<C::FormatReq>)
	-> Result<(), Error>
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	println!("{:?}", path_str);
	
	// Split the file text into its data source sections.
	// let split_contents = match res.content_split_regex.as_ref() {
	// 	Some(re) => Either::Left(re.split(text)),
	// 	None     => Either::Right(std::iter::once(text)),
	// };

	// TODO: We only handle emitting a single entry from the file. How do we
	// construct multiple entries from the same file? When do we try this?
	
	let lines: Vec<_> = text.lines().collect();

	// Extract the Entry ID.
	let id = match &prefs.entry_id_source {
		MatchSource::Default => entries.len().try_into()?,
		MatchSource::Path { pattern } => {
			pattern
				.captures(path_str)
				.ok_or(anyhow!("Entry ID: invalid path regex"))?
				.get(1)
				.ok_or(anyhow!("Entry ID: invalid path regex group"))?
				.as_str()
				.parse::<u64>()?
		},
		MatchSource::Content { line, pattern } => {
			let line = lines.get(*line)
				.ok_or(anyhow!("Entry ID: invalid line index"))?;
			pattern
				.captures(line)
				.ok_or(anyhow!("Entry ID: invalid line regex"))?
				.get(1)
				.ok_or(anyhow!("Entry ID: invalid line regex group"))?
				.as_str()
				.parse::<u64>()?
		},
	};

	// Extract the Entry TimeInterval.
	let time = match &prefs.entry_time_source {
		MatchSource::Default => TimeInterval::<C>::unknown(),
		MatchSource::Path { pattern } => {
			TimeInterval::parse_format(
				pattern
					.captures(path_str)
					.ok_or(anyhow!("Entry time interval: invalid path regex"))?
					.get(1)
					.ok_or(anyhow!("Entry time interval: invalid path regex /
						group"))?
					.as_str(),
				&res.calendar_parse_fmt,
				&res.calendar_regex,
				&res.calendar_regex_map)?
		},
		MatchSource::Content { line, pattern } => {
			let line = lines.get(*line)
				.ok_or(anyhow!("Entry time interval: invalid line index"))?;
			TimeInterval::parse_format(
				pattern
					.captures(line)
					.ok_or(anyhow!("Entry time interval: invalid line regex"))?
					.get(1)
					.ok_or(anyhow!("Entry time interval: invalid line regex \
						group"))?
					.as_str(),
				&res.calendar_parse_fmt,
				&res.calendar_regex,
				&res.calendar_regex_map)?
		},
	};

	// Extract the Entry source file.
	let source_path = DataSource(path_str.into());

	// Extract the Entry source ref.
	let source_ref = match &prefs.entry_ref_source {
		None                       |
		Some(MatchSource::Default) => String::new().into_boxed_str(),
		Some(MatchSource::Path { pattern }) => {
			pattern
				.captures(path_str)
				.ok_or(anyhow!("Entry source ref: invalid path regex"))?
				.get(1)
				.ok_or(anyhow!("Entry source ref: invalid path regex group"))?
				.as_str()
				.to_owned()
				.into_boxed_str()
		},
		Some(MatchSource::Content { line, pattern }) => {
			let line = lines.get(*line)
				.ok_or(anyhow!("Entry source ref: invalid line index"))?;
			pattern
				.captures(line)
				.ok_or(anyhow!("Entry source ref: invalid line regex"))?
				.get(1)
				.ok_or(anyhow!("Entry source ref: invalid line regex \
					group"))?
				.as_str()
				.to_owned()
				.into_boxed_str()
		},
	};

	// Extract the Entry attributes.
	let mut attributes = BTreeMap::new();
	for (key, msa) in prefs.entry_attribute_sources.iter() {
		let attribute = match msa {
			MatchSourceAttribute::Path { pattern, format } => {
				let s = pattern
					.captures(path_str)
					.ok_or(anyhow!("Entry attribute: invalid path regex"))?
					.get(1)
					.ok_or(anyhow!("Entry attribute: invalid path regex \
						group"))?
					.as_str();

				format.parse_dyn(s)?
			},
			MatchSourceAttribute::Content { line, pattern, format } => {
				let line = lines.get(*line)
					.ok_or(anyhow!("Entry attribute: invalid line index"))?;
				let s = pattern
					.captures(line)
					.ok_or(anyhow!("Entry attribute: invalid line regex"))?
					.get(1)
					.ok_or(anyhow!("Entry attribute: invalid line regex \
						group"))?
					.as_str();

				format.parse_dyn(s)?
			},
		};
		attributes.insert(key.clone(), attribute);
	}

	entries.push(Entry {
		id,
		time,
		source_path,
		source_ref,
		attributes,
	});
	Ok(())
}


////////////////////////////////////////////////////////////////////////////////
// Resources
////////////////////////////////////////////////////////////////////////////////
/// Resources used for processing entries.
#[derive(Debug, Clone)]
struct Resources<P> {
	/// The compiled calendar pattern.
	pub calendar_regex: Regex,
	/// The calendar pattern capture group map.
	pub calendar_regex_map: HashMap<usize, usize>,
	/// The calendar value parse format information.
	pub calendar_parse_fmt: P,
}
