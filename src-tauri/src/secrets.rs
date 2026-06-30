//! API key storage in the OS keychain (macOS Keychain via the `keyring` crate).
//! Keys never touch `jana.db` and are never returned to the frontend — the UI can
//! only set, clear, or check presence.

use keyring::{Entry, Error as KeyringError};

/// Keychain service name. Matches the app bundle identifier so entries are
/// namespaced to Jana.
const SERVICE: &str = "com.jana.editor";

/// Map a provider id to its keychain account name. Only cloud providers need keys.
fn account_for(provider: &str) -> Result<String, String> {
    match provider {
        "openai" | "anthropic" | "gemini" => Ok(format!("{}_api_key", provider)),
        other => Err(format!("Unknown provider: {}", other)),
    }
}

fn entry(provider: &str) -> Result<Entry, String> {
    let account = account_for(provider)?;
    Entry::new(SERVICE, &account).map_err(|e| e.to_string())
}

/// Read a provider's API key from the keychain. Returns None if unset or on error
/// — callers treat "no key" and "couldn't read" the same way (prompt the user).
pub fn get_api_key(provider: &str) -> Option<String> {
    entry(provider).ok()?.get_password().ok()
}

#[tauri::command]
pub async fn set_api_key(provider: String, key: String) -> Result<(), String> {
    entry(&provider)?
        .set_password(&key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<bool, String> {
    match entry(&provider)?.get_password() {
        Ok(_) => Ok(true),
        Err(KeyringError::NoEntry) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<(), String> {
    match entry(&provider)?.delete_credential() {
        Ok(_) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
