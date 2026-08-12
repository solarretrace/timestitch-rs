////////////////////////////////////////////////////////////////////////////////
// Timestitch journal and timeline creator
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Table generation module.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::application::Config;
use crate::application::Prefs;
use crate::Entry;

// External library imports.
use anyhow::Error;
use anyhow::Context as _;
use table_gen_markdown::MarkdownGridRenderer;
use table_gen::Table;

// Standard library imports.
use std::io::Write;


////////////////////////////////////////////////////////////////////////////////
// write_table
////////////////////////////////////////////////////////////////////////////////
/// Writes an iterator of entries to the given output as a table.
pub fn write_table<'a, W, I>(
    _config: &Config,
    _prefs: &Prefs,
    out: &mut W,
    entries: I)
    -> Result<(), Error>
    where
        I: IntoIterator<Item=Entry>,
        W: Write,
{
    let mut table = Table::new_builder(entries, MarkdownGridRenderer::new())
        .finish();
    table.render(out)
        .context("failed to write table")
}
