pub async fn handle(
    cmdline: &Vec<&str>,
    event: &matrix_sdk::events::SyncMessageEvent<matrix_sdk::events::room::message::MessageEventContent>,
    room: &matrix_sdk::room::Joined,
) {
    let content = matrix_sdk::events::AnyMessageEventContent::RoomMessage(matrix_sdk::events::room::message::MessageEventContent::text_plain(
        "Pong 🏓",
    ));

    room.send(content, None).await.unwrap();
}
