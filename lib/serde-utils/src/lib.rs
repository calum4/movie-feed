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
