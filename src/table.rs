use std::fs::{File, OpenOptions, rename};
use std::io;
use std::io::prelude::*;
// use std::io::Write;
use std::collections::BTreeMap;

use encode::Encode;
use decode::*;

type DataMap = BTreeMap<i64,Vec<u8>>;

pub struct Table {
    file: Option<File>,
    map: DataMap,
}

#[derive(Debug)]
pub enum TableError {
    IOError(io::Error),
    DecodeError(DecodeError),
    NotWritable,
}

impl Table {
    pub fn open(path: &str) -> Result<Table, TableError> {
        let mut f = match File::open(path) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(f) => f,
        };
        let mut stats = DecodeStats::default();
        let m = match DataMap::decode_stats(&mut f, &mut stats) {
            Err(de) => return Err(TableError::DecodeError(de)),
            Ok(m) => m,
        };
        println!("read: {} discarded: {}", stats.read(), stats.discarded());
        Ok(Table { file: None, map: m })
    }

    pub fn open_rw(path: &str) -> Result<Table, TableError> {
        let f_or_e = OpenOptions::new().read(true).write(true).create(true).open(path);
        let mut f = match f_or_e {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(f) => f,
        };
        let mut stats = DecodeStats::default();
        let m = match DataMap::decode_stats(&mut f, &mut stats) {
            Err(de) => return Err(TableError::DecodeError(de)),
            Ok(m) => m,
        };
        println!("read: {} discarded: {}", stats.read(), stats.discarded());
        Ok(Table { file: Some(f), map: m })
    }

    pub fn compact(&mut self, path: &str) -> Result<(), TableError> {
        // We must be open rw: close the file.
        match self.file {
            None => return Err(TableError::NotWritable),
            Some(ref _f) => (),
        };
        self.file = None;

        // Path for temporary file is path + "~".
        let mut newpath = path.to_string();
        newpath.push('~');

        let f_or_e = OpenOptions::new().write(true).create(true).truncate(true).open(&newpath);
        let mut f = match f_or_e {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(f) => f,
        };

        // Write contents of map to the temporary file.
        match self.map.encode(&mut f) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };

        // On all unixes we ought to be able to do the rename
        // and hold onto f as our file handle, but we will just close
        // before we rename for now.
        drop(f);

        // Rename it to be the main file.
        match rename(newpath, path) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };

        // Open it again.
        let f_or_e = OpenOptions::new().append(true).open(path);
        match f_or_e {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(f) => { self.file = Some(f); },
        };

        Ok(())
    }

    pub fn get(&self, key: &i64) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<Option<Vec<u8>>, TableError> {
        // Get the file handle (which is rw if present).
        let mut f = match self.file {
            None => return Err(TableError::NotWritable),
            Some(ref mut f) => f,
        };
        // Append the new value to the file.
        match key.encode(&mut f) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };
        match value.as_slice().encode(&mut f) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };
        // Update the map in memory.
        Ok(self.map.insert(key, value))
    }

    pub fn remove(&mut self, key: &i64) -> Result<Option<Vec<u8>>, TableError> {
        // Get the file handle (which is rw if present).
        let mut f = match self.file {
            None => return Err(TableError::NotWritable),
            Some(ref mut f) => f,
        };
        // Append (key, None) to the file.
        match key.encode(&mut f) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };
        match None.encode(&mut f) {
            Err(ioerr) => return Err(TableError::IOError(ioerr)),
            Ok(()) => (),
        };
        Ok(self.map.remove(key))
    }
}

impl IntoIterator for Table {
    type Item = (i64, Vec<u8>);
    type IntoIter = ::std::collections::btree_map::IntoIter<i64,Vec<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a Table {
    type Item = (&'a i64, &'a Vec<u8>);
    type IntoIter = ::std::collections::btree_map::Iter<'a, i64, Vec<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
