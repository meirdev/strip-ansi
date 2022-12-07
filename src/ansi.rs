use std::ops::Bound::*;
use std::ops::{RangeBounds, RangeInclusive};

const CSI: &[u8] = &[0x1b, 0x5b]; // ESC[

const PARAMETER_BYTE: RangeInclusive<u8> = 0x30..=0x3F; // 0–9:;<=>?

const INTERMEDIATE_BYTE: RangeInclusive<u8> = 0x20..=0x2F; // !"#$%&'()*+,-./

const FINAL_BYTE: RangeInclusive<u8> = 0x40..=0x7E; // @A–Z[\]^_`a–z{|}~

const MIN_CSI_LENGTH: usize = 3;

fn is_parameter_byte(b: &u8) -> bool {
    PARAMETER_BYTE.contains(b)
}

fn is_intermediate_byte(b: &u8) -> bool {
    INTERMEDIATE_BYTE.contains(b)
}

fn is_final_byte(b: &u8) -> bool {
    FINAL_BYTE.contains(b)
}

macro_rules! check {
    ($function:ident, $s:ident, $i:ident) => {
        $i < $s.len() && $function(&$s[$i])
    };
}

fn find_csi_sequence(s: &[u8]) -> Result<Option<RangeInclusive<usize>>, ()> {
    // println!("bytes: {:?}", s);

    let mut i = 0;
    while s.len() >= MIN_CSI_LENGTH && i <= s.len() - MIN_CSI_LENGTH {
        if &s[i..i + 2] == CSI {
            let j = i;
            i += 2;

            while check!(is_parameter_byte, s, i) {
                i += 1;
            }

            while check!(is_intermediate_byte, s, i) {
                i += 1;
            }

            if check!(is_final_byte, s, i) {
                return Ok(Some(j..=i));
            }

            return Err(());
        }

        i += 1;
    }

    return Ok(None);
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
}
