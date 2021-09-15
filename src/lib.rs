
pub mod dsl;
pub mod gram;
pub mod parser;
pub mod error;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VerboseLv {
    V0,
    V1,
    V2
}

thread_local! {
    pub static VERBOSE: VerboseLv = VerboseLv::V0
}


#[cfg(test)]
mod tests {

}
