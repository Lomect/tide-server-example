use std::collections::HashMap;

use async_std::stream::StreamExt;
use mongodb::bson::{from_bson, oid::ObjectId, to_document, Bson, Document};
use mongodb::options::FindOptions;
use mongodb::{Client, Database};
use serde::Serialize;
use tide::{log, StatusCode};

#[derive(Clone)]
pub struct MongoDb {
    db: Database,
}

impl MongoDb {
    pub(crate) async fn new(uri: &str, database_name: &str) -> tide::Result<Self> {
        let client = match Client::with_uri_str(uri).await {
            Ok(cli) => cli,
            Err(e) => {
                log::error!("Connect Mongo DB Fail Error: {:?}", e);
                return Err(tide::Error::new(StatusCode::InternalServerError, e));
            }
        };
        log::info!("Connect Mongo DB");
        Ok(MongoDb {
            db: client.database(database_name),
        })
    }

    pub async fn find(&self, opt: Options) -> tide::Result<Vec<Document>> {
        let options = FindOptions::builder()
            .limit(opt.limit)
            .sort(opt.sort)
            .skip(opt.skip)
            .projection(opt.fileds.clone())
            .build();
        let mut cur = self
            .db
            .collection(&opt.collect)
            .find(opt.filter.clone(), options)
            .await?;
        let mut res = Vec::new();
        while let Some(docmument_res) = cur.next().await {
            match docmument_res {
                Ok(docmument) => res.push(docmument),
                Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e)),
            }
        }
        Ok(res.to_owned())
    }

    pub async fn insert_one<T: Serialize>(&self, opt: Options, data: &T) -> tide::Result<String> {
        let bs = to_document(data)?;
        match self.db.collection(&opt.collect).insert_one(bs, None).await {
            Ok(s) => match from_bson::<ObjectId>(s.inserted_id.clone()) {
                Ok(id) => Ok(id.to_hex()),
                Err(e) => {
                    log::error!("插入单条数据获取id出错ID: {:?}", s.inserted_id);
                    Err(tide::Error::new(StatusCode::InternalServerError, e))
                }
            },
            Err(e) => {
                log::error!("插入单条数据错误 {:?}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }

    pub async fn insert_many<T: Serialize>(
        &self,
        opt: Options,
        data: &[T],
    ) -> tide::Result<Vec<String>> {
        let bs = data
            .iter()
            .map(|d| to_document(d).unwrap_or_else(|_| Document::new()))
            .collect::<Vec<Document>>();
        match self.db.collection(&opt.collect).insert_many(bs, None).await {
            Ok(s) => {
                let mut res: Vec<String> = Vec::new();
                let ids: HashMap<usize, Bson> = s.inserted_ids.to_owned();
                for v in ids.values() {
                    match v.as_object_id() {
                        Some(id) => res.push(id.to_hex()),
                        None => log::error!("插入数据获取id出错ID为空"),
                    }
                }
                Ok(res)
            }
            Err(e) => {
                log::error!("插入单条数据错误 {:?}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }

    pub async fn delete(&self, opt: Options) -> tide::Result<i64> {
        let filter = opt.filter.unwrap_or(Document::new());
        let res = match opt.limit {
            Some(_) => {
                self.db
                    .collection(&opt.collect)
                    .delete_many(filter, None)
                    .await
            }
            None => {
                self.db
                    .collection(&opt.collect)
                    .delete_one(filter, None)
                    .await
            }
        };

        match res {
            Ok(s) => Ok(s.deleted_count),
            Err(e) => {
                log::error!("删除数据错误 {:?}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }

    pub async fn update(&self, data: Document, opt: Options) -> tide::Result<i64> {
        let filter = opt.filter.unwrap_or(Document::new());
        let res = match opt.limit {
            Some(_) => {
                self.db
                    .collection(&opt.collect)
                    .update_many(filter, data, None)
                    .await
            }
            None => {
                self.db
                    .collection(&opt.collect)
                    .update_one(filter, data, None)
                    .await
            }
        };
        match res {
            Ok(s) => Ok(s.modified_count),
            Err(e) => {
                log::error!("更新数据错误 {:?}", e);
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct Options {
    pub(crate) collect: String,
    pub(crate) filter: Option<Document>,
    pub(crate) limit: Option<i64>,
    pub(crate) skip: Option<i64>,
    pub(crate) sort: Option<Document>,
    pub(crate) fileds: Option<Document>,
}

impl Options {
    pub fn new(
        collect: &str,
        filter: Option<Document>,
        limit: Option<i64>,
        skip: Option<i64>,
        sort: Option<Document>,
        fileds: Option<Document>,
    ) -> Self {
        Self {
            collect: collect.to_string(),
            filter,
            limit,
            skip,
            sort,
            fileds,
        }
    }

    pub fn set_collect(&mut self, collect: &str) {
        self.collect = collect.to_string();
    }
    pub fn del_opt(&mut self, collect: &str, filter: Option<Document>, limit: Option<i64>) {
        self.collect = collect.to_string();
        self.filter = filter;
        self.limit = limit;
    }

    pub fn update_opt(&mut self, collect: &str, filter: Option<Document>, limit: Option<i64>) {
        self.collect = collect.to_string();
        self.filter = filter;
        self.limit = limit;
    }

    pub fn find_one_opt(&mut self, collect: &str, filter: Option<Document>) {
        self.collect = collect.to_string();
        self.filter = filter;
        self.limit = Some(1);
    }
}

#[cfg(test)]
mod tests {
    use super::MongoDb;
    use crate::db::Options;
    use crate::models::User;
    use crate::CONFIG;
    use chrono::Local;
    use mongodb::bson::{doc, oid::ObjectId};

    #[async_std::test]
    async fn test_insert_mongo() {
        let now = Local::now().into();
        let user = User {
            email: "1@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let conn = MongoDb::new(&CONFIG.database.mongo_url, "test")
            .await
            .unwrap();

        let mut opt = Options::default();
        opt.set_collect("test1");

        let id = conn.insert_one(opt, &user).await;
        assert!(id.is_ok());
        conn.db.collection("test1").drop(None).await.unwrap();
    }

    #[async_std::test]
    async fn test_find_one_mongo() {
        let now = Local::now().into();
        let user = User {
            email: "2@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let user1 = User {
            email: "3@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let conn = MongoDb::new(&CONFIG.database.mongo_url, "test")
            .await
            .unwrap();

        let mut opt = Options::default();
        opt.set_collect("test2");

        let users = vec![user, user1];
        let id = conn.insert_many(opt.clone(), &users).await;
        let filter = doc! { "email": "2@qq.com" };

        opt.find_one_opt("test2", filter);
        if let Ok(res) = conn.find(opt.clone()).await {
            assert_eq!(res.len(), 1);
            let one = &res[0];
            if let Ok(find_id) = one.get_object_id("_id") {
                assert!(id.is_ok());
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
        conn.db.collection("test2").drop(None).await.unwrap();
    }

    #[async_std::test]
    async fn test_find_many_mongo() {
        let now = Local::now().into();
        let user = User {
            email: "2@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };
        let user1 = User {
            email: "3@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let conn = MongoDb::new(&CONFIG.database.mongo_url, "test")
            .await
            .unwrap();

        let mut opt = Options::default();
        opt.set_collect("test3");
        let users: Vec<User> = vec![user, user1];
        let id = conn.insert_many(opt.clone(), &users).await;

        let filter = doc! { "username": "lomect" };
        opt.find_one_opt("test3", filter);
        opt.limit = None;

        if let Ok(res) = conn.find(opt).await {
            assert_eq!(res.len(), 2);
        } else {
            assert!(false);
        }
        conn.db.collection("test3").drop(None).await.unwrap();
    }

    #[async_std::test]
    async fn test_delete_one_mongo() {
        let now = Local::now().into();
        let user = User {
            email: "3@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let conn = MongoDb::new(&CONFIG.database.mongo_url, "test")
            .await
            .unwrap();
        let mut opt = Options::default();
        opt.set_collect("test4");

        conn.insert_one(opt.clone(), &user).await;
        let filter = doc! { "email": "3@qq.com" };

        opt.del_opt("test4", filter, None);
        let delete_id = conn.delete(opt).await;
        assert!(delete_id.is_ok());
        conn.db.collection("test4").drop(None).await.unwrap();
    }

    #[async_std::test]
    async fn test_delete_many_mongo() {
        let now = Local::now().into();
        let user = User {
            email: "3@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let user1 = User {
            email: "4@qq.com".to_string(),
            username: String::from("lomect"),
            password: String::from("123456"),
            phone: String::from("157"),
            active: false,
            create_at: now,
            update_at: Some(now),
        };

        let conn = MongoDb::new(&CONFIG.database.mongo_url, "test")
            .await
            .unwrap();

        let mut opt = Options::default();
        opt.set_collect("test5");

        let users = vec![user, user1];
        conn.insert_many(opt.clone(), &users).await;
        let filter = doc! { "username": "lomect" };

        opt.del_opt("test5", filter, Some(1));
        let delete_count = conn.delete(opt).await;

        if delete_count.is_ok() {
            assert_eq!(delete_count.ok(), Some(2i64));
        } else {
            assert!(false);
        }
        conn.db.collection("test5").drop(None).await.unwrap();
    }
}
