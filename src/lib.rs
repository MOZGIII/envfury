#![doc = include_str!("../README.md")]

use std::{env::VarError, str::FromStr};

pub mod custom;

pub use custom::Custom;

/// Error from reading the env var.
#[derive(Debug, thiserror::Error)]
#[error("error reading {key} env var: {reason}")]
pub struct Error<T> {
    /// The envirobment variable name.
    pub key: &'static str,
    /// The reason for the error.
    #[source]
    pub reason: T,
}

impl<T> Error<T> {
    /// Create a new [`Error`].
    pub fn new(key: &'static str, reason: T) -> Self {
        Self { key, reason }
    }

    /// Map the reason to change the error type.
    pub fn map_reason<U, F>(self, map: F) -> Error<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { key, reason } = self;
        let reason = (map)(reason);
        Error { key, reason }
    }
}

/// Error while processing the value.
#[derive(Debug, thiserror::Error)]
pub enum ValueError<T> {
    /// The value was not a valid unicode.
    #[error("value is not a valid unicode")]
    NonUnicode,
    /// The value could not be parsed from a string.
    #[error("unable to parse: {0}")]
    Parse(#[source] T),
}

/// Error while processing a required variable.
#[derive(Debug, thiserror::Error)]
pub enum MustError<T> {
    /// The variable was not set.
    #[error("not set")]
    NotSet,
    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<T>),
}

/// Error while processing a variable or parsing it's default.
#[derive(Debug, thiserror::Error)]
pub enum OrParseError<T> {
    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<T>),
    /// The default could not be properly parsed.
    #[error("unable to parse the default value while the variable was not set: {0}")]
    ParseDefault(T),
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns [`None`].
/// Returns an error if the value is an invalid unicode or if the value could not be parsed.
pub fn maybe<T>(key: &'static str) -> Result<Option<T>, Error<ValueError<T::Err>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let val = match std::env::var(key) {
        Ok(val) => val,
        Err(VarError::NotPresent) => return Ok(None),
        Err(VarError::NotUnicode(_)) => return Err(Error::new(key, ValueError::NonUnicode)),
    };
    let val = val
        .parse()
        .map_err(|err| Error::new(key, ValueError::Parse(err)))?;
    Ok(Some(val))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// Returns an error if the value is not set, is an invalid unicode or if the value could not
/// be parsed.
pub fn must<T>(key: &'static str) -> Result<T, Error<MustError<T::Err>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match maybe(key) {
        Ok(Some(val)) => Ok(val),
        Ok(None) => Err(Error::new(key, MustError::NotSet)),
        Err(err) => Err(err.map_reason(MustError::Value)),
    }
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the `default` argument.
/// Returns an error if the value is an invalid unicode or if the value could not be parsed.
pub fn or<T>(key: &'static str, default: T) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or(default))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the result of calling the `default` argument.
/// Returns an error if the value is an invalid unicode or if the value could not be parsed.
pub fn or_else<T, F>(key: &'static str, default: F) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
    F: FnOnce() -> T,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or_else(default))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the result of calling the `default` and parsing the default
/// argument.
/// Returns an error if the variable value is an invalid unicode or if the value could not be
/// parsed, or if the default could not be parsed.
pub fn or_parse<T>(
    key: &'static str,
    default: impl Into<String>,
) -> Result<T, Error<OrParseError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    let val = maybe(key).map_err(|err| err.map_reason(OrParseError::Value))?;
    if let Some(val) = val {
        return Ok(val);
    }
    let val = default
        .into()
        .parse()
        .map_err(|err| Error::new(key, OrParseError::ParseDefault(err)))?;
    Ok(val)
}
