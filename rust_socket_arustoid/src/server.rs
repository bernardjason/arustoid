use std::cell::Cell;
use std::rc::Rc;

use ws::{CloseCode, Error, Handler, Handshake, Message, Result, Sender};

use crate::GLOBAL_GAME_LOGIC;

pub struct Server {
    pub out: Sender,
    pub game_number:String,
    pub player_number:String,
    pub count: Rc<Cell<usize>>,
}

impl Handler for Server {

    fn on_open(&mut self, handshake: Handshake) -> Result<()> {

        let url = handshake.request.resource();
        println!("PATH IS {}",url);

        let splits = url.split("/").collect::<Vec<&str>>();
        let player_number = splits[2];
        let game_number = splits[1];
        self.game_number = game_number.parse().unwrap();
        self.player_number = player_number.parse().unwrap();

        println!("Player is {} and game is {}   len is {}",player_number,game_number,splits.len());

        GLOBAL_GAME_LOGIC.lock().unwrap().create_new_game(game_number.to_string());

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        GLOBAL_GAME_LOGIC.lock().unwrap().consume_update(&*self.game_number, &*self.player_number, msg.as_text().unwrap());

        let response = GLOBAL_GAME_LOGIC.lock().unwrap().produce(&*self.game_number, &*self.player_number);
        let r = Message::text(response);
        self.out.send(r)
    }



    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection. {}",self.player_number),
            CloseCode::Away   => println!("The client is leaving the site. {}",self.player_number),
            _ => println!("The client encountered an error: {}   {}", reason,self.player_number),
        }

    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);

    }

}

