use config::{Config, ConfigError, File};

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub mongo_url: String,
    pub redis_url: String,
    pub mongo_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub server: String,
    pub domain: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Email {
    pub email_name: String,
    pub email_password: String,
    pub email_server: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Setting {
    pub database: Database,
    pub server: Server,
    pub email: Email,
    pub env: String
}


impl Setting {
    pub fn new(name: &str) -> Result<Setting, ConfigError> {
        let name = name.to_uppercase();

        let mut s = Config::default();
        s.set("env", name.clone())?;
        let filename = format!("./config/{}", name);
        s.merge(File::with_name(&filename))?;
        s.try_into()
    }
}

