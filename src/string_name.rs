use indexmap::IndexSet;
use once_cell::sync::Lazy;
use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
    mem::transmute,
    sync::RwLock,
};

static INTERN_MAP: Lazy<RwLock<IndexSet<String>>> = Lazy::new(|| RwLock::new(IndexSet::new()));

fn intern(name: &str) -> usize {
    let intern_map = INTERN_MAP.read().unwrap();

    match intern_map.get_index_of(name) {
        Some(index) => index,
        None => {
            drop(intern_map);

            let mut intern_map = INTERN_MAP.write().unwrap();
            let (index, _) = intern_map.insert_full(name.to_string());

            if cfg!(debug_assertions) {
                println!("Interned {}: {:?}", index, name);
            }

            index
        }
    }
}

/// An immutable string that is interned. The strings are simply compared by comparing the pointers.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringName(usize);
impl StringName {
    pub fn as_str(&self) -> &'static str {
        unsafe { transmute(INTERN_MAP.read().unwrap()[self.0].as_str()) }
    }
}
impl Display for StringName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Debug for StringName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringName({}: {:?})", self.0, self.as_str())
    }
}
impl From<&str> for StringName {
    fn from(name: &str) -> Self {
        Self(intern(name))
    }
}
impl From<String> for StringName {
    fn from(name: String) -> Self {
        Self(intern(name.as_str()))
    }
}
impl From<StringName> for String {
    fn from(name: StringName) -> Self {
        name.as_str().to_string()
    }
}
impl From<StringName> for &str {
    fn from(name: StringName) -> Self {
        name.as_str()
    }
}
