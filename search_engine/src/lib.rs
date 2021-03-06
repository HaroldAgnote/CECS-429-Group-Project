extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate stemmer;
extern crate byteorder;

pub mod index;
pub mod parser;
pub mod paths;
pub mod processor;
pub mod reader;
pub mod classifier;