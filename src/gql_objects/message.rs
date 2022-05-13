use crate::gql_objects::User;
use crate::Context;
use chrono::{DateTime, Utc};
use mysql::prelude::*;
use uuid::Uuid;

pub struct Message {
    message_id: Uuid,
    user_id: Uuid,
    message: String,
    channel_id: Uuid,
    timestamp: DateTime<Utc>,
}

#[graphql_object(Context=Context)]
impl Message {
    pub fn message_id(&self) -> &Uuid {
        &self.message_id
    }
    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }
    pub fn message(&self) -> &String {
        &self.message
    }
    pub fn channel_id(&self) -> &Uuid {
        &self.channel_id
    }
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    pub fn author(&self, context: &Context) -> User {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let row: (Uuid, String) = tsx
            .exec_first(
                "SELECT id, username FROM users WHERE id=?",
                (&self.user_id,),
            )
            .unwrap()
            .expect(&format!(
                "User with id: {} not found when looking for message author!",
                &self.user_id
            ));

        return User::new(row.0, row.1, None);
    }
}

impl Message {
    pub fn new(
        message_id: Uuid,
        message: String,
        user_id: Uuid,
        channel_id: Uuid,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            message_id,
            user_id,
            message,
            channel_id,
            timestamp,
        }
    }
}
