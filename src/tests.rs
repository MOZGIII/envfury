//! Each test reads its own variable, so the tests don't interfere when run in parallel.

use super::*;

/// Assert that the key of `err` is borrowed and equal to `expected`.
fn assert_borrowed<T>(err: &Error<T>, expected: &str) {
    assert!(
        matches!(err.key, Cow::Borrowed(key) if key == expected),
        "expected a borrowed {expected:?}, got {:?}",
        err.key
    );
}

/// Assert that the key of `err` is owned and equal to `expected`.
fn assert_owned<T>(err: &Error<T>, expected: &str) {
    assert!(
        matches!(&err.key, Cow::Owned(key) if key == expected),
        "expected an owned {expected:?}, got {:?}",
        err.key
    );
}

/// A static key yields a borrowed key in a `maybe` parse error.
#[test]
fn maybe_parse_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_MAYBE_PARSE";
    std::env::set_var(KEY, "nope");

    let err = maybe::<u32>(KEY).unwrap_err();

    assert!(matches!(err.reason, ValueError::Parse(_)));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in a `must` not-set error.
#[test]
fn must_not_set_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_MUST_NOT_SET";
    std::env::remove_var(KEY);

    let err = must::<u32>(KEY).unwrap_err();

    assert!(matches!(err.reason, MustError::NotSet));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in a `must` parse error.
#[test]
fn must_parse_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_MUST_PARSE";
    std::env::set_var(KEY, "nope");

    let err = must::<u32>(KEY).unwrap_err();

    assert!(matches!(err.reason, MustError::Value(ValueError::Parse(_))));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or` parse error, which comes from `maybe`.
#[test]
fn or_parse_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_PARSE";
    std::env::set_var(KEY, "nope");

    let err = or::<u32>(KEY, 1).unwrap_err();

    assert!(matches!(err.reason, ValueError::Parse(_)));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_else_try` parse error.
#[test]
fn or_else_try_parse_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_ELSE_TRY_PARSE";
    std::env::set_var(KEY, "nope");

    let err = or_else_try::<u32, &str, _>(KEY, || Ok(1)).unwrap_err();

    assert!(matches!(
        err.reason,
        OrElseTryError::Value(ValueError::Parse(_))
    ));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_else_try` error from the handler.
#[test]
fn or_else_try_else_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_ELSE_TRY_ELSE";
    std::env::remove_var(KEY);

    let err = or_else_try::<u32, &str, _>(KEY, || Err("failed")).unwrap_err();

    assert!(matches!(err.reason, OrElseTryError::Else("failed")));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_parse` parse error.
#[test]
fn or_parse_value_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_PARSE_VALUE";
    std::env::set_var(KEY, "nope");

    let err = or_parse::<u32>(KEY, "1").unwrap_err();

    assert!(matches!(
        err.reason,
        OrParseError::Value(ValueError::Parse(_))
    ));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_parse` error from the default.
#[test]
fn or_parse_default_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_PARSE_DEFAULT";
    std::env::remove_var(KEY);

    let err = or_parse::<u32>(KEY, "nope").unwrap_err();

    assert!(matches!(err.reason, OrParseError::ParseDefault(_)));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_try_into` parse error.
#[test]
fn or_try_into_value_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_TRY_INTO_VALUE";
    std::env::set_var(KEY, "nope");

    let err = or_try_into::<u32, i64>(KEY, 1).unwrap_err();

    assert!(matches!(
        err.reason,
        OrTryIntoError::Value(ValueError::Parse(_))
    ));
    assert_borrowed(&err, KEY);
}

/// A static key yields a borrowed key in an `or_try_into` error from the default.
#[test]
fn or_try_into_default_error_borrows_static_key() {
    const KEY: &str = "ENVFURY_TEST_OR_TRY_INTO_DEFAULT";
    std::env::remove_var(KEY);

    let err = or_try_into::<u32, i64>(KEY, -1).unwrap_err();

    assert!(matches!(err.reason, OrTryIntoError::TryInto(_)));
    assert_borrowed(&err, KEY);
}

/// A runtime key yields an owned key in a `must` not-set error.
#[test]
fn must_not_set_owns_runtime_key() {
    let prefix = "MUST_RUNTIME";
    let key = format!("ENVFURY_TEST_{prefix}_NOT_SET");
    std::env::remove_var(&key);

    let err = must::<u32>(key.clone()).unwrap_err();

    assert!(matches!(err.reason, MustError::NotSet));
    assert_owned(&err, &key);
}

/// A runtime key yields an owned key in an `or_parse` parse error.
#[test]
fn or_parse_value_error_owns_runtime_key() {
    let prefix = "OR_PARSE_RUNTIME";
    let key = format!("ENVFURY_TEST_{prefix}_VALUE");
    std::env::set_var(&key, "nope");

    let err = or_parse::<u32>(key.clone(), "1").unwrap_err();

    assert!(matches!(
        err.reason,
        OrParseError::Value(ValueError::Parse(_))
    ));
    assert_owned(&err, &key);
}

/// A runtime key reads the variable it names.
#[test]
fn maybe_reads_runtime_key() {
    let prefix = "MAYBE_RUNTIME";
    let key = format!("ENVFURY_TEST_{prefix}_VALUE");
    std::env::set_var(&key, "42");

    assert_eq!(maybe::<u32>(key).unwrap(), Some(42));
}
