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

pub struct SyncPayloads {
    pub settings: Option<Vec<u8>>,
    pub english_dict: Option<Vec<u8>>,
    pub macros: Option<Vec<u8>>,
    pub clipboard: Option<Vec<u8>>,
    pub app_configs: Option<Vec<u8>>,
}

fn encrypt_json<T: Serialize>(data: &T, sync_password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let json_data = serde_json::to_string(data)?;
    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(sync_password.as_bytes(), b"vnkey_cloud_salt", 100_000, &mut key);
    let encrypted = encrypt_text(&json_data, &key);
    Ok(encrypted.into_bytes())
}

fn decrypt_json<T: serde::de::DeserializeOwned>(encrypted_bytes: &[u8], sync_password: &str) -> Result<T, Box<dyn Error>> {
    let encrypted_payload = String::from_utf8(encrypted_bytes.to_vec())?;
    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(sync_password.as_bytes(), b"vnkey_cloud_salt", 100_000, &mut key);
    let dec_json = decrypt_text(&encrypted_payload, &key);
    let data: T = serde_json::from_str(&dec_json)?;
    Ok(data)
}

pub fn get_sync_payloads(sync_password: &str) -> Result<SyncPayloads, Box<dyn Error>> {
    let mut payloads = SyncPayloads {
        settings: None,
        english_dict: None,
        macros: None,
        clipboard: None,
        app_configs: None,
    };

    let sync_settings = db::db_get_kv("syncSettings").unwrap_or_else(|| "1".to_string()) == "1";
    let sync_english_dict = db::db_get_kv("syncEnglishDict").unwrap_or_else(|| "1".to_string()) == "1";
    let sync_macros = db::db_get_kv("syncMacros").unwrap_or_else(|| "1".to_string()) == "1";
    let sync_clipboard = db::db_get_kv("syncClipboard").unwrap_or_else(|| "1".to_string()) == "1";
    let sync_app_configs = db::db_get_kv("syncAppConfigs").unwrap_or_else(|| "1".to_string()) == "1";

    if sync_english_dict {
        payloads.english_dict = Some(encrypt_json(&db::db_get_english_words(), sync_password)?);
    }
    if sync_macros {
        payloads.macros = Some(encrypt_json(&db::db_get_macros(), sync_password)?);
    }
    if sync_clipboard {
        payloads.clipboard = Some(encrypt_json(&db::db_get_clipboard_items(), sync_password)?);
    }

    if let Some(handle) = crate::get_app_handle() {
        if sync_settings {
            if let Some(path) = crate::get_settings_path(&handle) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        payloads.settings = Some(encrypt_json(&json, sync_password)?);
                    }
                }
            }
        }
        if sync_app_configs {
            if let Some(path) = crate::get_app_settings_path(&handle) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        payloads.app_configs = Some(encrypt_json(&json, sync_password)?);
                    }
                }
            }
        }
    }

    Ok(payloads)
}

pub fn apply_sync_payloads(
    settings: Option<&[u8]>,
    english_dict: Option<&[u8]>,
    macros: Option<&[u8]>,
    clipboard: Option<&[u8]>,
    app_configs: Option<&[u8]>,
    sync_password: &str,
) -> Result<(), Box<dyn Error>> {
    
    if let Some(bytes) = english_dict {
        if let Ok(words) = decrypt_json::<Vec<String>>(bytes, sync_password) {
            db::db_insert_english_words(&words);
            crate::engine::set_custom_english_words(&db::db_get_english_words().join("\n"));
        }
    }
    if let Some(bytes) = macros {
        if let Ok(m) = decrypt_json::<Vec<(String, String)>>(bytes, sync_password) {
            db::db_insert_macros(&m);
            for (shortcut, content) in m {
                crate::engine::add_macro(&shortcut, &content);
            }
        }
    }
    if let Some(bytes) = clipboard {
        if let Ok(items) = decrypt_json::<Vec<crate::ClipboardItem>>(bytes, sync_password) {
            db::db_insert_clipboard_items(&items);
            let items_db = db::db_get_clipboard_items();
            if let Some(history_mutex) = crate::CLIPBOARD_HISTORY.get() {
                let mut history = history_mutex.lock().unwrap();
                *history = items_db;
            } else {
                let _ = crate::CLIPBOARD_HISTORY.set(std::sync::Mutex::new(items_db));
            }
        }
    }
    
    if let Some(handle) = crate::get_app_handle() {
        if let Some(bytes) = settings {
            if let Ok(json) = decrypt_json::<serde_json::Value>(bytes, sync_password) {
                if let Some(path) = crate::get_settings_path(&handle) {
                    if let Ok(content) = serde_json::to_string_pretty(&json) {
                        let _ = std::fs::write(&path, content);
                    }
                }
            }
        }
        if let Some(bytes) = app_configs {
            if let Ok(json) = decrypt_json::<serde_json::Value>(bytes, sync_password) {
                if let Some(path) = crate::get_app_settings_path(&handle) {
                    if let Ok(content) = serde_json::to_string_pretty(&json) {
                        let _ = std::fs::write(&path, content);
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn upload_sync_data(creds: &CloudCredentials, sync_password: &str) -> Result<(), Box<dyn Error>> {
    let payloads = get_sync_payloads(sync_password)?;

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

    if let Some(data) = payloads.settings {
        bucket.put_object("vnkey_sync_settings.enc", &data).await?;
    }
    if let Some(data) = payloads.english_dict {
        bucket.put_object("vnkey_sync_english_dict.enc", &data).await?;
    }
    if let Some(data) = payloads.macros {
        bucket.put_object("vnkey_sync_macros.enc", &data).await?;
    }
    if let Some(data) = payloads.clipboard {
        bucket.put_object("vnkey_sync_clipboard.enc", &data).await?;
    }
    if let Some(data) = payloads.app_configs {
        bucket.put_object("vnkey_sync_app_configs.enc", &data).await?;
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

    let settings = bucket.get_object("vnkey_sync_settings.enc").await.map(|r| r.bytes().to_vec()).ok();
    let english_dict = bucket.get_object("vnkey_sync_english_dict.enc").await.map(|r| r.bytes().to_vec()).ok();
    let macros = bucket.get_object("vnkey_sync_macros.enc").await.map(|r| r.bytes().to_vec()).ok();
    let clipboard = bucket.get_object("vnkey_sync_clipboard.enc").await.map(|r| r.bytes().to_vec()).ok();
    let app_configs = bucket.get_object("vnkey_sync_app_configs.enc").await.map(|r| r.bytes().to_vec()).ok();

    apply_sync_payloads(
        settings.as_deref(),
        english_dict.as_deref(),
        macros.as_deref(),
        clipboard.as_deref(),
        app_configs.as_deref(),
        sync_password
    )?;
    
    Ok(())
}

