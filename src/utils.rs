use mysql::prelude::Queryable;
use uuid::Uuid;

pub fn get_user<T: Queryable>(tsx: &mut T, auth_token: &Uuid) -> Option<Uuid> {
    tsx.exec_first("SELECT (user_id) FROM sessions WHERE id=?", (auth_token,))
        .ok()?
}
