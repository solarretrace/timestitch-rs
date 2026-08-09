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
use crate::Entry;
use crate::DataSource;
use crate::MatchSource;
use crate::MatchSourceAttribute;
use crate::TimeInterval;

// External library imports.
use anyhow::anyhow;
use anyhow::Context as _;
use anyhow::Error;
use either::Either;
use regex::Regex;

// Standard library imports.
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// process_files
////////////////////////////////////////////////////////////////////////////////
/// Reads `Entry`s from an iterator of files according to the `Entry` data
/// schema provided by `Prefs`.
///
/// This is the main data extraction function of the application.
pub fn process_files<'a, I>(
    _config: Config,
    prefs: Prefs,
    paths: I)
    -> Result<Vec<Entry>, Error>
    where I: IntoIterator<Item=&'a Path>
{
    // Compile matchers.
    let re_all: Regex = Regex::new(".*").unwrap();
    let path_matcher = prefs
        .path_matcher
        .as_ref()
        .map(|m| Regex::new(m))
        .transpose()?;
    let content_split = prefs
        .content_split
        .as_ref()
        .map(|m| Regex::new(m))
        .transpose()?;
    let content_line_matchers = prefs
        .content_line_matchers
        .iter()
        .map(|matcher| matcher
            .as_deref()
            .map(Regex::new)
            .unwrap_or(Ok(re_all.clone())))
        .collect::<Result<Vec<_>, _>>()?;

    let mut entries = Vec::new();
    for path in paths {
        let path_str = path.to_string_lossy();
        let path_captures = path_matcher
            .as_ref()
            .unwrap_or(&re_all)
            .captures(&path_str)
            .expect("construct path capture groups");

        let contents = get_file_contents(path)?;
        for content in split_contents(&contents, content_split.as_ref()) {
            let line_captures: Vec<Option<_>> = content.lines()
                .zip(content_line_matchers.iter())
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
                MatchSource::Default => TimeInterval::unknown(),
                MatchSource::Path { group } => {
                    path_captures
                        .get(group)
                        .ok_or(anyhow!("invalid path capture group"))?
                        .as_str()
                        .parse::<TimeInterval>()?
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
                        .parse::<TimeInterval>()?
                },
            };

            // Extract the Entry source file.
            let source_path = DataSource(path.into());

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

    }

    Ok(entries)
}


/// Reads the file contents from the given `Path` into a string.
fn get_file_contents<P>(path: P) -> Result<Box<str>, Error> 
    where P: AsRef<Path>
{
    let path = path.as_ref();
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file for reading: {}",
            path.display()))?;
    let len = file.metadata()
        .context("Failed to read file metadata.")?
        .len();
    let mut buf = Vec::with_capacity(len.try_into()?);
    let _ = file.read_to_end(&mut buf)
        .context("Failed to read entry file")?;
    String::from_utf8(buf)
        .context("Failed to read file as valid UTF-8")
        .map(|s| s.into_boxed_str())
}

/// Splits the given string using the provided `Regex`, or returns it unmodified
/// if no regex is provided.
fn split_contents<'a>(contents: &'a str, content_split: Option<&Regex>)
    -> impl Iterator<Item=&'a str>
{
    match content_split {
        Some(re) => Either::Left(re.split(contents)),
        None     => Either::Right(std::iter::once(contents)),
    }
}
