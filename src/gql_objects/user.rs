use crate::Context;
use uuid::Uuid;

pub struct User {
    id: Uuid,
    username: String,
    profile_picture: Option<String>,
}

#[graphql_object(Context=Context)]
impl User {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn profile_picture(&self) -> Option<&String> {
        self.profile_picture.as_ref()
    }
}

impl User {
    pub fn new(id: Uuid, username: String, profile_picture: Option<String>) -> Self {
        Self {
            id,
            username,
            profile_picture,
        }
    }
}
