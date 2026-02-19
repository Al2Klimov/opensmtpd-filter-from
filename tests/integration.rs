use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn run_filter(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_opensmtpd-filter-from"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn config_ready_registers_events() {
    let output = run_filter(&[], b"config|ready\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("register|report|smtp-in|tx-begin\n"));
    assert!(stdout.contains("register|filter|smtp-in|data-line\n"));
    assert!(stdout.contains("register|filter|smtp-in|commit\n"));
    assert!(stdout.contains("register|report|smtp-in|link-disconnect\n"));
    assert!(stdout.contains("register|ready\n"));
}

#[test]
fn mail_without_blacklist_is_allowed() {
    let input = concat!(
        "config|ready\n",
        "report|0.7|1234567890.000000|smtp-in|tx-begin|session1\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|From: sender@example.com\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|.\n",
        "filter|0.7|1234567890.000000|smtp-in|commit|session1|token1\n",
    );
    let output = run_filter(&[], input.as_bytes());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("filter-result|session1|token1|proceed"));
}

#[test]
fn blacklisted_address_is_denied() {
    let path = std::env::temp_dir().join("test_integration_blacklisted_address.txt");
    fs::write(&path, "blocked@example.com\n").unwrap();
    let path_str = path.to_str().unwrap().to_string();

    let input = concat!(
        "config|ready\n",
        "report|0.7|1234567890.000000|smtp-in|tx-begin|session1\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|From: blocked@example.com\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|.\n",
        "filter|0.7|1234567890.000000|smtp-in|commit|session1|token1\n",
    );
    let output = run_filter(&["addr-file", &path_str], input.as_bytes());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("filter-result|session1|token1|reject|550 Sender or domain is blacklisted")
    );
    fs::remove_file(path).ok();
}

#[test]
fn blacklisted_domain_is_denied() {
    let path = std::env::temp_dir().join("test_integration_blacklisted_domain.txt");
    fs::write(&path, "example.com\n").unwrap();
    let path_str = path.to_str().unwrap().to_string();

    let input = concat!(
        "config|ready\n",
        "report|0.7|1234567890.000000|smtp-in|tx-begin|session1\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|From: sender@example.com\n",
        "filter|0.7|1234567890.000000|smtp-in|data-line|session1|token1|.\n",
        "filter|0.7|1234567890.000000|smtp-in|commit|session1|token1\n",
    );
    let output = run_filter(&["domain-file", &path_str], input.as_bytes());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("filter-result|session1|token1|reject|550 Sender or domain is blacklisted")
    );
    fs::remove_file(path).ok();
}
