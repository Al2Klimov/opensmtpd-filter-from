use crate::cnt_iter::CounterIterator;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};

pub(crate) type Args = (HashSet<String>, Vec<String>);

pub(crate) enum ParseArgsError {
    UnknownArg,
    NoFile,
    EmptyName,
    BadFile(IoError),
    BadLine(usize, IoError),
}

pub(crate) fn blame_user(err: ParseArgsError, consumed: usize) {
    match err {
        ParseArgsError::UnknownArg => {
            eprintln!(
                "Unknown argument (CLI argument #{}), expected \"addr-file\"/\"domain-file\".",
                consumed
            );
        }
        ParseArgsError::NoFile => {
            eprintln!("Unexpected end of CLI arguments, expected file.");
        }
        ParseArgsError::EmptyName => {
            eprintln!(
                "Illegal empty string (CLI argument #{}), expected file.",
                consumed
            );
        }
        ParseArgsError::BadFile(er) => {
            eprintln!(
                "Inaccessible file (CLI argument #{}), error: {}",
                consumed, er
            );
        }
        ParseArgsError::BadLine(no, er) => {
            eprintln!(
                "File read error (CLI argument #{}, line #{}): {}",
                consumed, no, er
            );
        }
    }
}

pub(crate) fn parse_cmdline(
    mut args: impl Iterator<Item = OsString>,
) -> (Option<OsString>, Result<Args, ParseArgsError>, usize) {
    let program = args.next();
    let mut ci = CounterIterator::new(args);

    (program, parse_args(&mut ci), ci.taken())
}

fn parse_args(args: &mut dyn Iterator<Item = OsString>) -> Result<Args, ParseArgsError> {
    let mut addrs = HashSet::new();
    let mut domains = Vec::new();

    loop {
        match args.next() {
            None => return Ok((addrs, domains)),
            Some(arg) => match arg.to_string_lossy().as_ref() {
                "addr-file" => require_lines(args.next(), |line| {
                    addrs.insert(line);
                })?,
                "domain-file" => require_lines(args.next(), |mut line| {
                    if !line.starts_with(".") {
                        line.insert(0, '@');
                    }

                    domains.push(line);
                })?,
                _ => return Err(ParseArgsError::UnknownArg),
            },
        }
    }
}

fn require_lines(
    oarg: Option<OsString>,
    mut on_line: impl FnMut(String),
) -> Result<(), ParseArgsError> {
    let name = oarg.ok_or(ParseArgsError::NoFile)?;
    if name.is_empty() {
        return Err(ParseArgsError::EmptyName);
    }

    let mut ci = CounterIterator::new(
        BufReader::new(File::open(name).map_err(|err| ParseArgsError::BadFile(err))?).lines(),
    );
    loop {
        match ci.next() {
            None => return Ok(()),
            Some(Err(err)) => return Err(ParseArgsError::BadLine(ci.taken(), err)),
            Some(Ok(line)) => {
                if !line.is_empty() {
                    on_line(line);
                }
            }
        }
    }
}
