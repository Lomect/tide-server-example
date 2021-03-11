use mongodb::bson::{doc, from_document, oid::ObjectId};
use redis::AsyncCommands;
use tide::{log, prelude::*, Request};
use validator::Validate;

use super::schema::{Login, Register, Resend, ResetPwd};
use crate::db::Options;
use crate::middleware::Token;
use crate::models::{User, USER};
use crate::utils::{hash_password, password_verify, send_email, status, Responser};
use crate::{State, CONFIG};

pub(crate) async fn login(mut req: Request<State>) -> tide::Result {
    let req_data: Login = req.body_json().await?;
    if let Err(e) = req_data.validate() {
        return Responser::new(Some(e), &status::BAD_REQUEST).to_result();
    }

    let mongo_col = &req.state().mongo;
    let redis_cli = req.state().redis.clone();
    let mut redis_con = redis_cli.connection().await?;
    // 查询帐号
    let mut opt = Options::default();
    opt.find_one_opt(&USER, Some(doc! { "email": req_data.email }));

    let mut user_doc = mongo_col.find(opt).await?;
    let data = match user_doc.pop() {
        Some(d) => d,
        None => return Responser::new(Some("帐号不存在"), &status::BAD_REQUEST).to_result(),
    };

    let id = data.get_object_id("_id")?.to_hex();
    let user: User = from_document(data)?;

    // 密码检测
    return if password_verify(&user.password, &req_data.password) {
        // Session 生成
        let token_str: String = redis_con.get(&id).await?;
        if !token_str.is_empty() {
            // session时间小于1小时重新生成
            let seconds: usize = redis_con.ttl(&id).await?;
            if seconds < 3600 {
                redis_con.del(&id).await?;
                redis_con.del(&token_str).await?;
                let token = redis_cli.set_token(id).await?;
                return Responser::new(Some(token), &status::OK).to_result();
            }
            let token = Token { token: token_str };
            Responser::new(Some(token), &status::OK).to_result()
        } else {
            let token = redis_cli.set_token(id).await?;
            Responser::new(Some(token), &status::OK).to_result()
        }
    } else {
        Responser::new(Some(""), &status::UNAUTH).to_result()
    };
}

pub(crate) async fn register(mut req: Request<State>) -> tide::Result {
    let reg = req.body_json::<Register>().await?;
    if let Err(e) = reg.validate() {
        return Responser::new(Some(e), &status::BAD_REQUEST).to_result();
    }
    let mongo_col = &req.state().mongo;
    let redis_cli = req.state().redis.clone();

    // 判断用户存在
    let email = reg.email.clone();
    let exist_user = doc! { "email": email.clone() };
    let mut opt = Options::default();
    opt.find_one_opt(&USER, Some(exist_user));
    let exit = mongo_col.find(opt).await?;
    if !exit.is_empty() {
        return Responser::new(Some("帐号已注册"), &status::BAD_REQUEST).to_result();
    }

    // 写入数据库
    let user = User::from(reg);
    let mut opt = Options::default();
    opt.set_collect(&USER);
    let id = mongo_col.insert_one(opt, &user).await?;

    let token = redis_cli.set_token(id).await?;
    send_email(
        &email,
        "Register API TEST Email",
        &format!(
            "{}/api/v1/auth/confirm/{}",
            CONFIG.server.domain, token.token
        ),
    )
    .await?;

    Responser::new(Some("success"), &status::OK).to_result()
}

pub(crate) async fn confirm(mut req: Request<State>) -> tide::Result {
    let token: Token = req.body_json().await?;
    let redis_cli = req.state().redis.clone();
    let mut redis_con = redis_cli.connection().await?;
    let mongo_col = &req.state().mongo;

    let id_str: String = redis_con.get(&token.token).await?;
    let filter = doc! { "_id": ObjectId::with_string(&id_str)? };
    let data = doc! { "$set": { "active": true } };
    let mut opt = Options::default();

    redis_con.del(&token.token).await?;
    redis_con.del(&id_str).await?;

    opt.update_opt(&USER, Some(filter), None);
    mongo_col.update(data, opt).await?;

    let token = redis_cli.set_token(id_str).await?;

    Responser::new(Some(token), &status::OK).to_result()
}

pub(crate) async fn resend(mut req: Request<State>) -> tide::Result {
    let data: Resend = req.body_json().await?;
    let redis_cli = &req.state().redis;
    let mut redis_con = redis_cli.connection().await?;
    let mongo_col = &req.state().mongo;

    let email = data.email.clone();
    let filter = doc! { "email": &email };
    let mut opt = Options::default();
    opt.find_one_opt(&USER, Some(filter));

    let user_doc = mongo_col.find(opt).await?;
    if user_doc.is_empty() {
        return Responser::new(Some("帐号不存在"), &status::BAD_REQUEST).to_result();
    }
    let data = user_doc[0].clone();
    let id = data.get_object_id("_id")?.to_hex();

    let token_str: String = redis_con.get(&id).await?;

    if !token_str.is_empty() {
        // session时间小于1小时重新生成
        let seeds: usize = redis_con.ttl(&id).await?;
        if seeds > 3600 {
            return Responser::new(Some("请勿重复发送"), &status::BAD_REQUEST).to_result();
        }
        redis_con.del(&id).await?;
        redis_con.del(&token_str).await?;
        let token = redis_cli.set_token(id.clone()).await?;
        send_email(
            &email,
            "Resend API TEST Email",
            &format!(
                "{}/api/v1/auth/confirm/{}",
                CONFIG.server.domain, token.token
            ),
        )
        .await?;
        return Responser::new(Some(token), &status::OK).to_result();
    }
    let token = redis_cli.set_token(id).await?;
    send_email(
        &email,
        "Resend API TEST Email",
        &format!(
            "{}/api/v1/auth/confirm/{}",
            CONFIG.server.domain, token.token
        ),
    )
    .await?;
    Responser::new(Some(token), &status::OK).to_result()
}

pub async fn reset_pwd(mut req: Request<State>) -> tide::Result {
    let pwd_data: ResetPwd = req.body_json().await?;
    if let Err(e) = pwd_data.validate() {
        return Responser::new(Some(e), &status::BAD_REQUEST).to_result();
    }
    let token = match req.ext::<Token>() {
        None => {
            return Responser::new(Some("请先登录，或者发送验证邮件！"), &status::UNAUTH)
                .to_result()
        }
        Some(t) => t,
    };

    let redis_cli = &req.state().redis;
    let mut redis_con = redis_cli.connection().await?;
    let mongo_col = &req.state().mongo;
    let id: String = redis_con.get(&token.token).await?;

    let mut opt = Options::default();
    let filter = doc! { "_id": ObjectId::with_string(&id)?};
    opt.update_opt(&USER, Some(filter), None);
    let password = hash_password(&pwd_data.password);
    let data = doc! { "$set": { "password": password }};
    let _ = mongo_col.update(data, opt).await?;
    return Responser::new(Some("密码修改成功！"), &status::OK).to_result();
}
