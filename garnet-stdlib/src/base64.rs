//! `std::base64` — standard RFC 4648 base64 (Layer 1, no caps).
//!
//! Hand-rolled (no external crate) so the binary takes no new dependency for
//! a fully-specified algorithm. Standard alphabet, with `=` padding.
//! `@stability(experimental)`.

use crate::StdError;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode bytes to a standard base64 string (with padding).
pub fn encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn decode_byte(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'a'..=b'z' => Some((c - b'a' + 26) as u32),
        b'0'..=b'9' => Some((c - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decode a standard base64 string to bytes. Errors on invalid length,
/// invalid characters, or misplaced padding.
pub fn decode(input: &str) -> Result<Vec<u8>, StdError> {
    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(StdError::InvalidInput(format!(
            "base64: length {} is not a multiple of 4",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let pad = chunk.iter().filter(|&&c| c == b'=').count();
        if pad > 2 {
            return Err(StdError::InvalidInput(
                "base64: more than two padding characters in a group".into(),
            ));
        }
        let mut n: u32 = 0;
        for (i, &c) in chunk.iter().enumerate() {
            let v = if c == b'=' {
                // Padding only allowed in the last two positions of the group.
                if i < 2 {
                    return Err(StdError::InvalidInput(
                        "base64: padding in an invalid position".into(),
                    ));
                }
                0
            } else {
                decode_byte(c).ok_or_else(|| {
                    StdError::InvalidInput(format!("base64: invalid character {:?}", c as char))
                })?
            };
            n |= v << (18 - 6 * i);
        }
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4648 §10 test vectors.
    #[test]
    fn rfc4648_vectors() {
        assert_eq!(encode(b""), "");
        assert_eq!(encode(b"f"), "Zg==");
        assert_eq!(encode(b"fo"), "Zm8=");
        assert_eq!(encode(b"foo"), "Zm9v");
        assert_eq!(encode(b"foob"), "Zm9vYg==");
        assert_eq!(encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn decode_rfc4648_vectors() {
        assert_eq!(decode("").unwrap(), b"");
        assert_eq!(decode("Zg==").unwrap(), b"f");
        assert_eq!(decode("Zm8=").unwrap(), b"fo");
        assert_eq!(decode("Zm9v").unwrap(), b"foo");
        assert_eq!(decode("Zm9vYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn roundtrip_all_byte_values() {
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(decode(&encode(&all)).unwrap(), all);
        // a few binary blobs of awkward lengths
        for len in [1usize, 2, 3, 7, 16, 31, 100] {
            let blob: Vec<u8> = (0..len).map(|i| (i * 37 % 256) as u8).collect();
            assert_eq!(decode(&encode(&blob)).unwrap(), blob);
        }
    }

    #[test]
    fn decode_rejects_bad_length() {
        match decode("abc") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_invalid_char() {
        match decode("Zg=*") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_misplaced_padding() {
        match decode("Z===") {
            Err(StdError::InvalidInput(_)) => {}
            other => panic!("expected InvalidInput for padding pos, got {other:?}"),
        }
    }
}
