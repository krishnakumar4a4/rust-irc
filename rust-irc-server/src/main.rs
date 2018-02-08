#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate simple_redis;
extern crate rand;
use std::collections::HashMap;
use rocket::State;
use rand::{thread_rng, Rng};
use std::sync::{Mutex, Arc};
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
use rocket_contrib::{Json, Value};

extern crate reqwest;
extern crate url;
use url::{Url};

struct ClientId {
    source_ip: String,
    session_id: String
}

struct Register {
    data_map: Arc<Mutex<HashMap<String, ClientId>>>
}

#[derive(Deserialize)]
struct Message {
    source_ip: String,
    user_name: String,
    session_id: String,
    message: String
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/register/<name>/<ip>")]
fn register(name: String, ip: String, register: State<Register>) -> String {
    println!("register {}",name);
    let mut mutable_data_map = register.data_map.lock().unwrap();
    let found = match mutable_data_map.get(&name) {
        Some(data) => {
            println!("already registered");
            true
        },
        None => {
            println!("registering");
            false
        }
    };

    if !found {
        let mut rng = thread_rng();
        let id = rng.gen::<usize>();
        println!("id generated for {} is {}", name, id);
        let session_id = format!("{}",id);
        mutable_data_map.insert(name, ClientId{source_ip: ip, session_id: session_id.clone()});
        session_id
    } else {
        "This name is already registered".to_string()
    }
}

#[post("/broadcast",data="<message>")]
fn broadcast_msg(message: Json<Message>, register: State<Register>) {
    let mutable_data_map = register.data_map.lock().unwrap();
    let found = match mutable_data_map.get(&message.user_name) {
        Some(clientId) => {
            println!("found clientid {};{}", clientId.session_id, clientId.source_ip);
            println!("received details {};{};{}", message.session_id, message.source_ip, message.user_name);
            if clientId.session_id == message.session_id
                && clientId.source_ip == message.source_ip {
                println!("clientid {};{}", clientId.session_id, clientId.source_ip);
                true
            }else {
                false
            }
        },
        None => {
            false
        }
    };

    if found {
        println!("sending message to all {}", message.message);
        for val in mutable_data_map.values() {
            println!("sending to {}, {}",val.source_ip, val.session_id)
        }
        let uri_string = format!("http://localhost:8000/receive/{}",message.message);
        let uri:Url = uri_string.parse().unwrap();
        let mut response = reqwest::get(uri).unwrap();
        println!("send to client {}", response.text().unwrap())
    }
}

//Logout implementation
//Periodic Cache clear
//get messages
//get users
//add message time


//fn start_redis() -> Option<simple_redis::client::Client> {
//    match simple_redis::create("redis://127.0.0.1:6379/") {
//        Ok(mut client) =>  {
//            println!("Created Redis Client");
//            Some(client)
//        },
//        Err(error) => {
//            println!("Unable to create Redis client: {}", error);
//            None
//        }
//    }
//}

fn main() {
    rocket::ignite()
//        .manage(start_redis())
        .manage(Register {data_map: Arc::new(Mutex::new(HashMap::new()))})
        .mount("/", routes![index,register,broadcast_msg])
        .launch();
}