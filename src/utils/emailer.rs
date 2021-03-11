use lettre::{
    transport::smtp::authentication::Credentials, transport::smtp::AsyncSmtpTransport, AsyncStd1Connector,
    AsyncStd1Transport, Message,
};
use tide::StatusCode;

use super::{status, Responser};
use crate::CONFIG;

pub async fn send_email(to: &str, topic: &str, message: &str) -> tide::Result {
    let email = Message::builder()
        .from(CONFIG.email.email_name.clone().parse().unwrap())
        .to(to.parse().unwrap())
        .subject(topic)
        .body(String::from(message))
        .unwrap();

    let creds = Credentials::new(CONFIG.email.email_name.clone(), CONFIG.email.email_password.clone());

    // Open a remote connection to gmail using STARTTLS
    let mailer = AsyncSmtpTransport::<AsyncStd1Connector>::starttls_relay(&CONFIG.email.email_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => Responser::new(Some("Email Send Success".to_string()), &status::OK).to_result(),
        Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e)),
    }
}

#[cfg(test)]
mod test {
    use crate::utils::Responser;

    #[async_std::test]
    async fn test_send_email() {
        let res = super::send_email("lome@lomect.com", "id", "test send email").await;
        println!("{:?}", res);
        assert!(res.is_ok())
    }
}
