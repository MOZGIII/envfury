#![doc = include_str!("../README.md")]

use std::{borrow::Cow, env::VarError, str::FromStr};

/// Error from reading the env var.
#[derive(Debug, thiserror::Error)]
#[error("error reading {key} env var: {reason}")]
pub struct Error<T> {
    /// The envirobment variable name.
    ///
    /// Borrowed for `'static` keys, owned for keys built at runtime.
    pub key: Cow<'static, str>,

    /// The reason for the error.
    #[source]
    pub reason: T,
}

impl<T> Error<T> {
    /// Create a new [`Error`].
    ///
    /// The `key` can be a `&'static str` or a [`String`] built at runtime.
    pub fn new(key: impl Into<Cow<'static, str>>, reason: T) -> Self {
        Self {
            key: key.into(),
            reason,
        }
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
pub enum ValueError<ParseError> {
    /// The value was not a valid unicode.
    #[error("value is not a valid unicode")]
    NonUnicode,

    /// The value could not be parsed from a string.
    #[error("unable to parse: {0}")]
    Parse(#[source] ParseError),
}

/// Error while processing a required variable.
#[derive(Debug, thiserror::Error)]
pub enum MustError<ParseError> {
    /// The variable was not set.
    #[error("not set")]
    NotSet,

    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<ParseError>),
}

/// Error while processing a variable or parsing it's default.
#[derive(Debug, thiserror::Error)]
pub enum OrParseError<ParseError> {
    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<ParseError>),

    /// The default could not be properly parsed.
    #[error("unable to parse the default value while the variable was not set: {0}")]
    ParseDefault(ParseError),
}

/// Error while processing a variable or executing a fallible else.
#[derive(Debug, thiserror::Error)]
pub enum OrElseTryError<ParseError, ElseError> {
    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<ParseError>),

    /// The `else` handler failed.
    #[error("the absent value handler failed: {0}")]
    Else(#[source] ElseError),
}

/// Error while processing a variable or executing a [`TryInto::try_into`].
#[derive(Debug, thiserror::Error)]
pub enum OrTryIntoError<ParseError, TryIntoError> {
    /// The value couldn't be processed.
    #[error(transparent)]
    Value(ValueError<ParseError>),

    /// The default could not be properly converted.
    #[error("unable to convert the default value while the variable was not set: {0}")]
    TryInto(#[source] TryIntoError),
}

impl<ParseError> From<OrParseError<ParseError>> for OrElseTryError<ParseError, ParseError> {
    fn from(value: OrParseError<ParseError>) -> Self {
        match value {
            OrParseError::Value(value_error) => OrElseTryError::Value(value_error),
            OrParseError::ParseDefault(parse_error) => OrElseTryError::Else(parse_error),
        }
    }
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns [`None`].
/// Returns an error if the value is invalid Unicode or if the value could not be parsed.
pub fn maybe<T>(key: impl Into<Cow<'static, str>>) -> Result<Option<T>, Error<ValueError<T::Err>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let key = key.into();
    lookup(&key).map_err(|reason| Error::new(key, reason))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// Returns an error if the value is not set, is invalid Unicode or if the value could not
/// be parsed.
pub fn must<T>(key: impl Into<Cow<'static, str>>) -> Result<T, Error<MustError<T::Err>>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let key = key.into();
    match lookup(&key) {
        Ok(Some(val)) => Ok(val),
        Ok(None) => Err(Error::new(key, MustError::NotSet)),
        Err(reason) => Err(Error::new(key, MustError::Value(reason))),
    }
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns a newly constructed value via the [`Default`] trait.
/// Returns an error if the value is invalid Unicode or if the value could not be parsed.
pub fn or_default<T>(key: impl Into<Cow<'static, str>>) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    T: Default,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or_default())
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the `default` argument.
/// Returns an error if the value is invalid Unicode or if the value could not be parsed.
pub fn or<T>(key: impl Into<Cow<'static, str>>, default: T) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or(default))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the result of calling `.into()` on the `default` argument.
/// Returns an error if the value is invalid Unicode or if the value could not be parsed.
pub fn or_into<T>(
    key: impl Into<Cow<'static, str>>,
    default: impl Into<T>,
) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or(default.into()))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the result of calling the `f` argument.
/// Returns an error if the value is invalid Unicode or if the value could not be parsed.
pub fn or_else<T, F>(
    key: impl Into<Cow<'static, str>>,
    f: F,
) -> Result<T, Error<ValueError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    F: FnOnce() -> T,
{
    let val = maybe(key)?;
    Ok(val.unwrap_or_else(f))
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the `Ok`-result of calling the `f` argument.
/// Returns an error if the variable value is invalid Unicode, if the value could not be parsed,
/// or if calling `f` resulted in an error.
pub fn or_else_try<T, E, F>(
    key: impl Into<Cow<'static, str>>,
    f: F,
) -> Result<T, Error<OrElseTryError<T::Err, E>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    F: FnOnce() -> Result<T, E>,
{
    let key = key.into();
    let val = match lookup(&key) {
        Ok(val) => val,
        Err(reason) => return Err(Error::new(key, OrElseTryError::Value(reason))),
    };
    if let Some(val) = val {
        return Ok(val);
    }
    let val = f().map_err(|err| Error::new(key, OrElseTryError::Else(err)))?;
    Ok(val)
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the result of calling the `default` and parsing the default
/// argument.
/// Returns an error if the variable value is invalid Unicode, if the value could not be parsed,
/// or if the `default` could not be parsed.
pub fn or_parse<T>(
    key: impl Into<Cow<'static, str>>,
    default: impl Into<String>,
) -> Result<T, Error<OrParseError<T::Err>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let key = key.into();
    let val = match lookup(&key) {
        Ok(val) => val,
        Err(reason) => return Err(Error::new(key, OrParseError::Value(reason))),
    };
    if let Some(val) = val {
        return Ok(val);
    }
    let val = default
        .into()
        .parse()
        .map_err(|err| Error::new(key, OrParseError::ParseDefault(err)))?;
    Ok(val)
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// If the variable is not set - returns the `Ok`-result of calling `.try_into()` on the `default`
/// argument.
/// Returns an error if the variable value is invalid Unicode, if the value could not be parsed,
/// or if the `default` could not be parsed.
#[allow(
    clippy::type_complexity,
    reason = "Not much to simplify here, only type aliases are available to our disposal - \
              but they're not too useful here."
)]
pub fn or_try_into<T, D>(
    key: impl Into<Cow<'static, str>>,
    default: D,
) -> Result<T, Error<OrTryIntoError<T::Err, D::Error>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    D: TryInto<T>,
{
    let key = key.into();
    let val = match lookup(&key) {
        Ok(val) => val,
        Err(reason) => return Err(Error::new(key, OrTryIntoError::Value(reason))),
    };
    if let Some(val) = val {
        return Ok(val);
    }
    let val = default
        .try_into()
        .map_err(|err| Error::new(key, OrTryIntoError::TryInto(err)))?;
    Ok(val)
}

/// Get the value of environment variable `key` and parse it into the type `T` if variable is set.
/// The callers own the key and build the [`Error`] from the returned reason.
fn lookup<T>(key: &str) -> Result<Option<T>, ValueError<T::Err>>
where
    T: FromStr,
{
    let val = match std::env::var(key) {
        Ok(val) => val,
        Err(VarError::NotPresent) => return Ok(None),
        Err(VarError::NotUnicode(_)) => return Err(ValueError::NonUnicode),
    };
    let val = val.parse().map_err(ValueError::Parse)?;
    Ok(Some(val))
}
