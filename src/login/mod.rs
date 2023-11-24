mod input_popup;
mod login_new;
pub mod persist_session;

use crate::login::login_new::login_new;
use matrix_sdk::{
    self,
    config::SyncSettings,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
    Client, Room, RoomState,
};
use std::path::Path;
use tokio::fs;

use matrix_sdk::{ruma::api::client::filter::FilterDefinition, Error, LoopCtrl};

use self::{
    input_popup::show_popup,
    persist_session::{restore_session, FullSession},
};

/// Restoring a session with encryption without having a persisted store
/// will break the encryption setup and the client will not be able to send or
/// receive encrypted messages, hence the need to persist the session.
///
/// To reset the login, simply delete the folder containing the session
/// file, the location is shown in the logs. Note that the database must be
/// deleted too as it can't be reused.
pub async fn login() -> anyhow::Result<()> {
    // The folder containing this example's data.
    let data_dir = dirs::data_dir()
        .expect("no data_dir directory found")
        .join("persist_session");
    // The file where the session is persisted.
    let session_file = data_dir.join("session");

    let (client, sync_token) = if session_file.exists() {
        restore_session(&session_file).await?
    } else {
        let homeserver_url =
            show_popup("Homeserver URL", "Please Input your homeserver URL here.")?;
        (
            login_new(&data_dir, &session_file, homeserver_url).await?,
            None,
        )
    };

    sync(client, sync_token, &session_file)
        .await
        .map_err(Into::into)
}

/// Setup the client to listen to new messages.
async fn sync(
    client: Client,
    initial_sync_token: Option<String>,
    session_file: &Path,
) -> anyhow::Result<()> {
    println!("Launching a first sync to ignore past messages…");

    // Enable room members lazy-loading, it will speed up the initial sync a lot
    // with accounts in lots of rooms.
    // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
    let filter = FilterDefinition::with_lazy_loading();

    let mut sync_settings = SyncSettings::default().filter(filter.into());

    // We restore the sync where we left.
    // This is not necessary when not using `sync_once`. The other sync methods get
    // the sync token from the store.
    if let Some(sync_token) = initial_sync_token {
        sync_settings = sync_settings.token(sync_token);
    }

    // Let's ignore messages before the program was launched.
    // This is a loop in case the initial sync is longer than our timeout. The
    // server should cache the response and it will ultimately take less time to
    // receive.
    loop {
        match client.sync_once(sync_settings.clone()).await {
            Ok(response) => {
                // This is the last time we need to provide this token, the sync method after
                // will handle it on its own.
                sync_settings = sync_settings.token(response.next_batch.clone());
                persist_sync_token(session_file, response.next_batch).await?;
                break;
            }
            Err(error) => {
                println!("An error occurred during initial sync: {error}");
                println!("Trying again…");
            }
        }
    }

    println!("The client is ready! Listening to new messages…");

    // Now that we've synced, let's attach a handler for incoming room messages.
    client.add_event_handler(on_room_message);

    // This loops until we kill the program or an error happens.
    client
        .sync_with_result_callback(sync_settings, |sync_result| async move {
            let response = sync_result?;

            // We persist the token each time to be able to restore our session
            persist_sync_token(session_file, response.next_batch)
                .await
                .map_err(|err| Error::UnknownError(err.into()))?;

            Ok(LoopCtrl::Continue)
        })
        .await?;

    Ok(())
}

/// Persist the sync token for a future session.
/// Note that this is needed only when using `sync_once`. Other sync methods get
/// the sync token from the store.
async fn persist_sync_token(session_file: &Path, sync_token: String) -> anyhow::Result<()> {
    let serialized_session = fs::read_to_string(session_file).await?;
    let mut full_session: FullSession = serde_json::from_str(&serialized_session)?;

    full_session.sync_token = Some(sync_token);
    let serialized_session = serde_json::to_string(&full_session)?;
    fs::write(session_file, serialized_session).await?;

    Ok(())
}

/// Handle room messages.
async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    // We only want to log text messages in joined rooms.
    if room.state() != RoomState::Joined {
        return;
    }
    let MessageType::Text(text_content) = &event.content.msgtype else { return };

    let room_name = match room.display_name().await {
        Ok(room_name) => room_name.to_string(),
        Err(error) => {
            println!("Error getting room display name: {error}");
            // Let's fallback to the room ID.
            room.room_id().to_string()
        }
    };

    println!("[{room_name}] {}: {}", event.sender, text_content.body)
}
