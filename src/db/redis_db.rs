use redis::{aio::Connection, AsyncCommands, Client};
use tide::{log, StatusCode};

use crate::middleware::Token;
use crate::utils::rand_str;

lazy_static! {
    pub(crate) static ref TOKEN_SIZE: usize = 32;
    pub(crate) static ref EXPIRE_TIME: usize = 60 * 60 * 12;
}

#[derive(Clone)]
pub struct Redis {
    pub cli: Client,
}

impl Redis {
    pub async fn connection(&self) -> tide::Result<Connection> {
        match self.cli.get_async_connection().await {
            Ok(con) => {
                Ok(con)
            }
            Err(e) => {
                log::error!("Get redis Connection Fail Error: {}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }

    pub fn new(uri: &str) -> tide::Result<Self> {
        match Client::open(uri) {
            Ok(cli) => Ok(Self { cli }),
            Err(e) => {
                log::error!("Open redis Fail Error: {}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }

    pub async fn set_token(&self, id: String) -> tide::Result<Token> {
        let token_str = rand_str(*TOKEN_SIZE);
        let mut redis_col = self.connection().await?;
        redis_col.set_ex(&token_str, &id, *EXPIRE_TIME).await?;
        redis_col.set_ex(&id, &token_str, *EXPIRE_TIME).await?;
        Ok(Token { token: token_str })
    }
}
