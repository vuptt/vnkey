use serde::{Deserialize, Serialize};
use std::error::Error;
use s3::{Bucket, Region};
use s3::creds::Credentials;
use crate::db::{self, encrypt_text, decrypt_text};

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudCredentials {
    pub account_id: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SyncPayload {
    pub macros: Vec<(String, String)>,
    pub english_words: Vec<String>,
    pub clipboard_items: Vec<crate::ClipboardItem>,
}

pub fn get_sync_payload(sync_password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let payload = SyncPayload {
        macros: db::db_get_macros(),
        english_words: db::db_get_english_words(),
        clipboard_items: db::db_get_clipboard_items(),
    };

    let json_data = serde_json::to_string(&payload)?;
    
    // Encrypt the payload before uploading
    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(sync_password.as_bytes(), b"vnkey_cloud_salt", 100_000, &mut key);
    let encrypted_payload = encrypt_text(&json_data, &key);
    Ok(encrypted_payload.into_bytes())
}

pub async fn upload_sync_data(creds: &CloudCredentials, sync_password: &str) -> Result<(), Box<dyn Error>> {
    let encrypted_payload = get_sync_payload(sync_password)?;

    let region = Region::Custom {
        region: "auto".to_string(),
        endpoint: format!("https://{}.r2.cloudflarestorage.com", creds.account_id),
    };
    
    let credentials = Credentials::new(
        Some(&creds.access_key),
        Some(&creds.secret_key),
        None,
        None,
        None,
    )?;
    
    let bucket = Bucket::new(&creds.bucket_name, region, credentials)?;
    let bucket = bucket.with_path_style();

    bucket.put_object("vnkey_sync_data.enc", &encrypted_payload).await?;

    Ok(())
}

pub fn apply_sync_payload(encrypted_payload_bytes: &[u8], sync_password: &str) -> Result<(), Box<dyn Error>> {
    let encrypted_payload = String::from_utf8(encrypted_payload_bytes.to_vec())?;

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(sync_password.as_bytes(), b"vnkey_cloud_salt", 100_000, &mut key);
    
    let dec_json = decrypt_text(&encrypted_payload, &key);
    let payload: SyncPayload = serde_json::from_str(&dec_json)?;

    // Merge logic
    db::db_insert_macros(&payload.macros);
    db::db_insert_english_words(&payload.english_words);
    db::db_insert_clipboard_items(&payload.clipboard_items);

    // After updating the DB, we need to apply them to the engine
    for (shortcut, content) in payload.macros {
        crate::engine::add_macro(&shortcut, &content);
    }
    
    let all_words = db::db_get_english_words();
    crate::engine::set_custom_english_words(&all_words.join("\n"));
    
    // Clipboard history will be updated on the next UI fetch or we could broadcast an event
    let items = db::db_get_clipboard_items();
    if let Some(history_mutex) = crate::CLIPBOARD_HISTORY.get() {
        let mut history = history_mutex.lock().unwrap();
        *history = items;
    } else {
        let _ = crate::CLIPBOARD_HISTORY.set(std::sync::Mutex::new(items));
    }
    
    Ok(())
}

pub async fn download_sync_data(creds: &CloudCredentials, sync_password: &str) -> Result<(), Box<dyn Error>> {
    let region = Region::Custom {
        region: "auto".to_string(),
        endpoint: format!("https://{}.r2.cloudflarestorage.com", creds.account_id),
    };
    
    let credentials = Credentials::new(
        Some(&creds.access_key),
        Some(&creds.secret_key),
        None,
        None,
        None,
    )?;
    
    let bucket = Bucket::new(&creds.bucket_name, region, credentials)?;
    let bucket = bucket.with_path_style();

    let response = bucket.get_object("vnkey_sync_data.enc").await?;
    apply_sync_payload(response.bytes(), sync_password)?;
    
    Ok(())
}
