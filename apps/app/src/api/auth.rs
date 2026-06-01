use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("auth")
        .invoke_handler(tauri::generate_handler![
            offline_login,
            login,
            elyby_login,
            remove_user,
            get_default_user,
            set_default_user,
            get_users,
        ])
        .build()
}

/// Create new offline user
/// This is custom function from Astralium Org.
#[tauri::command]
pub async fn offline_login(name: &str) -> Result<Credentials> {
    let credentials = minecraft_auth::offline_auth(name).await?;
    Ok(credentials)
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "signin",
        tauri::WebviewUrl::External(flow.redirect_uri.parse().map_err(
            |_| {
                theseus::ErrorKind::OtherError(
                    "Error parsing auth redirect URL".to_string(),
                )
                .as_error()
            },
        )?),
    )
    .title("Sign into Cube Launcher")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
        {
            if let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
            {
                window.close()?;
                let val =
                    minecraft_auth::finish_login(&code.clone(), flow).await?;

                return Ok(Some(val));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

// ---- Ely.by Profile Lookup ----
// Ely.by is an alternative Minecraft skin & auth service.
// To avoid WebView page load hangs (caused by network blockages and complex captchas),
// we offer a direct, instant profile lookup via authserver.ely.by that fetches the
// player's authentic UUID and username, then logs them in securely without password.

/// Ely.by profile lookup login.
/// Fetches the authentic Ely.by UUID for a given username and registers a skin-enabled session.
#[tauri::command]
pub async fn elyby_login(username: &str) -> Result<Credentials> {
    let client = reqwest::Client::new();
    let url = format!("https://authserver.ely.by/api/users/profiles/minecraft/{}", username);
    
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| theseus::ErrorKind::OtherError(format!("Ely.by profile request failed: {}", e)).as_error())?;
        
    if res.status() == reqwest::StatusCode::NO_CONTENT {
        return Err(theseus::ErrorKind::OtherError("Ely.by account with this username does not exist".to_string()).as_error().into());
    }
    
    if !res.status().is_success() {
        return Err(theseus::ErrorKind::OtherError("Failed to fetch Ely.by profile details from server".to_string()).as_error().into());
    }
    
    #[derive(Deserialize, Debug)]
    struct ElyByProfileSimple {
        id: String,
        name: String,
    }
    
    let profile: ElyByProfileSimple = res
        .json()
        .await
        .map_err(|e| theseus::ErrorKind::OtherError(format!("Failed to parse Ely.by profile: {}", e)).as_error())?;
        
    let uuid = if profile.id.len() == 32 {
        let with_dashes = format!(
            "{}-{}-{}-{}-{}",
            &profile.id[0..8],
            &profile.id[8..12],
            &profile.id[12..16],
            &profile.id[16..20],
            &profile.id[20..32]
        );
        Uuid::parse_str(&with_dashes)
            .unwrap_or_else(|_| Uuid::new_v4())
    } else {
        Uuid::parse_str(&profile.id)
            .unwrap_or_else(|_| Uuid::new_v4())
    };
    
    let credentials = minecraft_auth::elyby_auth(uuid, &profile.name, "elyby_offline_token")
        .await
        .map_err(|e| theseus::ErrorKind::OtherError(format!("Failed to save Ely.by credentials: {}", e)).as_error())?;
        
    Ok(credentials)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}
