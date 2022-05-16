#[macro_use]
extern crate juniper;
use chrono::{DateTime, NaiveDateTime, Utc};
use juniper::{EmptySubscription, FieldResult};
use mysql::prelude::*;
use mysql::{OptsBuilder, Pool};
use rocket::{response::content, Rocket, State};
use uuid::Uuid;

mod gql_objects;
use gql_objects::Channel;
use gql_objects::Message;

/// Available in all Queries and Mutations.
///
/// Mainly used to give global access to the database
pub struct Context {
    connection: Pool,
}

impl Context {
    pub fn new(connection: Pool) -> Self {
        Self { connection }
    }

    pub fn get_conn(&self) -> &Pool {
        &self.connection
    }
}

impl juniper::Context for Context {}

struct Query;

#[graphql_object(Context = Context)]
impl Query {
    fn apiVersion() -> &'static str {
        "0.1"
    }

    #[graphql(description = "Get all the channels associated with the login token.")]
    fn channels(context: &Context, token: Uuid) -> Vec<Channel> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        tsx.exec_map(
            "call discord_clone.get_user_channels(?);",
            (token,),
            |(id_channel, name): (String, String)| {
                return Channel::new(Uuid::parse_str(&id_channel).unwrap(), name);
            },
        )
        .unwrap()
    }

    #[graphql(description = "Returns the requested channel if user has access to the channel.")]
    fn get_channel(context: &Context, token: Uuid, channel_id: Uuid) -> Option<Channel> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let mut result: Vec<Channel> = tsx
            .exec_map(
                "call discord_clone.get_single_user_channel(?, ?);",
                (token, channel_id),
                |(id_channel, name): (String, String)| {
                    return Channel::new(Uuid::parse_str(&id_channel).unwrap(), name);
                },
            )
            .unwrap();

        if result.len() > 0 {
            Some(result.remove(0))
        } else {
            None
        }
    }

    fn get_message(context: &Context, token: Uuid, message_id: Uuid) -> Option<Message> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let mut result: Vec<Message> = tsx
            .exec_map(
                "call discord_clone.get_single_message(?, ?);",
                (token, message_id),
                |(id, message, user_id, channel_id, timestamp): (
                    Uuid,
                    String,
                    Uuid,
                    Uuid,
                    NaiveDateTime,
                )| {
                    let timestamp = DateTime::<Utc>::from_utc(timestamp, Utc);
                    println!("{}", timestamp);
                    return Message::new(id, message, user_id, channel_id, timestamp);
                },
            )
            .unwrap();

        if result.len() > 0 {
            Some(result.remove(0))
        } else {
            None
        }
    }
}

struct Mutation;

#[graphql_object(Context=Context)]
impl Mutation {
    #[graphql(description = "Create a user with username and password, returns user id")]
    fn create_user(context: &Context, username: String, password: String) -> FieldResult<Uuid> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        let uuid = Uuid::new_v4();
        tsx.exec_drop(
            "INSERT INTO users (id, username, password) VALUES (?, ?, ?)",
            (&uuid, username, hash),
        )
        .unwrap();

        tsx.commit()?;

        Ok(uuid)
    }

    #[graphql(description = "Sign in using username and password, returns the access token")]
    fn sign_in(context: &Context, username: String, password: String) -> Option<Uuid> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let user_pass_hash: Option<(Uuid, String)> = tsx
            .exec_first(
                "SELECT id, password FROM users WHERE username=?",
                (username,),
            )
            .unwrap();

        if user_pass_hash.is_none() {
            return None;
        }

        let (user_uuid, user_pass_hash) = user_pass_hash.unwrap();

        if bcrypt::verify(password, &user_pass_hash).unwrap() {
            let token_uuid = Uuid::new_v4();
            tsx.exec_drop(
                "INSERT INTO sessions (id, user_id) VALUES (?,?)",
                (&token_uuid, &user_uuid),
            )
            .unwrap();
            tsx.commit().unwrap();

            return Some(token_uuid);
        }

        return None;
    }

    #[graphql(description = "Send a message to a specific channel, returns message id")]
    fn send_message(
        context: &Context,
        token: Uuid,
        message: String,
        channel_id: Uuid,
    ) -> Option<Uuid> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let message_id = Uuid::new_v4();
        let user_id: Option<Uuid> = tsx
            .exec_first("SELECT user_id FROM sessions WHERE id=?", (token,))
            .unwrap();

        if let Some(user_id) = user_id {
            tsx.exec_drop(
                "INSERT INTO messages (id, message, user_id, channel_id) VALUES (?, ?, ?, ?)",
                (message_id, message, user_id, channel_id),
            )
            .unwrap();

            tsx.commit().unwrap();

            return Some(message_id);
        }

        return None;
    }

    #[graphql(description = "Change the content of a message")]
    fn update_message(
        context: &Context,
        token: Uuid,
        message_id: Uuid,
        message: String,
    ) -> Option<Message> {
        let mut tsx = context
            .get_conn()
            .start_transaction(Default::default())
            .unwrap();

        let user_id: Option<Uuid> = tsx
            .exec_first("SELECT user_id FROM sessions where id=?", (token,))
            .unwrap();

        if let Some(user_id) = user_id {
            tsx.exec_drop(
                "UPDATE messages SET message=? WHERE id=? AND user_id=?",
                (message, message_id, user_id),
            )
            .unwrap();

            let mut message = tsx
                .exec_map(
                    "call discord_clone.get_single_message(?, ?);",
                    (token, message_id),
                    |(id, message, user_id, channel_id, timestamp): (
                        Uuid,
                        String,
                        Uuid,
                        Uuid,
                        NaiveDateTime,
                    )| {
                        let timestamp = DateTime::<Utc>::from_utc(timestamp, Utc);
                        println!("{}", timestamp);
                        return Message::new(id, message, user_id, channel_id, timestamp);
                    },
                )
                .unwrap();

            tsx.commit().unwrap();

            if message.len() > 0 {
                return Some(message.remove(0));
            } else {
                return None;
            }
        }

        None
    }
}

type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

#[rocket::get("/gql")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/graphql?<request>")]
async fn get_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&*schema, &*context).await
}

#[rocket::post("/graphql", data = "<request>")]
async fn post_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&*schema, &*context).await
}

fn get_opts() -> OptsBuilder {
    OptsBuilder::new()
        .user(Some("mcwaffles"))
        .db_name(Some("discord_clone"))
        .pass(Some(""))
        .ip_or_hostname(Some("localhost"))
        .tcp_port(3306)
}

use rocket::fs::FileServer;

#[rocket::main]
async fn main() {
    let connection = Pool::new(get_opts()).expect("Unable to connect to database");

    // set session timezone to UTC to make sure that dates are stored in UTC timezone.
    {
        let mut tsx = connection.start_transaction(Default::default()).unwrap();
        tsx.exec_drop("SET time_zone = '+00:00'", ()).unwrap();
        tsx.commit().unwrap();
    }

    let context = Context::new(connection);

    Rocket::build()
        .manage(context)
        .manage(Schema::new(
            Query,
            Mutation,
            EmptySubscription::<Context>::new(),
        ))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        // a quick workaround for cors while developing the server
        .mount("/", FileServer::from("../discord-client/build"))
        .launch()
        .await
        .expect("server to launch");
}
