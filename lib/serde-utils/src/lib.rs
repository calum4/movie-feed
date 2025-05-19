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
    let str: &str = Deserialize::deserialize(deserializer)?;

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
            #[serde(deserialize_with = "deserialize_potentially_empty_string")]
            foo: Option<String>,
        }
        
        let empty_string: Data = serde_json::from_str(r#"{"foo": ""}"#).unwrap();
        assert_eq!(empty_string.foo, None);

        let non_empty_string: Data = serde_json::from_str(r#"{"foo": "bar"}"#).unwrap();
        assert_eq!(non_empty_string.foo, Some("bar".to_string()));
    }
}
