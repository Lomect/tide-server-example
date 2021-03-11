use crate::auth::Register;
use crate::utils::hash_password;
use chrono::prelude::{DateTime, Local};

lazy_static! {
    pub(crate) static ref USER: String = String::from("user");
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct User {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) phone: String,
    pub(crate) active: bool,
    pub(crate) create_at: DateTime<Local>,
    pub(crate) update_at: Option<DateTime<Local>>,
}

impl From<Register> for User {
    fn from(r: Register) -> User {
        let now = Local::now().into();
        let password = hash_password(&r.password);
        User {
            username: r.username,
            email: r.email,
            phone: r.phone,
            password,
            active: false,
            create_at: now,
            update_at: None,
        }
    }
}
