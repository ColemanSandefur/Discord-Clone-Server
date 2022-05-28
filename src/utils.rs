use mysql::prelude::Queryable;
use uuid::Uuid;

pub fn get_user<T: Queryable>(tsx: &mut T, auth_token: &Uuid) -> Option<Uuid> {
    tsx.exec_first("SELECT (user_id) FROM sessions WHERE id=?", (auth_token,))
        .ok()?
}

pub fn get_user_res<T: Queryable>(tsx: &mut T, auth_token: &Uuid) -> Result<Uuid, &'static str> {
    get_user(tsx, auth_token).ok_or("Invalid auth token")
}
