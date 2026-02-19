use std::io::{Result as IoResult, Write};

pub(crate) fn join_write_bytes<'a>(
    writer: &mut dyn Write,
    sep: &[u8],
    mut parts: impl Iterator<Item = &'a [u8]>,
) -> IoResult<()> {
    match parts.next() {
        None => {}
        Some(first) => {
            writer.write_all(first)?;

            for part in parts {
                writer.write_all(sep)?;
                writer.write_all(part)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_iterator_writes_nothing() {
        let mut buf = Vec::new();
        join_write_bytes(&mut buf, b"|", std::iter::empty()).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn single_item_no_separator() {
        let mut buf = Vec::new();
        join_write_bytes(&mut buf, b"|", [b"hello" as &[u8]].iter().copied()).unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn multiple_items_joined_with_separator() {
        let mut buf = Vec::new();
        join_write_bytes(&mut buf, b"|", [b"a" as &[u8], b"b", b"c"].iter().copied()).unwrap();
        assert_eq!(buf, b"a|b|c");
    }
}
