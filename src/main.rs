pub mod login;
mod sync;
pub mod ui_elements;

use self::login::login;
use self::sync::sync;

/// A simple program that adapts to the different login methods offered by a
/// Matrix homeserver.
///
/// Homeservers usually offer to login either via password, Single Sign-On (SSO)
/// or both.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt::init();

    let (client, sync_token, session_file) = login().await?;
    sync(client, sync_token, &session_file)
        .await
        .map_err(Into::into)
}
