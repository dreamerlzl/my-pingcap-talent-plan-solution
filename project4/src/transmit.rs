// use ron::{from_str, to_string};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str, to_string};

use crate::{KvsError, Result};

pub fn to_bytes<T: Serialize>(comm: T) -> Result<Vec<u8>> {
    to_string(&comm)
        .map(|s| s.into_bytes())
        .map_err(KvsError::KSPSerdeError)
}

pub fn from_bytes<T: DeserializeOwned>(bytes: Vec<u8>) -> Result<T> {
    from_str(std::str::from_utf8(&bytes).expect("all strings in this kvstore shall be safe"))
        .map_err(KvsError::KSPSerdeError)
}

#[cfg(test)]
mod tests {
    use super::{from_str, to_string};
    use crate::KSP;

    #[test]
    fn test_serde() {
        let cases = [
            KSP::Set("a".to_owned(), "b".to_owned()),
            KSP::Get("a".to_owned()),
            KSP::Rm("a".to_owned()),
        ];
        for case in cases {
            let bytes: String = to_string(&case).expect("fuck");
            assert_eq!(from_str::<KSP>(&bytes).unwrap(), case);
        }
    }
}
