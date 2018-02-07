#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<user>")]
fn hello_user(user: String) -> String {
    format!("Hello, {}!", user)
}

fn main() {
    rocket::ignite().mount("/", routes![index,hello_user]).launch();
}
