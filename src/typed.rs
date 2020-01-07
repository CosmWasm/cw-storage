use named_type::NamedType;
use serde::{de::DeserializeOwned, ser::Serialize};
use snafu::{OptionExt, ResultExt};
use std::marker::PhantomData;

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr};
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::traits::Storage;
use snafu::futures01::FutureExt;

pub struct TypedStorage<'a, S: Storage, T>
where
    T: Serialize + DeserializeOwned + NamedType,
{
    storage: &'a mut S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
}

impl<'a, S: Storage, T> TypedStorage<'a, S, T>
where
    T: Serialize + DeserializeOwned + NamedType,
{
    pub fn new(storage: &'a mut S) -> Self {
        TypedStorage {
            storage,
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> Result<()> {
        let bz = to_vec(data).context(SerializeErr {
            kind: T::short_type_name(),
        })?;
        self.storage.set(key, &bz);
        Ok(())
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&mut self, key: &[u8]) -> Result<T> {
        self.may_load(key)?.context(ContractErr {
            msg: "uninitialized data",
        })
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&mut self, key: &[u8]) -> Result<Option<T>> {
        let bz = self.storage.get(key);
        match bz {
            Some(d) => from_slice(&d).context(ParseErr {
                kind: T::short_type_name(),
            }),
            None => Ok(None),
        }
    }
}
