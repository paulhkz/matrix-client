mod login;
mod persist_session;

use self::login::login_and_sync;
use std::{env, process::exit};

/// A simple program that adapts to the different login methods offered by a
/// Matrix homeserver.
///
/// Homeservers usually offer to login either via password, Single Sign-On (SSO)
/// or both.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Some(homeserver_url) = env::args().nth(1) else {
        eprintln!("Usage: {} <homeserver_url>", env::args().next().unwrap());
        exit(1)
    };

    login_and_sync(homeserver_url).await?;

    Ok(())
}
