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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;

    fn args_iter<'a>(v: &'a [&'a str]) -> impl Iterator<Item = OsString> + use<'a> {
        v.iter().map(|s| OsString::from(s))
    }

    #[test]
    fn no_args_returns_empty_sets() {
        let (_, result, consumed) = parse_cmdline(args_iter(&["program"]));
        assert_eq!(consumed, 0);
        let (addrs, domains) = result.ok().unwrap();
        assert!(addrs.is_empty());
        assert!(domains.is_empty());
    }

    #[test]
    fn unknown_arg_returns_error() {
        let (_, result, _) = parse_cmdline(args_iter(&["program", "unknown"]));
        assert!(matches!(result, Err(ParseArgsError::UnknownArg)));
    }

    #[test]
    fn addr_file_without_path_returns_error() {
        let (_, result, _) = parse_cmdline(args_iter(&["program", "addr-file"]));
        assert!(matches!(result, Err(ParseArgsError::NoFile)));
    }

    #[test]
    fn domain_file_without_path_returns_error() {
        let (_, result, _) = parse_cmdline(args_iter(&["program", "domain-file"]));
        assert!(matches!(result, Err(ParseArgsError::NoFile)));
    }

    #[test]
    fn addr_file_empty_name_returns_error() {
        let (_, result, _) = parse_cmdline(args_iter(&["program", "addr-file", ""]));
        assert!(matches!(result, Err(ParseArgsError::EmptyName)));
    }

    #[test]
    fn addr_file_nonexistent_returns_error() {
        let path = std::env::temp_dir()
            .join("nonexistent_subdir_xyz789")
            .join("nonexistent_file.txt");
        let path_str = path.to_str().unwrap().to_string();
        let (_, result, _) = parse_cmdline(args_iter(&["program", "addr-file", &path_str]));
        assert!(matches!(result, Err(ParseArgsError::BadFile(_))));
    }

    #[test]
    fn addr_file_loads_addresses() {
        let path = std::env::temp_dir().join("test_addr_file_loads_addresses.txt");
        fs::write(&path, "user@example.com\nother@test.com\n").unwrap();
        let path_str = path.to_str().unwrap().to_string();
        let (_, result, _) = parse_cmdline(args_iter(&["program", "addr-file", &path_str]));
        let (addrs, domains) = result.ok().unwrap();
        assert!(addrs.contains("user@example.com"));
        assert!(addrs.contains("other@test.com"));
        assert!(domains.is_empty());
        fs::remove_file(path).ok();
    }

    #[test]
    fn domain_file_loads_domains() {
        let path = std::env::temp_dir().join("test_domain_file_loads_domains.txt");
        fs::write(&path, "example.com\n.sub.test.com\n").unwrap();
        let path_str = path.to_str().unwrap().to_string();
        let (_, result, _) = parse_cmdline(args_iter(&["program", "domain-file", &path_str]));
        let (addrs, domains) = result.ok().unwrap();
        assert!(addrs.is_empty());
        assert!(domains.contains(&"@example.com".to_string()));
        assert!(domains.contains(&".sub.test.com".to_string()));
        fs::remove_file(path).ok();
    }
}
