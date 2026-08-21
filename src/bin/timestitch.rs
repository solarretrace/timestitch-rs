////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use timestitch::application::CliOpts;
use timestitch::application::Config;
use timestitch::application::Prefs;
use timestitch::application::process_args;
use timestitch::application::TraceGuard;
use timestitch::application::write_table;
use timestitch::CalendarSystem;
use timestitch::clock::ClockTime;
use timestitch::Entry;
use timestitch::gregorian::GregorianProleptic;

// External library imports.
use anyhow::Context;
use anyhow::Error;
use clap::error::ErrorKind;
use clap::Parser;
use regex::Regex;
use tracing::event;
use tracing::Level;
use tracing::span;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() {
	// The worker_guard holds the worker thread handle for the nonblocking
	// trace writer. It should be held until all tracing is complete, as any
	// trace spans or events after it is dropped will be ignored.
	let mut trace_guard = TraceGuard::default();

	if let Err(err) = main_facade(&mut trace_guard) {
		// Trace errors without coloring.
		colored::control::set_override(false);
		event!(Level::ERROR, "{:?}", err);

		// Print errors to stderr and exit with error code.
		colored::control::unset_override();
		eprintln!("ERROR: {:?}", err);
		
		let exit_code = match err.downcast::<clap::Error>()
			.map(|e| e.kind())
		{
			Ok(ErrorKind::DisplayHelp)    |
			Ok(ErrorKind::DisplayVersion) => 0,
			_ => 1,
		};

		std::process::exit(exit_code);
	}
}


////////////////////////////////////////////////////////////////////////////////
// main_facade
////////////////////////////////////////////////////////////////////////////////
/// The application facade for propagating user errors.
pub fn main_facade(trace_guard: &mut TraceGuard) -> Result<(), Error> {
	// Parse opts line options.
	let mut opts = CliOpts::try_parse()?;
	if opts.no_expand_directories { opts.expand_directories = false }
	if !opts.expand_directories { opts.expand_recursive = false }

	// We lazily populate the current directory. We may fail to access it, and
	// it would be a spurious error to fail if we don't actually need to use it.
	let mut cur_dir = None;

	// Find the path for the config file.
	let config_path = match &opts.config {
		Some(path) => path.clone(),
		None => {
			if cur_dir.is_none() {
				cur_dir = Some(std::env::current_dir()?);
			}
			cur_dir.as_ref().unwrap().join(Config::DEFAULT_CONFIG_PATH)
		},
	};

	// Load the config file.
	let mut config_load_status = Ok(());
	let config = Config::read_from_path(&config_path)
		.with_context(|| format!("Unable to load config file: {:?}", 
			config_path))
		.unwrap_or_else(|e| {
			// Store the error for output until after the logger is configured.
			config_load_status = Err(e);
			Config::new().with_load_path(&config_path)
		});

	// Initialize the global tracing subscriber.
	let base_level = match (opts.verbose, opts.quiet, opts.trace) {
		(_, _, true) => Level::TRACE,
		(_, true, _) => Level::WARN,
		(true, _, _) => Level::INFO,
		_            => Level::WARN,
	};
	*trace_guard = config.trace_config.init_global_default(base_level)?;
	let _span = span!(Level::INFO, "main").entered();


	// Print version information.
	event!(Level::INFO, "TimeStitch version: {}", env!("CARGO_PKG_VERSION"));
	let rustc_meta = rustc_version_runtime::version_meta();
	event!(Level::DEBUG, "Rustc version: {} {:?}",
		rustc_meta.semver,
		rustc_meta.channel);
	if let Some(hash) = rustc_meta.commit_hash {
		event!(Level::DEBUG, "Rustc git commit: {}", hash);
	}
	event!(Level::DEBUG, "{:#?}", opts);
	event!(Level::DEBUG, "{:#?}", config);

	// Find the path for the prefs file.
	let prefs_path = match &opts.prefs {
		Some(path) => path.clone(),
		None => {
			if cur_dir.is_none() {
				cur_dir = Some(std::env::current_dir()?);
			}
			cur_dir.as_ref().unwrap().join(&config.prefs_path)
		},
	};

	// Load the prefs file.
	let prefs = match Prefs::read_from_path(&prefs_path) {
		Err(e) if opts.prefs.is_some() => {
			// Path is user-specified, so it is an error to not load it.
			return Err(e).with_context(|| format!(
				"Unable to load preferences file: {:?}", 
				prefs_path));
		},
		Err(_) => {
			// Path is default, so it is ok to use default prefs fallback.
			event!(Level::DEBUG, "Using default prefs.");
			Prefs::new().with_load_path(prefs_path)
		},
		Ok(prefs) => {
			// Prefs path successfully loaded.
			event!(Level::TRACE, "{:#?}", prefs); 
			prefs
		},
	};
	event!(Level::DEBUG, "{:#?}", prefs);

	
	println!("{}", prefs);
	let mut errors = Vec::new();
	match &prefs.calendar_system {
		CalendarSystem::GregorianProleptic { pattern, pattern_map, format } => {
			let entries: Vec<Entry<GregorianProleptic>> = process_args(
				&config,
				&prefs,
				&opts,
				&mut errors,
				Regex::new(pattern)?,
				pattern_map.clone(),
				format.clone(),
				opts.files.iter().map(|p| p.as_path()))?;

			//println!("{:?}", entries);
			//println!("{:?}", errors);
			let mut out = std::io::stdout();
			write_table(&config, &prefs, &mut out, entries.into_iter())
		},

		CalendarSystem::ClockTime { pattern, pattern_map, format } => {
			let entries: Vec<Entry<ClockTime>> = process_args(
				&config,
				&prefs,
				&opts,
				&mut errors,
				Regex::new(pattern)?,
				pattern_map.clone(),
				format.clone(),
				opts.files.iter().map(|p| p.as_path()))?;

			//println!("{:?}", entries);
			//println!("{:?}", errors);
			let mut out = std::io::stdout();
			write_table(&config, &prefs, &mut out, entries.into_iter())
		},
	}
}
