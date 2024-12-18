use std::error::Error;
use std::fmt::{Display, Write};
// TODO DB specific encoders

pub struct PGSQL {
    // successfully built string
    value: String,
    // failure message
    failure: Option<String>,
}

#[warn(dead_code)]
impl PGSQL {
    pub fn new() -> Self {
        Self {
            // avoid excessive realloc's
            value : String::with_capacity(4096),
            failure: None,
        }
    }

    // PRIVATE
    fn write_is_safe_as_is<T: Display>(&mut self, value: T) {
        write!(&mut self.value, "{}", value).unwrap();
    }

    // escape identifier
    fn escape_identifier(&mut self, value: &str) -> () {
        let first = value.chars().next();

        let any_to_escape = value.
            chars().
            any(|c| 
                !(c.is_ascii_lowercase() || c.is_ascii_digit() || 
                  c == '_'));

        match (first, first.unwrap_or('X').is_ascii_digit() || any_to_escape)  {
            // invalid identifier
            (None, _) =>
                 self.failure = Some("empty identifier".into()),

            // need to quote
            (_, true) => {
                // needs escape and quoting
                self.value.push('"');
                
                value.
                    chars().
                    for_each(|c| {
                        match c {
                            // disallowed to avoid \0 attacks on C code
                            '\0' => 
                                self.failure = Some("string contains the null character".into()),

                            // escape "
                            '"' => self.value.push_str("\"\""),

                            '\x08' => self.value.push_str("\\b"),
                            '\x0C' => self.value.push_str("\\f"),
                            '\n'   => self.value.push_str("\\n"),
                            '\r'   => self.value.push_str("\\r"),
                            '\t'   => self.value.push_str("\\t"),

                            // otherwise pass through
                            c => self.value.push(c),
                        };
                    });
                
                self.value.push('"');
            },

            // no change needed
            (_, _) => self.value.push_str(value),
        };
    }

    // escape string
    fn escape_string(&mut self, value: &str) -> () {
        let any_to_escape = value.
            chars().
            any(|c|
                match c {
                    // special case we will reject
                    '\0' => true,
                    // simply escape
                    '\'' | '\x08' | '\x0C' | '\n' | '\r' | '\t' => true,
                    _ => false
                });
        
        if any_to_escape {
            self.value.push_str("E\'");
            value.
                chars().
                for_each(|c| {
                    match c {
                        '\0'   => {
                            self.failure = Some("Zero character in string".into());
                        },
                        '\\'   => self.value.push_str("\\\\"),
                        '\''   => self.value.push_str("\\'"),
                        '\x08' => self.value.push_str("\\b"),
                        '\x0C' => self.value.push_str("\\f"),
                        '\n'   => self.value.push_str("\\n"),
                        '\r'   => self.value.push_str("\\r"),
                        '\t'   => self.value.push_str("\\t"),
                        c      => self.value.push(c),
                    }
                });
            self.value.push('\'');
            
        } else {
            self.value.push('\'');
            self.value.push_str(value);
            self.value.push('\'');
        }
    }

    // SQL text
    pub fn s(mut self, value: &str) -> Self {
        self.value += value;
        self
    }

    // comma
    pub fn c(mut self) -> Self {
        self.value += ", ";
        self
    }

    // dot
    pub fn dot(mut self) -> Self {
        self.value += ".";
        self
    }
    
    // new line
    pub fn nl(mut self) -> Self {
        self.value += "\n";
        self
    }

    // white space (if not already white space?? TODO)
    pub fn w(mut self) -> Self {
        self.value += " ";
        self
    }

    // Encoded SQL types
    pub fn text(mut self, value: &String) -> Self {
        self.escape_string(value);
        self
    }

    pub fn varchar(mut self, value: &String, max : usize) -> Self {
        let size = value.chars().count();
        if size > max {
            self.failure = Some(format!("varchar too long: {} vs {}", size, max));
        } else {
            self.escape_string(value);
        }
        self
    }
    
    
    // numbers
    pub fn smallint(mut self, value: i16) -> Self {
        self.write_is_safe_as_is(value);
        self
    }

    pub fn int(mut self, value: i32) -> Self {
        self.write_is_safe_as_is(value);
        self
    }

    pub fn integer(mut self, value: i32) -> Self {
        self.write_is_safe_as_is(value);
        self
    }

    pub fn bigint(mut self, value: i64) -> Self {
        self.write_is_safe_as_is(value);
        self
    }

    pub fn real(mut self, value: f32) -> Self {
        self.write_is_safe_as_is(value);
        self
    }

    pub fn double(mut self, value: f64) -> Self {
        self.write_is_safe_as_is(value);
        self
    }
    
    // TODO arrays for IN


    pub fn i(mut self, value: &String) -> Self {
        self.escape_identifier(value.as_str());
        self
    }

    pub fn build(&self) -> Result<&String, Box<dyn Error>> {
        if let Some(failure) = &self.failure {
            Err(failure.clone())?;
        }
        
        Ok(&self.value)
    }
}

pub fn pgsql() -> PGSQL {
    PGSQL::new()
}

