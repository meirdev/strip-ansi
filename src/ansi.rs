use std::ops::Bound::*;
use std::ops::{RangeBounds, RangeInclusive};

const CSI: &[u8] = &[0x1b, 0x5b]; // ESC[

const PARAMETER_BYTE: RangeInclusive<u8> = 0x30..=0x3F; // 0–9:;<=>?

const INTERMEDIATE_BYTE: RangeInclusive<u8> = 0x20..=0x2F; // !"#$%&'()*+,-./

const FINAL_BYTE: RangeInclusive<u8> = 0x40..=0x7E; // @A–Z[\]^_`a–z{|}~

#[inline]
fn is_parameter_byte(b: &u8) -> bool {
    PARAMETER_BYTE.contains(b)
}

#[inline]
fn is_intermediate_byte(b: &u8) -> bool {
    INTERMEDIATE_BYTE.contains(b)
}

#[inline]
fn is_final_byte(b: &u8) -> bool {
    FINAL_BYTE.contains(b)
}

fn find_csi_sequence(s: &[u8]) -> Result<Option<RangeInclusive<usize>>, ()> {
    // println!("bytes: {:?}", s);

    let result = s.windows(CSI.len())
        .enumerate()
        .find(|(_, chunk)| *chunk == CSI)
        .map(|(i, _)| {
            let end_csi = i + CSI.len();
            let buffer = &s[end_csi..];

            if let Some((j, _)) = buffer
                .iter()
                .enumerate()
                .skip_while(|(_, x)| is_parameter_byte(x))
                .skip_while(|(_, x)| is_intermediate_byte(x))
                .next()
                .filter(|(_, x)| is_final_byte(x))
            {
                return Ok(Some(i..=end_csi + j));
            } else {
                return Err(());
            }
        });

    return result.unwrap_or(Ok(None));
}

pub fn strip_ansi(s: &str) -> String {
    let mut b: Vec<u8> = Vec::with_capacity(s.chars().count());
    let mut s = s.as_bytes();

    while let Ok(csi) = find_csi_sequence(s) {
        if let Some(csi) = csi {
            // println!("ansi range: {:?}", csi);

            if let Included(&start) = csi.start_bound() {
                b.extend_from_slice(&s[..start]);
            }

            if let Included(&end) = csi.end_bound() {
                s = &s[end + 1..];
            }
        } else {
            break;
        }
    }

    b.extend_from_slice(s);

    return String::from_utf8(b).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_with_ansi() {
        let s = "Hello, \x1b[1mworld\x1b[0m!";
        let expected = "Hello, world!";
        assert_eq!(strip_ansi(s), expected);
    }

    #[test]
    fn test_text_without_ansi() {
        let s = "Hello, world!";
        let expected = "Hello, world!";
        assert_eq!(strip_ansi(s), expected);
    }

    #[test]
    fn test_invalid_ansi() {
        let s = "Hello, \x1b[\nmworld!";
        let expected = "Hello, \x1b[\nmworld!";
        assert_eq!(strip_ansi(s), expected);
    }

    #[test]
    fn test_missing_final_byte() {
        let s = "Hello, \x1b[123";
        let expected = "Hello, \x1b[123";
        assert_eq!(strip_ansi(s), expected);
    }

    #[test]
    fn test_minimum_csi() {
        let s = "Hello\x1b[m";
        let expected = "Hello";
        assert_eq!(strip_ansi(s), expected);
    }
}
