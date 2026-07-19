////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////


use timestitch::application::TraceGuard;

// External library imports.
use anyhow::Context;
use anyhow::Error;
use anyhow::anyhow;
use clap::Parser;
use clap::error::ErrorKind;
use clap::CommandFactory as _;
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
        eprintln!("{:?}", err);

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
    Ok(())
}
