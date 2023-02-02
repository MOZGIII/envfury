//! Custom parsing.
//!
//! See [`Custom`].

/// Use [`Custom`] to override the standard parsing implementation for your type.
///
/// # Example
///
/// ```no_run
/// use envfury::custom::FromStr;
///
/// impl envfury::custom::FromStr for u8 {
///     type Err = &'static str;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         if (s == "one") {
///             Ok(1)
///         } else if (s == "two") {
///             Ok(2)
///         } else {
///             Err(r#"not "one" or "two""#)
///         }
///     }
/// }
///
/// let envfury::Custom::<u8>(myvar) = envfury::must("MY_ONE_OR_TWO");
/// ```
pub struct Custom<T>(pub T);

/// A custom [`FromStr`] trait to enable customization of value parsing.
///
/// Implement this trait for any type that you want to override the parsing for.
///
/// This type is analogus to [`std::str::FromStr`], see that for details.
pub trait FromStr: Sized {
    /// See [`std::str::FromStr::Err`].
    type Err;

    /// See [`std::str::FromStr::from_str`].
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

impl<T> std::str::FromStr for Custom<T>
where
    T: self::FromStr,
{
    type Err = <T as self::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = <T as self::FromStr>::from_str(s)?;
        Ok(Custom(val))
    }
}
