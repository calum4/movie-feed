use serde::{Deserialize, Deserializer};

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Eq, PartialEq)]
pub enum Gender {
    NotSpecified,
    Female,
    Male,
    NonBinary,
}

impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let gender = u8::deserialize(deserializer)?;

        Ok(match gender {
            1 => Self::Female,
            2 => Self::Male,
            3 => Self::NonBinary,
            _ => Self::NotSpecified,
        })
    }
}

impl Default for Gender {
    fn default() -> Self {
        Self::NotSpecified
    }
}
