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
use either::Either;
use regex::Regex;
use regex::Captures;

// Standard library imports.
use std::collections::BTreeMap;
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
	calendar_pattern: Regex,
	calendar_parse_fmt: C::FormatReq,
	paths: I)
	-> Result<Vec<Entry<C>>, Error>
	where
		I: IntoIterator<Item=&'a Path>,
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	// Compile resources.
	let re_all: Regex = Regex::new(".*").unwrap();
	let path_source_pattern = prefs
		.path_source_pattern
		.as_ref()
		.map(|m| Regex::new(m))
		.transpose()?;
	let content_split_pattern = prefs
		.content_split_pattern
		.as_ref()
		.map(|m| Regex::new(m))
		.transpose()?;
	let content_source_patterns = prefs
		.content_source_patterns
		.iter()
		.map(|matcher| matcher
			.as_deref()
			.map(Regex::new)
			.unwrap_or(Ok(re_all.clone())))
		.collect::<Result<Vec<_>, _>>()?;

	let mut res = Resources {
		path_source_pattern: path_source_pattern.unwrap_or(re_all),
		content_split_pattern,
		content_source_patterns,
		calendar_pattern,
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
	let path_captures = res.path_source_pattern
		.captures(&path_str)
		.expect("construct path capture groups");

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
			path_captures, 
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
	source_str: &str,
	path_captures: Captures<'_>,
	text: &str,
	prefs: &Prefs,
	entries: &mut Vec<Entry<C>>,
	res: &mut Resources<C::FormatReq>)
	-> Result<(), Error>
	where
		C: Calendar,
		C::ParseErr: Send + Sync + 'static
{
	// Split the file text into its data source sections.
	let split_contents = match res.content_split_pattern.as_ref() {
		Some(re) => Either::Left(re.split(text)),
		None     => Either::Right(std::iter::once(text)),
	};

	// TODO: We only handle emitting a single entry from the file. How do we
	// construct multiple entries from the same file? When do we try this?
	for source in split_contents {
		let line_captures: Vec<Option<_>> = source.lines()
			.zip(res.content_source_patterns.iter())
			.map(|(l, re)| re.captures(l))
			.collect();

		// Extract the Entry ID.
		let id = match prefs.entry_id_source {
			MatchSource::Default => entries.len().try_into()?,
			MatchSource::Path { group } => {
				path_captures
					.get(group)
					.ok_or(anyhow!("invalid path capture group"))?
					.as_str()
					.parse::<u64>()?
			},
			MatchSource::Content { line, group } => {
				line_captures
					.get(line)
					.ok_or(anyhow!("invalid line index"))?
					.as_ref()
					.ok_or(anyhow!("line capture match failed"))?
					.get(group)
					.ok_or(anyhow!("invalid line capture group"))?
					.as_str()
					.parse::<u64>()?
			},
		};

		// Extract the Entry TimeInterval.
		let time = match prefs.entry_time_source {
			MatchSource::Default => TimeInterval::<C>::unknown(),
			MatchSource::Path { group } => {
				TimeInterval::parse_format(
					path_captures
						.get(group)
						.ok_or(anyhow!("invalid path capture group"))?
						.as_str(),
					todo!(),
					todo!(),
					todo!())?
			},
			MatchSource::Content { line, group } => {
				TimeInterval::parse_format(
					line_captures
						.get(line)
						.ok_or(anyhow!("invalid line index"))?
						.as_ref()
						.ok_or(anyhow!("line capture match failed"))?
						.get(group)
						.ok_or(anyhow!("invalid line capture group"))?
						.as_str(),
					todo!(),
					todo!(),
					todo!())?
			},
		};

		// Extract the Entry source file.
		let source_path = DataSource(source_str.into());

		// Extract the Entry source ref.
		let source_ref = match prefs.entry_ref_source {
			None          |
			Some(MatchSource::Default) => String::new().into_boxed_str(),
			Some(MatchSource::Path { group }) => {
				path_captures
					.get(group)
					.ok_or(anyhow!("invalid path capture group"))?
					.as_str()
					.to_owned()
					.into_boxed_str()
			},
			Some(MatchSource::Content { line, group }) => {
				line_captures
					.get(line)
					.ok_or(anyhow!("invalid line index"))?
					.as_ref()
					.ok_or(anyhow!("line capture match failed"))?
					.get(group)
					.ok_or(anyhow!("invalid line capture group"))?
					.as_str()
					.to_owned()
					.into_boxed_str()
			},
		};

		// Extract the Entry attributes.
		let mut attributes = BTreeMap::new();
		for (key, msa) in prefs.entry_attribute_sources.iter() {
			let attribute = match *msa {
				MatchSourceAttribute::Path { group, format } => {
					format.parse_dyn(
						path_captures
							.get(group)
							.ok_or(anyhow!("invalid path capture group"))?
							.as_str())?
						
				},
				MatchSourceAttribute::Content { line, group, format } => {
					format.parse_dyn(
						line_captures
							.get(line)
							.ok_or(anyhow!("invalid line index"))?
							.as_ref()
							.ok_or(anyhow!("line capture match failed"))?
							.get(group)
							.ok_or(anyhow!("invalid line capture group"))?
							.as_str())?
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
		})
	}
	Ok(())
}


////////////////////////////////////////////////////////////////////////////////
// Resources
////////////////////////////////////////////////////////////////////////////////
/// Resources used for processing entries.
#[derive(Debug, Clone)]
struct Resources<P> {
	/// The compiled path pattern.
	pub path_source_pattern: Regex,
	/// The compiled content splitter pattern.
	pub content_split_pattern: Option<Regex>,
	/// The compiled content line patterns.
	pub content_source_patterns: Vec<Regex>,
	/// The compiled calendar pattern.
	pub calendar_pattern: Regex,
	/// The calendar value parse format information.
	pub calendar_parse_fmt: P,
}
