use uuid::Uuid;

#[derive(GraphQLObject)]
pub struct Message {
    message_id: Uuid,
    user_id: Uuid,
    message: String,
    channel_id: Uuid,
}

impl Message {
    pub fn new(message_id: Uuid, message: String, user_id: Uuid, channel_id: Uuid) -> Self {
        Self {
            message_id,
            user_id,
            message,
            channel_id,
        }
    }
}
