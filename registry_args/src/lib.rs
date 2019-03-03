extern crate validates;

use std::str::FromStr;
use std::sync::Arc;
use validates::ValidationResult;

pub trait RegistryArg: Send + Sized + Sync {
    fn parse(arg: &str) -> ValidationResult<Self>;
}

impl RegistryArg for Arc<str> {
    fn parse(arg: &str) -> ValidationResult<Arc<str>> {
        return Result::Ok(Arc::from(arg));
    }
}

pub trait RegistryArgs: Send + Sized + Sync {
    fn help_meta_suffix() -> &'static str;
    fn argct() -> usize;
    fn parse(args: &[&str]) -> ValidationResult<Self>;
}

pub trait MayRegistryArgFromStr {
}

impl<T: FromStr + MayRegistryArgFromStr + Send + Sync> RegistryArg for T where T::Err: std::error::Error {
    fn parse(arg: &str) -> ValidationResult<T> {
        return Result::Ok(T::from_str(arg)?);
    }
}

impl MayRegistryArgFromStr for usize {
}
