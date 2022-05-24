use crate::error::MBytesError;

type Result<T> = std::result::Result<T, MBytesError>;

// the input is a valid prefix of simple string
pub fn extract_simple_string(resp: &[u8]) -> Result<(String, usize)> {
    let l = resp.len();
    if l < 3 || resp[0] != b'+' {
        return Err(MBytesError::InvalidSimple(resp.to_vec()));
    }
    if let Some(cr_lf_pos) = resp.iter().position(is_separator) {
        let result = String::from_utf8(resp[1..cr_lf_pos].to_vec())?;
        // 3 -> + \r\n
        Ok((result, cr_lf_pos - 1 + 3))
    } else {
        Err(MBytesError::InvalidSimple(resp.to_vec()))
    }
}

// the input is a valid prefix of bulk string
pub fn extract_bulk_string(resp: &[u8]) -> Result<(Option<Vec<u8>>, usize)> {
    if resp[0] != b'$' {
        return Err(MBytesError::InvalidBulk(resp.to_vec()));
    }
    if let Some(first_cr_lf_pos) = resp.iter().position(is_separator) {
        if resp[1] == b'-' {
            // $-1\r\n
            Ok((None, 5))
        } else {
            let num_bytes = resp[1..first_cr_lf_pos]
                .iter()
                .fold(0, |acc, x| acc * 10 + u8_to_digit(*x)) as usize;
            //dbg!(first_cr_lf_pos, num_bytes);
            let first_pos = first_cr_lf_pos + 2;
            Ok((
                Some(resp[first_pos..first_pos + num_bytes].to_vec()),
                // $ \r\n \r\n
                num_bytes + 5 + first_cr_lf_pos - 1,
            ))
        }
    } else {
        Err(MBytesError::InvalidBulk(resp.to_vec()))
    }
}

pub fn resp_to_vec(resp: &[u8]) -> Result<Vec<Option<Vec<u8>>>> {
    if resp[0] != b'*' {
        return Err(MBytesError::InvalidResp(resp.to_vec()));
    }
    if let Some(first_cr_lf_pos) = resp.iter().position(is_separator) {
        let num_ele = resp[1..first_cr_lf_pos]
            .iter()
            .fold(0, |acc, x| acc * 10 + u8_to_digit(*x)) as usize;
        let mut first_pos = first_cr_lf_pos + 2;
        let mut results: Vec<Option<Vec<u8>>> = Vec::new();
        for _ in 0..num_ele {
            let (result, num_bytes) = match resp[first_pos] {
                b'+' => {
                    let (result, num_bytes) = extract_simple_string(&resp[first_pos..])?;
                    (Some(result.into_bytes()), num_bytes)
                }
                b'$' => extract_bulk_string(&resp[first_pos..])?,
                _ => {
                    unimplemented!()
                }
            };
            dbg!(first_pos, num_bytes);
            first_pos += num_bytes;
            results.push(result);
        }
        Ok(results)
    } else {
        Err(MBytesError::InvalidResp(resp.to_vec()))
    }
}

fn is_separator(c: &u8) -> bool {
    *c == b'\r' || *c == b'\n'
}

fn u8_to_digit(c: u8) -> u32 {
    (c - 48) as u32
}

#[cfg(test)]
mod tests {
    use super::{extract_bulk_string, extract_simple_string, resp_to_vec};

    #[test]
    fn test_simple_string() {
        let cases = vec![
            ("+OK\r\n", Some("OK"), 5),
            ("a", None, 0),
            ("ab", None, 0),
            ("+Ok\r\n1", Some("Ok"), 5),
        ];
        for (input, output, num_bytes) in cases.into_iter() {
            let result = extract_simple_string(input.as_bytes());
            if let Some(s) = output {
                assert!(result.is_ok());
                assert_eq!((s.to_owned(), num_bytes), result.unwrap());
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_bulk_string() {
        let cases = vec![
            ("$6\r\nhelloo\r\n", Some(b"helloo".to_vec()), 12),
            ("$6\r\nhelloo\r\n+3abc\r\n", Some(b"helloo".to_vec()), 12),
            ("$8\r\nhel\r\nloo\r\n", Some(b"hel\r\nloo".to_vec()), 14),
            ("$0\r\n\r\n", Some(b"".to_vec()), 6),
            ("$-1\r\n", None, 5),
        ];
        for (input, output, num_bytes) in cases {
            let result = extract_bulk_string(input.as_bytes());
            if output.is_some() {
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), (output, num_bytes), "{:?}", input);
            } else if result.is_ok() {
                assert_eq!(result.unwrap().0, None);
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_resp_parse() {
        let cases = vec![
            (
                "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
                Some(vec![Some(b"hello".to_vec()), Some(b"world".to_vec())]),
            ),
            ("*0\r\n", Some(vec![])),
            (
                "*3\r\n+1\r\n+2\r\n+3ab\r\n",
                Some(vec![
                    Some(b"1".to_vec()),
                    Some(b"2".to_vec()),
                    Some(b"3ab".to_vec()),
                ]),
            ),
            ("+1\r\n+1a\r\n", None),
        ];
        for (input, output) in cases {
            let result = resp_to_vec(input.as_bytes());
            if let Some(o) = output {
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), o);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
