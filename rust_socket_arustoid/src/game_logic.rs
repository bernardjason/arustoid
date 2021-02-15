/*
all_games = dict()
all_game_deletes = dict()
all_games_last_update = dict()
new_game_number = 0
 */
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DeleteEntry {
    record: String,
    click_age: i32,
}

const DELETE_CLICK_AGE: i32 = 500;
const MAX_DELETE_HISTORY:usize = 200;

impl DeleteEntry {
    pub fn new(record: String) -> DeleteEntry {
        DeleteEntry {
            record,
            click_age: DELETE_CLICK_AGE,
        }
    }
    pub fn age(&mut self) -> bool {
        self.click_age = self.click_age - 1;
        self.click_age < 0
    }
}

pub struct GameLogic {
    pub all_games: HashMap<String, HashMap<String, String>>,
    pub all_game_deletes: HashMap<String, HashMap<String, DeleteEntry>>,
    pub delete_history: HashMap<String, Vec<String>>,
    pub all_games_last_update: HashMap<String, u64>,
    pub new_game_number: i32,
}

impl GameLogic {
    pub fn new() -> GameLogic {
        GameLogic {
            all_games: HashMap::new(),
            all_game_deletes: HashMap::new(),
            all_games_last_update: HashMap::new(),
            delete_history: HashMap::new(),
            new_game_number: 0,
        }
    }
    pub fn next_game_number(&mut self) -> i32 {
        self.new_game_number = self.new_game_number + 1;
        return self.new_game_number;
    }
    pub fn get_current_games(&self) -> Vec<String> {
        let mut result = Vec::<String>::new();
        self.all_games.keys().for_each(|x| result.push(x.to_string()));

        return result;
    }

    pub fn create_new_game(&mut self, game_number: String) {
        if !self.all_games.contains_key(&*game_number) {
            println!("CREATE NEW GAME {}", game_number);
            let game = HashMap::<String, String>::new();
            let deletes = HashMap::<String, DeleteEntry>::new();
            self.all_games.insert(game_number.clone(), game);
            self.all_game_deletes.insert(game_number.clone(), deletes);

            let delete_history = Vec::new();
            self.delete_history.insert(game_number.clone(), delete_history);

            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("now").as_secs();
            self.all_games_last_update.insert( game_number.clone(),now);
        } else {
            println!("******* JOIN NEW GAME {}", game_number);
        }
    }
    pub fn consume_update(&mut self, game_number: &str, player_number: &str, message: &str) {
        let game = self.all_games.get_mut(&*game_number.clone()).unwrap();
        let delete = self.all_game_deletes.get_mut(&*game_number).unwrap();
        let delete_history = self.delete_history.get_mut(&*game_number.clone()).unwrap();

        message.lines().for_each(|line| {
            if !line.starts_with("----") {
                let mut iter = line.split_whitespace();
                let j_player = iter.next().unwrap();
                let j_id = iter.next().unwrap();
                if j_player == "0" {
                    //delete.entry(j_id.parse().unwrap()).or_insert("".parse().unwrap());
                    //delete.insert(j_id.parse().unwrap(),line.parse().unwrap());
                    //delete.push(line.parse().unwrap());
                    let j_id_string: String = j_id.parse().unwrap();
                    if ! delete_history.contains(&j_id_string) {
                        if !delete.contains_key(j_id) {
                            delete.insert(j_id_string, DeleteEntry::new(line.parse().unwrap()));
                        }
                    }
                } else if j_player == player_number {
                    game.entry(j_id.parse().unwrap()).or_insert("".parse().unwrap());
                    game.insert(j_id.parse().unwrap(), line.parse().unwrap());
                }
            }
        });
        for (k, _d) in delete.iter() {
            game.remove(k);
        }
        while delete_history.len() > MAX_DELETE_HISTORY {
            delete_history.pop();
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("now").as_secs();
        self.all_games_last_update.insert(game_number.parse().unwrap(), now);

        //println!("Game size {}  delete size {}", game.len(), delete.len());
    }
    pub fn produce(&mut self, game_number: &str, player_number: &str) -> String {
        let game = self.all_games.get_mut(&*game_number.clone()).unwrap();
        let delete = self.all_game_deletes.get_mut(&*game_number).unwrap();
        let delete_history = self.delete_history.get_mut(&*game_number.clone()).unwrap();

        let mut repsonse = "".to_owned();
        for (_key, value) in game {
            let player_field = value.split_whitespace().next().unwrap();
            if player_field != player_number {
                repsonse.push_str(&*format!("{}\n", value));
            }
        }
        for delete_entry in delete.iter() {
            repsonse.push_str(&*format!("{}\n", delete_entry.1.record));
        }
        let mut key_list = Vec::new();
        delete.keys().for_each(|k| key_list.push(k.to_string()));

        for k in key_list.iter(){
            if delete.get_mut(k).unwrap().age() {
                delete.remove(k);
                delete_history.insert(0,k.to_string());
            }
        }
        //println!("RESPONSE FOR {} {}",player_number,repsonse.len());
        return repsonse;
    }

    pub fn remove_expired(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("now").as_secs();
        for (game,last_update) in self.all_games_last_update.clone() {
           if last_update + 20 < now {
               println!("Remove {}",game);
               self.all_games_last_update.remove(&*game);
               self.all_games.remove(&*game);
               self.all_game_deletes.remove(&*game);
               self.delete_history.remove(&*game);
           }
        }

    }
}