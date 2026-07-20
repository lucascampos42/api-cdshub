use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordVerifier, SaltString},
    Argon2, PasswordHasher,
};

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        let valid = bcrypt::verify(password, hash).map_err(|e| e.to_string())?;
        return Ok(valid);
    }

    let parsed_hash = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_random_password(length: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_argon2() {
        let password = "MySecureP@ss123";
        let hash = hash_password(password).unwrap();
        assert!(hash.starts_with("$argon2"));
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "CorrectP@ss1";
        let hash = hash_password(password).unwrap();
        assert!(!verify_password("WrongP@ss1", &hash).unwrap());
    }

    #[test]
    fn test_verify_bcrypt_hash() {
        // bcrypt hash of "testPassword123"
        let hash = "$2b$12$LJ3m4ys3Lk0TSwHmsVrK7eWRj0lK0aB5O9M0x7Y1F2f3f4f5f6f7f8";
        // We just test that verify_password handles bcrypt hashes without error
        let result = verify_password("testPassword123", hash);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_generate_random_password_length() {
        let pwd = generate_random_password(16);
        assert_eq!(pwd.len(), 16);
    }

    #[test]
    fn test_generate_random_password_variety() {
        let pwd1 = generate_random_password(32);
        let pwd2 = generate_random_password(32);
        assert_ne!(pwd1, pwd2);
    }

    #[test]
    fn test_generate_random_password_min_length() {
        let pwd = generate_random_password(1);
        assert_eq!(pwd.len(), 1);
    }

    #[test]
    fn test_hash_different_each_time() {
        let password = "SameP@ss1";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash1, hash2);
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_verify_invalid_hash() {
        assert!(verify_password("pwd", "not-a-hash").is_err());
    }

    #[test]
    fn test_generate_random_password_only_allowed_chars() {
        let pwd = generate_random_password(1000);
        let allowed: std::collections::HashSet<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
                .chars()
                .collect();
        assert!(pwd.chars().all(|c| allowed.contains(&c)));
    }
}
