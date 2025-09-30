use std::str::FromStr;

use heapless::String as HeaplessString;

use crate::coreerror;


#[derive(Debug, Clone)]
pub enum String {
    Tiny(HeaplessString<32>),
    Small(HeaplessString<64>),
    Medium(HeaplessString<128>),
    Large(std::string::String),
}

impl FromStr for String {

    type Err = coreerror::CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();

        let res = match len {
            0..33   => {Self::Tiny(HeaplessString::try_from(s)?)},
            33..65  => {Self::Small(HeaplessString::try_from(s)?)},
            65..129 => {Self::Medium(HeaplessString::try_from(s)?)},
            129..   => {Self::Large(std::string::String::from_str(s)?)}
        };

        Ok(res)
        // TODO: Write tests!
    }

}