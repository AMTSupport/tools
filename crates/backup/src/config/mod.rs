pub mod backend;
pub mod rules;
pub mod runtime;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub rules: rules::Rules,
    pub exporters: Vec<backend::Backend>,
}
