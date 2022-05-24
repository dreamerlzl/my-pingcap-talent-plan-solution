use std::error::Error;

mod error;
pub mod extract;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn set_to_bytes(k: String, v: String) -> Result<Vec<u8>> {
    Ok(strings_to_commands(vec![
        Some("set".to_owned().into_bytes()),
        Some(k.into_bytes()),
        Some(v.into_bytes()),
    ]))
}

pub fn ping_to_bytes(maybe_s: Option<String>) -> Result<Vec<u8>> {
    if let Some(s) = maybe_s {
        Ok(strings_to_commands(vec![
            Some("ping".to_owned().into_bytes()),
            Some(s.into_bytes()),
        ]))
    } else {
        Ok(strings_to_commands(vec![Some(
            "ping".to_owned().into_bytes(),
        )]))
    }
}

pub fn get_to_bytes(k: String) -> Result<Vec<u8>> {
    Ok(strings_to_commands(vec![
        Some("get".to_owned().into_bytes()),
        Some(k.into_bytes()),
    ]))
}

fn strings_to_commands(vec_strs: Vec<Option<Vec<u8>>>) -> Vec<u8> {
    let mut result = vec![b'*'];
    result.extend(vec_strs.len().to_string().into_bytes());
    result.push(b'\r');
    result.push(b'\n');
    vec_strs
        .into_iter()
        .map(|s| bulk_string(s.as_ref()))
        .for_each(|s| result.extend(s));
    result
}

fn bulk_string(binary_safe_str: Option<&Vec<u8>>) -> Vec<u8> {
    if let Some(not_null) = binary_safe_str {
        let mut result: Vec<u8> = Vec::new();
        let len_digits = not_null.len().to_string().into_bytes();
        result.push(b'$');
        result.extend(len_digits);
        result.extend_from_slice(&[b'\r', b'\n']);
        result.extend(not_null);
        result.extend_from_slice(&[b'\r', b'\n']);
        result
    } else {
        b"$-1\r\n".to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::get_to_bytes;
    use crate::ping_to_bytes;
    use crate::set_to_bytes;
    use crate::Result;

    fn string_to_bytes(s: &str) -> Vec<u8> {
        s.to_owned().into_bytes()
    }

    #[test]
    fn test_get() -> Result<()> {
        let k = "abc".to_owned();
        let bytes = get_to_bytes(k)?;
        assert_eq!(bytes, string_to_bytes("*2\r\n$3\r\nget\r\n$3\r\nabc\r\n"));
        Ok(())
    }

    #[test]
    fn test_set() -> Result<()> {
        let (k, v) = ("a".to_owned(), "b".to_owned());
        let bytes = set_to_bytes(k, v)?;
        assert_eq!(
            bytes,
            string_to_bytes("*3\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\nb\r\n")
        );
        Ok(())
    }

    #[test]
    fn test_ping() -> Result<()> {
        let p = Some("ab".to_owned());
        let bytes = ping_to_bytes(p)?;
        assert_eq!(bytes, string_to_bytes("*2\r\n$4\r\nping\r\n$2\r\nab\r\n"));
        let p = None;
        let bytes = ping_to_bytes(p)?;
        assert_eq!(bytes, string_to_bytes("*1\r\n$4\r\nping\r\n"));
        Ok(())
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
