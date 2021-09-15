use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub struct Trap {
    msg: String
}

impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for Trap {}


impl Trap {
    pub fn new_box_err(msg: &str) -> Box<dyn Error> {
        Box::new(Self::new(msg))
    }

    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string()
        }
    }
}


#[derive(Debug)]
pub enum TrapCode<'a> {
    AmbigousLLRule(&'a str)
}

impl<'a> TrapCode<'a> {
    pub fn emit_box_err(&self) -> Box<dyn Error> {
        match self {
            Self::AmbigousLLRule(msg) => {
                Trap::new_box_err(
                    msg
                )
            },
            // _ => {
            //     Trap::new_box_err(
            //         format!(
            //             "{:#?}", self
            //         ).as_str()
            //     )
            // }
        }
    }
}
