mod login_new;
pub mod persist_session;

use crate::login::login_new::login_new;
use matrix_sdk::{self, Client};
use std::path::{Path, PathBuf};
use tokio::fs;

use self::persist_session::{restore_session, FullSession};

/// Restoring a session with encryption without having a persisted store
/// will break the encryption setup and the client will not be able to send or
/// receive encrypted messages, hence the need to persist the session.
///
/// To reset the login, simply delete the folder containing the session
/// file, the location is shown in the logs. Note that the database must be
/// deleted too as it can't be reused.
pub async fn login() -> anyhow::Result<(Client, Option<String>, PathBuf)> {
    // info_popup(Type::Informaton, "Informaton", "body")?;
    // info_popup(Type::Error, "Error", "Now iagine the body is very big and doesnt fit into one Line. I really wonder whats gonna happen then since I only have percentages inputted and thus am not able to ")?;

    // The folder containing this example's data.
    let data_dir = dirs::data_dir()
        .expect("no data_dir directory found")
        .join("persist_session");
    // The file where the session is persisted.
    let session_file = data_dir.join("session");

    let (client, sync_token) = if session_file.exists() {
        restore_session(&session_file).await?
    } else {
        (login_new(&data_dir, &session_file).await?, None)
    };

    Ok((client, sync_token, session_file))
}
