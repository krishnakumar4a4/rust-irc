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

struct Register {
    data_map: Arc<Mutex<HashMap<String, String>>>
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

#[get("/register/<name>")]
fn register(name: String, register: State<Register>) -> String {
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
        let value = format!("{}",id);
        mutable_data_map.insert(name, value);
    }
    "register".to_string()
}

//#[post("/broadcast",data="<Message>")]
//fn broadcast_msg(message: Json<Message>) {
//
//}

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
        .mount("/", routes![index,register])
        .launch();
}