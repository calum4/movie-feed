use serde::{Deserialize, Deserializer};

pub const fn bool_true() -> bool {
    true
}

pub const fn bool_false() -> bool {
    false
}

pub fn vec_zero_size<T>() -> Vec<T> {
    Vec::with_capacity(0)
}

pub fn deserialize_potentially_empty_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str: Option<String> = Deserialize::deserialize(deserializer)?;

    let str = match str {
        None => return Ok(None),
        Some(str) => str,
    };

    if str.is_empty() {
        return Ok(None);
    }

    Ok(Some(str.to_string()))
}

/// Copied from [serde-aux](https://github.com/iddm/serde-aux/blob/e34b5f86b4d9d009daee9a8673939a27365416c0/src/field_attributes.rs#L798)
///
/// Deserializes default value from nullable value or empty object. If the original value is `null` or `{}`,
/// `Default::default()` is used.
///
/// # Example:
///
/// ```rust
/// use serde_utils::*;
///
/// #[derive(serde::Deserialize, Debug)]
/// struct MyStruct {
///     #[serde(deserialize_with = "deserialize_default_from_empty_object")]
///     empty_as_default: Option<MyInnerStruct>,
/// }
///
/// #[derive(serde::Deserialize, Debug)]
/// struct MyInnerStruct {
///     mandatory: u64,
/// }
///
/// let s = r#" { "empty_as_default": { "mandatory": 42 } } "#;
/// let a: MyStruct = serde_json::from_str(s).unwrap();
/// assert_eq!(a.empty_as_default.unwrap().mandatory, 42);
///
/// let s = r#" { "empty_as_default": null } "#;
/// let a: MyStruct = serde_json::from_str(s).unwrap();
/// assert!(a.empty_as_default.is_none());
///
/// let s = r#" { "empty_as_default": {} } "#;
/// let a: MyStruct = serde_json::from_str(s).unwrap();
/// assert!(a.empty_as_default.is_none());
///
/// let s = r#" { "empty_as_default": { "unknown": 42 } } "#;
/// assert!(serde_json::from_str::<MyStruct>(s).is_err());
/// ```
pub fn deserialize_default_from_empty_object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct EmptyObject {}

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum EmptyOrNot<Y> {
        NonEmpty(Y),
        Empty(EmptyObject),
        Null,
    }

    let empty_or_not: EmptyOrNot<T> = EmptyOrNot::deserialize(deserializer)?;

    match empty_or_not {
        EmptyOrNot::NonEmpty(e) => Ok(e),
        _ => Ok(T::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_potentially_empty_string() {
        #[derive(Debug, Deserialize)]
        struct Data {
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            foo: Option<String>,
        }

        let non_existent: Data = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(non_existent.foo, None);

        let null: Data = serde_json::from_str(r#"{"foo": null}"#).unwrap();
        assert_eq!(null.foo, None);

        let empty_string: Data = serde_json::from_str(r#"{"foo": ""}"#).unwrap();
        assert_eq!(empty_string.foo, None);

        let non_empty_string: Data = serde_json::from_str(r#"{"foo": "bar"}"#).unwrap();
        assert_eq!(non_empty_string.foo, Some("bar".to_string()));
    }
}
