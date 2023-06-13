use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rules: super::rules::Rules,
    pub exporters: Vec<super::backend::Backend>,
}
