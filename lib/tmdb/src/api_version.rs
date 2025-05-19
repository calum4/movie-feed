#[derive(Debug, Copy, Clone)]
pub enum ApiVersion {
    V3,
}

impl ApiVersion {
    pub fn base_path(&self) -> &'static str {
        match self {
            ApiVersion::V3 => "3/",
        }
    }
}
