use redis::{AsyncCommands, Client};
use tide::{Middleware, Next, Request};

use crate::db::Redis;
use crate::CONFIG;
use crate::utils::{status, Responser};

pub struct LoginMiddleware {
    conn: Client,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub token: String,
}

impl LoginMiddleware {
    pub async fn new() -> tide::Result<Self> {
        let red = Redis::new(&CONFIG.database.redis_url)?;
        Ok(Self { conn: red.cli })
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LoginMiddleware {
    async fn handle(&self, mut request: Request<State>, next: Next<'_, State>) -> tide::Result {
        match request.header("Authorization") {
            Some(token) => {
                let mut con = self.conn.clone().get_async_connection().await?;
                let token_key = token.as_str();
                let redis_token: String = con.get(token_key).await?;
                let token = Token { token: redis_token };
                request.set_ext(token);
                let res = next.run(request).await;
                Ok(res)
            }
            None => Responser::new(Some(""), &status::UNAUTH).to_result(),
        }
    }
}
