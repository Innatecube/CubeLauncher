//! Authentication flow interface

use crate::state::{Credentials, MinecraftLoginFlow};
use crate::State;

#[tracing::instrument]
pub async fn begin_login() -> crate::Result<MinecraftLoginFlow> {
    let state = State::get().await?;

    crate::state::login_begin(&state.pool).await
}

#[tracing::instrument]
pub async fn finish_login(
    code: &str,
    flow: MinecraftLoginFlow,
) -> crate::Result<Credentials> {
    let state = State::get().await?;

    crate::state::login_finish(code, flow, &state.pool).await
}

#[tracing::instrument]
pub async fn offline_auth(
    name: &str
) -> crate::Result<Credentials> {
    let state = State::get().await?;
    crate::state::offline_auth(name, &state.pool).await
}

/// Store Ely.by account credentials retrieved via OAuth2.
/// uuid: the player's UUID from Ely.by profile
/// username: player's in-game name
/// access_token: Ely.by access token
#[tracing::instrument]
pub async fn elyby_auth(
    uuid: uuid::Uuid,
    username: &str,
    access_token: &str,
) -> crate::Result<Credentials> {
    use chrono::{Duration, Utc};

    let state = State::get().await?;

    let credentials = Credentials {
        id: uuid,
        username: username.to_string(),
        access_token: access_token.to_string(),
        refresh_token: access_token.to_string(),
        expires: Utc::now() + Duration::days(30),
        active: true,
    };

    credentials.upsert(&state.pool).await?;
    Ok(credentials)
}

#[tracing::instrument]
pub async fn get_default_user() -> crate::Result<Option<uuid::Uuid>> {
    let state = State::get().await?;
    let users = Credentials::get_active(&state.pool).await?;
    Ok(users.map(|x| x.id))
}

#[tracing::instrument]
pub async fn set_default_user(user: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    let (_, mut user) = users.remove(&user).ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get nonexistent user with ID {user}"
        ))
        .as_error()
    })?;

    user.active = true;
    user.upsert(&state.pool).await?;

    Ok(())
}

/// Remove a user account from the database
#[tracing::instrument]
pub async fn remove_user(uuid: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;

    let users = Credentials::get_all(&state.pool).await?;

    if let Some((uuid, user)) = users.remove(&uuid) {
        Credentials::remove(uuid, &state.pool).await?;

        if user.active {
            if let Some((_, mut user)) = users.into_iter().next() {
                user.active = true;
                user.upsert(&state.pool).await?;
            }
        }
    }

    Ok(())
}

/// Get a copy of the list of all user credentials
#[tracing::instrument]
pub async fn users() -> crate::Result<Vec<Credentials>> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    Ok(users.into_iter().map(|x| x.1).collect())
}
