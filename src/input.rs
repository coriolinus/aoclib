use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::{fs::File, io::Cursor};

const TEST_DATA_FILENAME: &str = "TEST DATA";

/// Parse the file at the specified path into a stream of `T`.
///
/// Each line is treated as a separate record. Leading and trailing spaces
/// are trimmed before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
///
/// See also [`parse_str`] for equivalent functionality for strings, useful for test data.
pub fn parse<'a, T>(path: &'a Path) -> std::io::Result<impl 'a + Iterator<Item = T>>
where
    T: 'a + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_reader(
        reader,
        path.file_name()
            .expect("File::open() didn't early return before now; qed")
            .to_string_lossy(),
    )
}

/// Parse the provided data into a stream of `T`.
///
/// Each line is treated as a separate record. Leading and trailing spaces
/// are trimmed before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
///
/// See also [`parse`] for equivalent functionality for input files.
pub fn parse_str<'a, T>(data: &'a str) -> std::io::Result<impl '_ + Iterator<Item = T>>
where
    T: 'a + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    parse_reader(Cursor::new(data), TEST_DATA_FILENAME)
}

/// Parse the contents of the provided reader into a stream of `T`.
///
/// Often [`parse`] or [`parse_str`] are more ergonomic.
///
/// Each line is treated as a separate record. Leading and trailing spaces
/// are trimmed before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
///
/// The file name can technically be anything which is `'a + Display`, but it's used within
/// error messages as the file name, so it should be reasonably interprable as such.
pub fn parse_reader<'a, T, Reader, Filename>(
    mut reader: Reader,
    file_name: Filename,
) -> std::io::Result<impl 'a + Iterator<Item = T>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    Reader: 'a + BufRead,
    Filename: 'a + std::fmt::Display,
{
    let mut buf = String::new();
    let mut line: usize = 0;
    Ok(std::iter::from_fn(move || {
        buf.clear();
        reader.read_line(&mut buf).ok().and_then(|_| {
            line += 1;
            (!buf.is_empty())
                .then(|| match T::from_str(&buf.trim()) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        eprintln!("{}:{}: {} for {:?}", file_name, line, e, buf.trim());
                        None
                    }
                })
                .flatten()
        })
    })
    .fuse())
}

/// Parse the file at the specified path into a stream of `T`.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been
/// collected, it (and internal newlines) are parsed into a `T` instance.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being
/// handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
///
/// See also [`parse_newline_sep_str`] for equivalent functionality for strings, useful for test data.
pub fn parse_newline_sep<'a, T>(path: &'a Path) -> std::io::Result<impl 'a + Iterator<Item = T>>
where
    T: 'a + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_newline_sep_reader(
        reader,
        path.file_name()
            .expect("File::open() didn't early return before now; qed")
            .to_string_lossy(),
    )
}

/// Parse the provided data into a stream of `T`.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been
/// collected, it (and internal newlines) are parsed into a `T` instance.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being
/// handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
///
/// See also [`parse_newline_sep`] for equivalent functionality for input files.
pub fn parse_newline_sep_str<'a, T>(data: &'a str) -> std::io::Result<impl 'a + Iterator<Item = T>>
where
    T: 'a + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    parse_newline_sep_reader(Cursor::new(data), TEST_DATA_FILENAME)
}

/// Parse the contents of the provided reader into a stream of `T`.
///
/// Often [`parse_newline_sep`] or [`parse_newline_sep_str`] are more ergonomic.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been
/// collected, it (and internal newlines) are parsed into a `T` instance.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being
/// handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
pub fn parse_newline_sep_reader<'a, T, Reader, Filename>(
    mut reader: Reader,
    file_name: Filename,
) -> std::io::Result<impl 'a + Iterator<Item = T>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    Reader: 'a + BufRead,
    Filename: 'a + std::fmt::Display,
{
    let mut buf = String::new();
    let mut line: usize = 0;

    fn is_new_field(buf: &str) -> bool {
        let patterns = ["\n\n", "\n\r\n"];
        patterns.iter().any(|pat| {
            buf.as_bytes()
                .iter()
                .rev()
                .zip(pat.as_bytes().iter())
                .all(|(b, p)| b == p)
        })
    }

    Ok(std::iter::from_fn(move || {
        buf.clear();
        while buf.is_empty() || !is_new_field(&buf) {
            line += 1;
            if reader.read_line(&mut buf).ok()? == 0 {
                break;
            }
        }
        (!buf.is_empty())
            .then(|| match T::from_str(&buf) {
                Ok(t) => Some(t),
                Err(e) => {
                    eprintln!("{}:{}: {} for {:?}", file_name, line - 1, e, buf);
                    None
                }
            })
            .flatten()
    })
    .fuse())
}

/// adaptor which plugs into parse, splitting comma-separated items from the line
///
/// This can be flattened or consumed by line, as required
pub struct CommaSep<T>(Vec<T>);

impl<T> FromStr for CommaSep<T>
where
    T: FromStr,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map(CommaSep)
    }
}

impl<T> IntoIterator for CommaSep<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
