use serde::{de::DeserializeOwned, ser::Serialize};
use snafu::ResultExt;

use cosmwasm::errors::{NotFound, ParseErr, Result, SerializeErr};
use cosmwasm::serde::{from_slice, to_vec};

/// serialize makes json bytes, but returns a cosmwasm::Error
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    to_vec(data).context(SerializeErr {
        kind: T::type_name(),
    })
}

/// may_deserialize parses json bytes from storage (Option), returning Ok(None) if no data present
///
/// value is an odd type, but this is meant to be easy to use with output from storage.get (Option<Vec<u8>>)
/// and value.map(|s| s.as_slice()) seems trickier than &value
pub(crate) fn may_deserialize<T: DeserializeOwned>(value: &Option<Vec<u8>>) -> Result<Option<T>> {
    match value {
        Some(d) => Ok(Some(deserialize(d.as_slice())?)),
        None => Ok(None),
    }
}

/// must_deserialize parses json bytes from storage (Option), returning NotFound error if no data present
pub(crate) fn must_deserialize<T: DeserializeOwned>(value: &Option<Vec<u8>>) -> Result<T> {
    match value {
        Some(d) => deserialize(&d),
        None => NotFound {
            kind: T::type_name(),
        }
        .fail(),
    }
}

// deserialize is a reflection of serialize and probably what most people outside the crate expect
pub fn deserialize<T: DeserializeOwned>(value: &[u8]) -> Result<T> {
    from_slice(value).context(ParseErr {
        kind: T::type_name(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm::errors::Error;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn serialize_and_deserialize() {
        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        let value = serialize(&data).unwrap();
        let loaded = Some(value);

        //        let parsed: Data = deserialize(loaded.map(|s| s.as_slice())).unwrap();
        //        assert_eq!(parsed, data);
        let parsed: Data = must_deserialize(&loaded).unwrap();
        assert_eq!(parsed, data);

        let may_parse: Option<Data> = may_deserialize(&loaded).unwrap();
        assert_eq!(may_parse, Some(data));
    }

    #[test]
    fn handle_none() {
        let may_parse = may_deserialize::<Data>(&None).unwrap();
        assert_eq!(may_parse, None);

        let parsed = must_deserialize::<Data>(&None);
        match parsed {
            Err(Error::NotFound { kind }) => assert_eq!(kind, "Data"),
            Err(e) => panic!("Unexpected error {}", e),
            Ok(_) => panic!("should error"),
        }
    }
}
