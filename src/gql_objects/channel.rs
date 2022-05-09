use crate::Context;
use crate::Message;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use mysql::prelude::*;
use uuid::Uuid;

pub struct Channel {
    id: Uuid,
    name: String,
}

#[graphql_object(Context=Context)]
impl Channel {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[graphql(description = "Get all the messages that belong to the server")]
    pub fn messages(&self, context: &Context) -> Vec<Message> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        tsx.exec_map(
            "SELECT id, message, user_id, channel_id, timestamp FROM messages WHERE messages.channel_id=? ORDER BY timestamp",
            (&self.id,),
            |(id, message, user_id, channel_id, timestamp): (Uuid, String, Uuid, Uuid, NaiveDateTime)| {
                let timestamp = DateTime::<Utc>::from_utc(timestamp, Utc);
                println!("{}", timestamp);
                return Message::new(id, message, user_id, channel_id, timestamp);
            },
        )
        .unwrap()
    }
}

impl Channel {
    pub fn new(id: Uuid, name: String) -> Self {
        Self { id, name }
    }
}
