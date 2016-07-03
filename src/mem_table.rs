use std::collections::{BTreeMap, Bound};
use std::collections::btree_map::Range;

use json::JSON;

pub type JSONRow = BTreeMap<String, JSON>;

pub trait TableObserver {
    fn update(&mut self, key: i64, old: Option<&JSONRow>, new: Option<&JSONRow>);
}

pub struct MemTable {
    map: BTreeMap<i64,JSONRow>,
    obs: Box<TableObserver>,
}

fn opt_ref<'a, T>(opt: &'a Option<T>) -> Option<&'a T> {
    match *opt {
        Some(ref t) => Some(t),
        None => None,
    }
}

impl MemTable {

    pub fn get(&self, key: &i64) -> Option<&JSONRow> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: i64, value: JSONRow) -> Option<JSONRow> {
        let old = self.map.insert(key, value);
        let new = self.map.get(&key);
        self.obs.update(key, opt_ref(&old), new);
        old
    }
}
