// compatible with sha256 random salt hashing of C# .NET
pub mod hasher {
    use rand::RngCore;
    use sha2::{Sha256, Digest};
    use base64::{Engine as _, engine::{general_purpose}};


    pub fn hash_password_with_sha256_with_salt(password: &str) -> String {
        let salt = generate_salt();
        let salted_password_bytes = generate_salted_password(password, &salt);
    
        let mut sha256 = Sha256::new();
        sha256.update(&salted_password_bytes);
    
        let hash_bytes = sha256.finalize();
        let salted_hash_bytes = generate_salted_hash(&salt, &hash_bytes);
    
        println!("SHA256 Salt generated: {}", general_purpose::STANDARD.encode(&salt));
        general_purpose::STANDARD.encode(&salted_hash_bytes)
    }
    
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }
    
    pub fn generate_salted_password(password: &str, salt: &[u8]) -> Vec<u8> {
        let password_bytes = password.as_bytes().to_vec();
        let mut salted_password_bytes = Vec::with_capacity(password_bytes.len() + salt.len());
    
        salted_password_bytes.extend_from_slice(&password_bytes);
        salted_password_bytes.extend_from_slice(salt);
    
        salted_password_bytes
    }
    
    pub fn generate_salted_hash(salt: &[u8], hash_bytes: &[u8]) -> Vec<u8> {
        let mut salted_hash_bytes = Vec::with_capacity(salt.len() + hash_bytes.len());
    
        salted_hash_bytes.extend_from_slice(salt);
        salted_hash_bytes.extend_from_slice(hash_bytes);
    
        salted_hash_bytes
    }
    
    pub fn verify_password_with_sha256_with_salt(password: &str, hashed_password: &str) -> bool {
        let salted_hash_bytes =  general_purpose::STANDARD.decode(hashed_password).unwrap();
        let salt = &salted_hash_bytes[..16];
    
        let salted_password_bytes = generate_salted_password(password, salt);
    
        let mut sha256 = Sha256::new();
        sha256.update(&salted_password_bytes);
    
        let hash_to_check = sha256.finalize();
        let salted_hash_to_check_bytes = generate_salted_hash(salt, &hash_to_check);
    
        salted_hash_bytes == salted_hash_to_check_bytes
    }
}