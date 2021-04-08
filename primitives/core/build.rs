// Copyright (C) 2020 Mangata team
// Based on Snowfork bridge implementation

use std::error::Error;
use std::fmt;

/// Wraps `&'static str` and implements the `Error` trait for it.
#[derive(Debug)]
struct StringError {
    error: &'static str,
}

impl StringError {
	#[allow(dead_code)]
    fn new(error: &'static str) -> Self {
        return StringError { error };
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        self.error
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
