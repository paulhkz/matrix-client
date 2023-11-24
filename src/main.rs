mod login;
pub mod ui_elements;

use self::login::login;

/// A simple program that adapts to the different login methods offered by a
/// Matrix homeserver.
///
/// Homeservers usually offer to login either via password, Single Sign-On (SSO)
/// or both.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    login().await?;

    Ok(())
}
