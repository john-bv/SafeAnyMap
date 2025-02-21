use std::any::Any;
use std::any::type_name;
use std::any::type_name_of_val;
use std::any::TypeId;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::hash::Hash;
use std::vec::IntoIter;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SafeAnyMapError {
    #[error("Double inserts disallowed. Enable them by creating SafeAnyMap with `::new_double_inserts`")]
    DoubleInsert,
    #[error("Conflicting Value Type `{got:?}` must match existing value type `{exist:?}`")]
    ConflictingValueType {
        got: &'static str,
        exist: String
    },
    #[error("Downcast failed for given type `{got:?}`")]
    FailedDowncast {
        got: &'static str
    }
}

/// A dynamically typed store for information about the current context.
/// Our safety for this store comes in two ways:
/// -. On destruction, we drop the value with `Box::from_raw` ONCE avoiding memory leak and double free's.
pub struct SafeAnyMap<K> {
    /// Item Ids are used to identify the type of item stored.
    items: HashMap<K, *mut dyn Any>,
    relations: HashMap<K, TypeId>,
    allow_double_inserts: bool
}

impl<K> SafeAnyMap<K>
where K: Hash + Eq + Clone + std::fmt::Debug {
    pub fn new() -> Self {
        SafeAnyMap {
            items: HashMap::new(),
            relations: HashMap::new(),
            allow_double_inserts: false
        }
    }

    pub fn new_double_inserts() -> Self {
        SafeAnyMap {
            items: HashMap::new(),
            relations: HashMap::new(),
            allow_double_inserts: true
        }
    }

    /// Same as [std::collections::HashMap::keys]
    pub fn keys(&self) -> Keys<K, *mut dyn Any> {
        self.items.keys()
    }

    /// Same as [std::collections::HashMap::values]
    pub fn values(&self) -> IntoIter<&dyn Any> {
        unsafe {
            let vec = self.items.values().map(|v| &**v).collect::<Vec<_>>();
            vec.into_iter()
        }
    }

    /// Same as [std::collections::HashMap::values_mut]
    /// Unsafe cause there's no garauntee that the type will match the relations store
    /// This is up to the caller to do.
    pub unsafe fn values_mut(&mut self) -> IntoIter<&mut dyn Any> {
        unsafe {
            let vec = self.items.values().map(|v| &mut **v).collect::<Vec<_>>();
            vec.into_iter()
        }
    }

    /// If the value exists within the store, `Ok(Some(Box<T>))` is returned.
    pub fn insert<T: Sized>(&mut self, key: K, value: T) -> Result<Option<Box<T>>, SafeAnyMapError>
    where
        T: Any + Hash + 'static,
    {
        let boxed = Box::into_raw(Box::new(value));

        if !self.allow_double_inserts && self.items.contains_key(&key) {
            return Err(SafeAnyMapError::DoubleInsert);
        }

        if !self.check_or_insert_existing_relation::<T>(&key, boxed) {
            return Err(SafeAnyMapError::ConflictingValueType { got: type_name::<T>(), exist: type_name_of_val(&boxed).to_string() });
        }

        if let Some(bx) = self.items.insert(key, boxed) {
            // Safety: Box is only converted once here, its not possible to convert after
            //         we delete it.
            let value = unsafe { Box::from_raw(bx) };
            if let Ok(v) = value.downcast::<T>() {
                return Ok(Some(v));
            } else {
                return Err(SafeAnyMapError::FailedDowncast { got: type_name::<T>() })
            }
        }

        Ok(None)
    }

    pub fn get<T: Sized>(&self, key: &K) -> Option<&T>
    where
        T: Any + Hash + 'static,
    {
        // first check the existing relation
        if let Some(actual) = self.relations.get(key) {
            // we cant use contains because the type is not the same
            if *actual != TypeId::of::<T>() {
                return None;
            }

            if let Some(item) = self.items.get(key) {
                // deref *mut dyn Any -> dyn Any -> &dyn Any
                let value = unsafe { &**(item as *const *mut dyn Any) };

                return value.downcast_ref::<T>();
            }
        }

        None
    }

    pub fn get_mut<T: Sized>(&mut self, key: &K) -> Option<&mut T>
    where
        T: Any + Hash + 'static,
    {
        // first check the existing relation
        if let Some(actual) = self.relations.get(key) {
            // we cant use contains because the type is not the same
            if *actual != TypeId::of::<T>() {
                return None;
            }

            if let Some(item) = self.items.get_mut(key) {
                let value = unsafe { &mut **(item as *const *mut dyn Any) };

                return value.downcast_mut::<T>();
            }
        }

        None
    }

    /// `T` required to remove so we know we're trying to remove
    /// the right thing.
    pub fn remove<T: Sized>(&mut self, key: &K) -> Option<T>
    where
        T: Any + Hash + 'static,
    {
        if let Some(actual) = self.relations.get(key) {
            if *actual != TypeId::of::<T>() {
                return None;
            }

            if self.items.contains_key(key) {
                self.relations.remove(key);

                // SAFETY: Drop the value with box
                // we avoid double free since drop is only called here.
                if let Some(item) = self.items.remove(key) {
                    let value = unsafe { Box::from_raw(item) };

                    if let Ok(v) = value.downcast::<T>() {
                        return Some(*v);
                    } else {
                        return None;
                    }
                }
            }
        }

        None
    }

    fn check_or_insert_existing_relation<T: 'static>(
        &mut self,
        key: &K,
        value: *mut dyn Any,
    ) -> bool {
        let requested_type_id = TypeId::of::<T>();
        let value = unsafe { &*(value as *const dyn Any) };

        if let Some(actual) = self.relations.get(key) {
            // if we find the relation type within our existing relations, we can check if the value is of the same type
            if *actual == requested_type_id {
                return value.is::<T>();
            } else {
                return false;
            }
        } else {
            if value.is::<T>() {
                self.relations.insert(key.clone(), requested_type_id);
                return true;
            }
            false
        }
    }
}
