use std::{error, fmt};

#[derive(Debug)]
pub struct LibraryNotLinked {}

impl LibraryNotLinked {
    pub fn new() -> LibraryNotLinked {
        LibraryNotLinked {}
    }
}

impl fmt::Display for LibraryNotLinked {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No library linked!")
    }
}

impl error::Error for LibraryNotLinked {}
