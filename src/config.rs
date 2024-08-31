use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ColeusConfig {
    pub name: String,
    pub id: String,
    pub path: String,
    pub lang_path: String,
}