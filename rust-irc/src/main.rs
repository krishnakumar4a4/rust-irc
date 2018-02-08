#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate reqwest;
extern crate url;
use url::{Url};

use rocket::State;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

struct MetaData {
    client_data: Arc<Mutex<ClientData>>
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

#[get("/receive/<message>")]
fn receive_msg(message: String) {
    println!("received message {} and storing it locally on client",message);
}

fn main() {
    rocket::ignite()
        .manage(MetaData{ client_data: Arc::new(Mutex::new(ClientData{session_id:"".to_string(), user_name:"".to_string(), local_ip:"".to_string()}))})
        .mount("/", routes![index,hello_user,register_me,send_msg,receive_msg])
        .launch();
}
