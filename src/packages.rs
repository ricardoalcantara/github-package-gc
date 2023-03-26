use serde::{Deserialize, Serialize};

pub type Packages = Vec<Package>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(rename = "package_html_url")]
    pub package_html_url: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub metadata: Metadata,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(rename = "package_type")]
    pub package_type: String,
    pub container: Container,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub tags: Vec<String>,
}
