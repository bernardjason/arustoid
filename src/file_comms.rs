use std::fs::File;
use std::io::prelude::*;
use std::fs;

pub fn write_to_file(player:usize,state:&Vec<String>) -> std::io::Result<()> {
    let tmp =format!("player{}.txt.new",player) ;
    let final_name =format!("player{}.txt",player) ;
    let mut file = File::create(tmp.clone())?;
    for s in state.iter() {
        /*
        if ! s.starts_with("0" ) {
            file.write_all(format!("{} ",player).as_bytes())?;
        }
         */
        file.write_all(s.as_bytes())?;
    }
    fs::rename(tmp,final_name)?;
    Ok(())
}

pub fn read_file(player:usize,state:&mut Vec<String>) -> std::io::Result<()> {
    let other = if player == 1 { 2 } else { 1};


    let filename = format!("player{}.txt",other);
    let contents = fs::read_to_string(filename);
    if contents.is_ok() {
        contents.unwrap().lines().for_each( |line| {
            let string = format!("{}",line);
            state.push(string);
        });


    }
    Ok(())
}