use validator::Validate;
use crate::utils::Page;
use chrono::prelude::{DateTime, Local};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct GetUser {
    pub(crate) name: Option<String>,
    pub(crate) page: Page,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ResUser {
    username: String,
    email: String,
    phone: String,
    create_at: DateTime<Local>,
}