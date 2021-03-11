use chrono::prelude::{DateTime, Local};
use validator::Validate;
use serde_json::Value;
use crate::utils::my_date_format;

#[derive(Deserialize, Serialize, Debug, Validate)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) data_type: String,
    pub(crate) max: Option<usize>,
    pub(crate) min: Option<usize>,
    pub(crate) length_min: Option<usize>,
    pub(crate) length_max: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub(crate) struct Interface {
    pub(crate) url: String,
    pub(crate) description: String,
    pub(crate) module: String,
    pub(crate) method: String,
    #[validate]
    pub(crate) data: Vec<Field>,
    #[validate]
    pub(crate) param: Vec<Field>,
    #[serde(with = "my_date_format")]
    pub(crate) create_at: DateTime<Local>,
}
