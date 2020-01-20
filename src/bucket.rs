//use named_type::NamedType;
//use serde::{de::DeserializeOwned, ser::Serialize};
//
//use cosmwasm::traits::Storage;
//use crate::{PrefixedStorage, TypedStorage};
//
//pub fn bucket<'a, S: Storage, T>(namespace: &[u8], storage: &'a mut S) -> Bucket<'a, S, T>
//    where
//        T: Serialize + DeserializeOwned + NamedType,
//{
//    Bucket::new(namespace, storage)
//}
//
//pub struct Bucket<'a, S: Storage, T>
//    where
//        T: Serialize + DeserializeOwned + NamedType,
//{
//    prefix: PrefixedStorage<'a, S>,
//    typed: TypedStorage<'a, PrefixedStorage<'a, S>, T>,
//}
//
//impl<'a, S: Storage, T> Bucket<'a, S, T>
//    where
//        T: Serialize + DeserializeOwned + NamedType,
//{
//    pub fn new(namespace: &[u8], storage: &'a mut S) -> Self {
//        let mut prefix = PrefixedStorage::new(namespace, storage);
//        let typed = TypedStorage::<_, T>::new(&mut prefix);
//        Bucket { prefix, typed }
//    }
//}
