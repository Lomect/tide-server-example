use argon2::{self, Config, ThreadMode, Variant, Version};
use base64::encode;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

lazy_static! {
    static ref CRYPTO_MEM: u32 = 4096;
    static ref CRYPTO_TIME: u32 = 10;
    static ref CRYPTO_LANES: u32 = 4;
    static ref CRYPTO_LEN: u32 = 32;
    static ref SALT_LEN: usize = 12;
}

pub fn rand_str(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn hash_password(password: &str) -> String {
    let salt = rand_str(*SALT_LEN);
    let pwd_byte = password.as_bytes();
    let salt_byte = salt.as_bytes();
    let config = Config {
        ad: &[],
        hash_length: *CRYPTO_LEN,
        lanes: *CRYPTO_LANES,
        mem_cost: *CRYPTO_MEM,
        secret: &[],
        thread_mode: ThreadMode::Parallel,
        time_cost: *CRYPTO_TIME,
        variant: Variant::Argon2id,
        version: Version::Version13,
    };
    let hash = argon2::hash_encoded(pwd_byte, salt_byte, &config).expect("Password Hash Error");
    let hash = hash.split("$").last().expect("Password Not get Hash Value");
    format!("{}:{}", salt, hash)
}

pub fn password_verify(password_hash: &str, password: &str) -> bool {
    let pwd_byte = password.as_bytes();
    let hash_pwd = password_hash.split(':').collect::<Vec<&str>>();
    let salt = match hash_pwd.get(0) {
        None => return false,
        Some(s) => {
            if s.len() != *SALT_LEN {
                return false;
            } else {
                encode(s).replace("==", "")
            }
        }
    };
    let hash = match hash_pwd.get(1) {
        None => return false,
        Some(s) => s,
    };
    let pwd_hash = format!(
        "${}$v={}$m={},t={},p={}${}${}",
        Variant::Argon2id.as_lowercase_str(),
        Version::Version13.as_u32(),
        *CRYPTO_MEM,
        *CRYPTO_TIME,
        *CRYPTO_LANES,
        salt,
        hash
    );
    argon2::verify_encoded(&pwd_hash, pwd_byte).unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::{hash_password, password_verify};

    #[test]
    fn test_password() {
        let pwd_hash = hash_password("123456");
        assert_eq!(password_verify(&pwd_hash, "123456"), true);
    }
}
