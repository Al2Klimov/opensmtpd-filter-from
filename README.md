## About

OpenSMTPd filter which rejects eMails based on configurable sender blacklists.

## Build

Compile like any other Rust program: `cargo build -r`

Find the resulting binary directly under `target/release/`.

## Usage

Integrate this filter into smtpd.conf(5).

### Command-line interface

```
opensmtpd-filter-from [addr-file|domain-file FILE ...]
```

The binary takes any number of sender lists as arguments.
Each one is a pair of the kind of blacklist, either individual addresses
or whole domains, and the path to the file on the local filesystem.

### Blacklist file format

Empty lines are ignored. The others must be UTF-8.

Every non-empty line is either an eMail address (root@example.com)
or a domain (example.com) to disallow.

Prefix a domain with a full stop (.example.com) for all subdomains.
The domain itself must be specified separately (example.com).
Useful e.g. for whole TLDs.
