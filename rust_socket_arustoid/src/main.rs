#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use rocket::State;
use rocket::fairing::AdHoc;
use rocket::http::RawStr;
use rocket::outcome::Outcome::Success;
use rocket::request::{self, FromRequest, Request};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use ws::listen;

use crate::game_logic::GameLogic;
use crate::server::Server;

mod server;
mod game_logic;

lazy_static! {
    pub static ref GLOBAL_GAME_LOGIC:Mutex<GameLogic> = Mutex::new(GameLogic::new());
}

struct ProxyDetailsAndWS {
    prefix:String,
    hostport:String,
    wshostport:String,
}


struct Http{ headers:HashMap<String,String>}

impl<'a, 'r> FromRequest<'a, 'r> for Http {
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let xx = request.headers();
        let mut headers = HashMap::<String,String>::new();
        for x in xx.iter() {
            headers.insert(x.name.to_string(), x.value.parse().unwrap());
        }
        Success(Http{ headers :headers})
    }
}

fn main() {
    thread::spawn(|| {
        let count = Rc::new(Cell::new(0));
        listen("0.0.0.0:9001", |out| {
            Server {
                out: out,
                count: count.clone(),
                player_number: "".parse().unwrap(),
                game_number: "".parse().unwrap(),
            }
        }).unwrap()
    });

    thread::spawn(|| {
        loop {
            GLOBAL_GAME_LOGIC.lock().unwrap().remove_expired();
            thread::sleep(Duration::from_secs_f32(30.0));
        }
    });


    rocket::ignite().
        mount("/", routes![index,play,worker]).
        mount("/wasm", StaticFiles::from("wasm")).
        mount("/img", StaticFiles::from("img")).
        mount("/css", StaticFiles::from("css")).
        mount("/js", StaticFiles::from("js")).
        attach(Template::fairing()).
        attach(AdHoc::on_attach("proxy prefix", |rocket| {
            let proxy_prefix = rocket.config()
                .get_str("proxy_prefix")
                .unwrap_or("/")
                .to_string();
            let wsocket_host_port = rocket.config()
                .get_str("wsocket_hostname_port")
                .unwrap_or("NOT SET!!")
                .to_string();
            let wssocket_host_port = rocket.config()
                .get_str("wssocket_hostname_port")
                .unwrap_or("NOT SET!!")
                .to_string();

            Ok(rocket.manage(ProxyDetailsAndWS { prefix:proxy_prefix , hostport:wsocket_host_port, wshostport: wssocket_host_port }))
        })).
        launch();
}


#[get("/")]
fn index(proxy_prefix:State<ProxyDetailsAndWS>) -> Template {
    let next_game_number = GLOBAL_GAME_LOGIC.lock().unwrap().next_game_number();
    let mut current_games = GLOBAL_GAME_LOGIC.lock().unwrap().get_current_games();

    current_games.sort_by(|a, b| a.parse::<i32>().unwrap().cmp(&b.parse::<i32>().unwrap()));


    let mut context: HashMap<&str, Vec<String>> = HashMap::new();
    context.insert("new_game_number", vec![next_game_number.to_string()]);
    context.insert("current_games", current_games);
    context.insert("proxy_prefix",vec![proxy_prefix.prefix.clone()]);

    Template::render("index", &context)
}

#[get("/play/<game_number>/<player_number>")]
fn play(proxy_prefix:State<ProxyDetailsAndWS>, game_number: &RawStr, player_number: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    context.insert("game_number", game_number.to_string());
    context.insert("player_number", player_number.to_string());
    context.insert("proxy_prefix",proxy_prefix.prefix.to_string());
    Template::render("play", &context)
}

#[get("/js/worker.js")]
fn worker(ws:State<ProxyDetailsAndWS>,http:Http) -> Template {
    let mut secure=false;
    for (name,value) in http.headers.iter() {
        if name.to_lowercase() == "referer" && value.to_lowercase().contains("https") {
            secure=true;
        }
    }
    let mut context: HashMap<&str, String> = HashMap::new();
    if secure {
        context.insert("protocol", "wss".parse().unwrap());
        context.insert("socket_hostname_port",ws.wshostport.clone());
    } else {
        context.insert("protocol", "ws".parse().unwrap());
        context.insert("socket_hostname_port",ws.hostport.clone());
    }
    Template::render("worker", &context)

}
