use named_type::NamedType;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use cosmwasm::errors::Result;
use cosmwasm::traits::Storage;

use crate::namespace_helpers::{get_with_prefix, key_prefix, key_prefix_nested, set_with_prefix};
use crate::type_helpers::{deserialize, may_deserialize, serialize};

pub fn bucket<'a, S: Storage, T>(namespace: &[u8], storage: &'a mut S) -> Bucket<'a, S, T>
where
    T: Serialize + DeserializeOwned + NamedType,
{
    Bucket::new(namespace, storage)
}

pub struct Bucket<'a, S: Storage, T>
where
    T: Serialize + DeserializeOwned + NamedType,
{
    storage: &'a mut S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
    prefix: Vec<u8>,
}

impl<'a, S: Storage, T> Bucket<'a, S, T>
where
    T: Serialize + DeserializeOwned + NamedType,
{
    pub fn new(namespace: &[u8], storage: &'a mut S) -> Self {
        Bucket {
            prefix: key_prefix(namespace),
            storage,
            data: PhantomData,
        }
    }

    pub fn multilevel(namespaces: &[&[u8]], storage: &'a mut S) -> Self {
        Bucket {
            prefix: key_prefix_nested(namespaces),
            storage,
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> Result<()> {
        set_with_prefix(self.storage, &self.prefix, key, &serialize(data)?);
        Ok(())
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> Result<T> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> Result<Option<T>> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        may_deserialize(&value)
    }

    /// update will load the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful
    ///
    /// This is the least stable of the APIs, and definitely needs some usage
    pub fn update(&mut self, key: &[u8], action: &dyn Fn(T) -> Result<T>) -> Result<T> {
        let input = self.load(key)?;
        let output = action(input)?;
        self.save(key, &output)?;
        Ok(output)
    }
}
