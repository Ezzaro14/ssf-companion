//! Interned identifiers: strings become integers here and stay integers.

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub mod ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(pub u16);
impl TagId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub u32);
impl GroupId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModId(pub u32);
impl ModId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaseId(pub u32);
impl BaseId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatId(pub u32);
impl StatId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId(pub u16);
impl DomainId {
    #[must_use]
    pub const fn index(self) -> usize { self.0 as usize }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// A string interner that assigns a unique integer ID to each unique string.
pub struct Interner {
    names: Vec<String>,             
    lookup: FxHashMap<String, u32>,  
}
impl Interner {
    pub fn intern(&mut self, s: &str) -> u32 {
        //look up the string in the hashmap, if it exists return the id, otherwise add it to the vector and hashmap and return the new id
        if let Some(&id) = self.lookup.get(s) {     
            return id;                              
        }
        let id = self.names.len() as u32;          
        self.names.push(s.to_owned());             
        self.lookup.insert(s.to_owned(), id);
        id;
    }
}