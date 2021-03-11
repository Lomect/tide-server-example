use crate::db::{MongoDb, Redis};
use crate::CONFIG;

#[derive(Clone)]
pub struct State {
    pub mongo: MongoDb,
    pub redis: Redis,
}

impl State {
    pub async fn new() -> tide::Result<Self> {
        let mongc = MongoDb::new(&CONFIG.database.mongo_url, &CONFIG.database.mongo_name).await?;
        let redic = Redis::new(&CONFIG.database.redis_url)?;
        Ok(State {
            mongo: mongc,
            redis: redic,
        })
    }
}
//
// #[async_std::test]
// async fn test_state() {
//     let sta = State::new().await;
//     let con = sta.mongo.conn();
//     for coll_name in con.list_collection_names(None).await {
//         println!("collection: {:?}", coll_name);
//     }
// }
