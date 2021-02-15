use std::fmt;
use std::fmt::Formatter;
use crate::STATS;

pub struct PlayerStats {
    pub(crate) id: u128,
    pub(crate) player_created_by:usize,
    pub score:i32,
    pub lives:i32,
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {} 0\n", self.id,STATS,self.score,self.lives)
    }
}

impl PlayerStats {
    pub fn new(id:u128,player_created_by:usize) -> PlayerStats {
        PlayerStats{
            id,
            player_created_by,
            score:0,
            lives:0,
        }

    }

}