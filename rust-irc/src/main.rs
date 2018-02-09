#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate reqwest;
extern crate url;
use url::{Url};

use rocket::State;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

//sqlite DB setup
extern crate rusqlite;
use rusqlite::Connection;
use rusqlite::{Result};

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
use rocket_contrib::{Json, Value};

//#[derive(Serialize)]
//struct ClientMessagesJson {
//    messages_vec: Vec<Json<ClientMessage>>
//}

#[derive(Deserialize,Serialize,Clone)]
struct ClientMessage {
    user_name: String,
    message: String,
    time: String
}

struct MetaData {
    client_data: Arc<Mutex<ClientData>>,
    sqlite_db: Arc<Mutex<Connection>>
}

struct ClientData {
    session_id: String,
    user_name: String,
    local_ip: String
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<user>")]
fn hello_user(user: String) -> String {
    format!("Hello, {}!", user)
}

#[get("/register/<name>")]
fn register_me(name: String, meta_data: State<MetaData>) -> String {
    let mut mut_client_data = meta_data.client_data.lock().unwrap();
    let uri_string = format!("http://localhost:8001/register/{}/127.0.0.1",name);
    let uri:Url = uri_string.parse().unwrap();
    let mut response = reqwest::get(uri).unwrap();
    let session_id = response.text().unwrap();
    println!("response {}",session_id);
    mut_client_data.session_id = session_id;
    mut_client_data.user_name = name;
    println!("response code {}",response.status());
    "registered u".to_string()
}

#[get("/send/<message>")]
fn send_msg(message: String, meta_data: State<MetaData>) {
    let client_data = meta_data.client_data.lock().unwrap();
    let uri_string = format!("http://localhost:8001/broadcast");
    let uri:Url = uri_string.parse().unwrap();

    let mut map = HashMap::new();
    map.insert("source_ip", "127.0.0.1");
    map.insert("user_name", &client_data.user_name);
    map.insert("session_id", &client_data.session_id);
    map.insert("message", &message);

    let client = reqwest::Client::new();
    let res = client.post(uri)
        .json(&map)
        .send().unwrap();
}

#[get("/receive/<user_name>/<message>/<time>")]
fn receive_msg(user_name: String, time: String, message: String, meta_data: State<MetaData>) {
    let mut_sqlite_conn = meta_data.sqlite_db.lock().unwrap();
    println!("received message {} and storing it locally on client",message);
    mut_sqlite_conn.execute("INSERT INTO messages(user_name, message, time) VALUES(?1,?2,?3)",
                            &[&user_name, &message, &time]).unwrap();
    ()
}

#[get("/get/messages/<count>")]
fn get_messages(count: i64, meta_data: State<MetaData>) -> String{
    let sqlite_conn = meta_data.sqlite_db.lock().unwrap();
    let mut stmt = sqlite_conn
        .prepare("SELECT id, user_name, message, time FROM messages order by id desc limit ?1").unwrap();
    let client_messages = stmt.query_map(&[&count],|row| {
        ClientMessage {
            user_name: row.get(1),
            message: row.get(2),
            time: row.get(3)
        }
    }).unwrap();

    let messages: Vec<String> = client_messages.map(|row| {
            println!("read message");
            let unwrapped_row = row.unwrap();
            let clone_row = unwrapped_row.clone();
            let mut message_metadata = String::new();
            let user_name:String = clone_row.user_name;
            let message:String = clone_row.message;
            let time:String = clone_row.time;
            message_metadata.push_str("{ \"user_name\":");
            message_metadata.push_str(&user_name.to_owned());
            message_metadata.push_str(", \"message\":");
            message_metadata.push_str(&message.to_owned());
            message_metadata.push_str(", \"time\":");
            message_metadata.push_str(&time.to_owned());
            message_metadata.push_str("}");
            message_metadata
        }).collect();
        println!("messages vector {:?}",messages);
        let mut return_value = String::new();
        return_value = messages.join(",");

//        messages.iter().for_each(|mes| {
//            return_value.push_str(mes);
//        });
        return_value
//    let mut messages: Vec<String> = Vec::new();
//    {
//        let sqlite_conn = meta_data.sqlite_db.lock().unwrap();
//        let mut stmt = sqlite_conn
//            .prepare("SELECT id, user_name, message, time FROM messages order by id desc limit ?1").unwrap();
//        stmt.query_map(&[&count],|row| {
//            ClientMessage {
//                user_name: row.get(1),
//                message: row.get(2),
//                time: row.get(3)
//            };
//            println!("read message");
//            let mut user_name:String = row.get_checked(1).unwrap();
//            let message:String = row.get_checked(2).unwrap();
//            let time:String = row.get_checked(3).unwrap();
//            user_name.push_str(&message.to_owned());
//            user_name.push_str(&time.to_owned());
//            messages.push(user_name)
//        }).unwrap();
//        println!("messages internally {:?}",messages);
//    }
//    println!("messages vector {:?}",messages);
//    let mut return_value = String::new();
//    messages.iter().for_each(|mes| {
//        return_value.push_str(mes);
//    });
//    return_value
}

fn init() -> MetaData {
    let conn = Connection::open("messages.db").unwrap();
    //Create table to store messages
    conn.execute("CREATE TABLE IF NOT EXISTS messages( \
    id INTEGER PRIMARY KEY,\
    user_name TEXT NOT NULL,\
    message TEXT NOT NULL,\
    time TEXT NOT NULL)", &[]).unwrap();
    //State
    MetaData{
        client_data: Arc::new(Mutex::new(ClientData{session_id:"".to_string(), user_name:"".to_string(), local_ip:"".to_string()})),
        sqlite_db: Arc::new(Mutex::new(conn))
    }
}

fn main() {
    rocket::ignite()
        .manage(init())
        .mount("/", routes![index,hello_user,register_me,send_msg,receive_msg,get_messages])
        .launch();
}
