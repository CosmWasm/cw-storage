use named_type::NamedType;
use serde::{de::DeserializeOwned, ser::Serialize};
use snafu::ResultExt;

use cosmwasm::errors::{NotFound, ParseErr, Result, SerializeErr};
use cosmwasm::serde::{from_slice, to_vec};

/// serialize makes json bytes, but returns a cosmwasm::Error
pub fn serialize<T: Serialize + NamedType>(data: &T) -> Result<Vec<u8>> {
    to_vec(data).context(SerializeErr {
        kind: T::short_type_name(),
    })
}

/// may_deserialize parses json bytes from storage (Option), returning Ok(None) if no data present
///
/// value is an odd type, but this is meant to be easy to use with output from storage.get (Option<Vec<u8>>)
/// and value.map(|s| s.as_slice()) seems trickier than &value
pub fn may_deserialize<T: DeserializeOwned + NamedType>(
    value: &Option<Vec<u8>>,
) -> Result<Option<T>> {
    match value {
        Some(d) => from_slice(d).context(ParseErr {
            kind: T::short_type_name(),
        }),
        None => Ok(None),
    }
}

/// deserialize parses json bytes from storage (Option), returning NotFound error if no data present
pub fn deserialize<T: DeserializeOwned + NamedType>(value: &Option<Vec<u8>>) -> Result<T> {
    match value {
        Some(d) => from_slice(d).context(ParseErr {
            kind: T::short_type_name(),
        }),
        None => NotFound {
            kind: T::short_type_name(),
        }
        .fail(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use named_type_derive::NamedType;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, NamedType, PartialEq, Debug)]
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
        let parsed: Data = deserialize(&loaded).unwrap();
        assert_eq!(parsed, data);

        let may_parse: Option<Data> = may_deserialize(&loaded).unwrap();
        assert_eq!(may_parse, Some(data));
    }
}
