//! # [Ratatui] Tabs example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui-org/ratatui
//! [examples]: https://github.com/ratatui-org/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui-org/ratatui/blob/main/examples/README.md

#![allow(clippy::wildcard_imports, clippy::enum_glob_use)]

use color_eyre::Result;
use matrix_sdk::{
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
    Client, Room, RoomState,
};

pub async fn chat(client: Client) -> Result<()> {
    // Now that we've synced, let's attach a handler for incoming room messages.
    client.add_event_handler(on_room_message);
    let joined_rooms = client.joined_rooms();
    println!(
        "{:?}",
        joined_rooms
            .iter()
            .map(|room| room.name().unwrap_or("".to_owned()))
            .collect::<Vec<_>>()
    );

    let room1 = joined_rooms
        .iter()
        .find(|room| room.name().unwrap_or("".to_owned()) == "room1")
        .unwrap();

    let sth = room1
        .send(RoomMessageEventContent::text_plain("hallo"))
        .await;
    println!("{:?}", sth);
    Ok(())
}

/// Handle room messages.
async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    // tx: Sender<OriginalSyncMessageLikeEvent<RoomMessageEventContent>>,
) {
    // We only want to log text messages in joined rooms.
    if room.state() != RoomState::Joined {
        return;
    }
    let MessageType::Text(text_content) = &event.content.msgtype else {
        return;
    };

    let room_name = match room.display_name().await {
        Ok(room_name) => room_name.to_string(),
        Err(error) => {
            println!("Error getting room display name: {error}");
            // Let's fallback to the room ID.
            room.room_id().to_string()
        }
    };

    println!("[{room_name}] {}: {}", event.sender, text_content.body);
}
