#![allow(non_upper_case_globals)]

#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap};
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;
use std::time::Instant;

use cgmath::{Matrix4, vec2, vec3, Vector2};
use emscripten_main_loop::{MainLoopEvent, run};
use rand::Rng;
use rand::rngs::ThreadRng;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, Window};

use crate::bullet::Bullet;
use crate::player_stats::PlayerStats;
use crate::rock::{Rock, RockMaker};
use crate::ship::Ship;


mod gl;
mod gl_helper;
mod ship;
mod bullet;
mod collision;
mod rock;
mod player_stats;


#[cfg(not(target_os = "emscripten"))]
mod file_comms;

#[cfg(not(target_os = "emscripten"))]
use crate::file_comms::{read_file, write_to_file};

trait Render {
    fn update(&mut self, rate: f32, gl: &gl::Gl);
    fn render(&mut self, gl: &gl::Gl);
    fn rollback(&mut self);
}

#[derive(Default, PartialEq)]
struct Dead {
    f_id: u128,
    f_type: u8,
}

pub struct Runtime {
    now: Instant,
    last: u128,
    last_rate: f32,
    sdl: Sdl,
    _video: VideoSubsystem,
    window: Window,
    _gl_context: GLContext,
    pub gl: std::rc::Rc<gl::Gl>,
    render_ship: HashMap<u128, Ship>,
    render_bullet: HashMap<u128, Bullet>,
    rockmaker: RockMaker,
    rocks: HashMap<u128, Rock>,
    dead_key_list: Vec<Dead>,
    player: usize,
    random: ThreadRng,
    ticker: u32,
    player_scores: HashMap<u128, PlayerStats>,
    level: i32,
}


pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 600;
pub const SCALE_TO_SCREEN: f32 = 0.043;

static mut GLOBAL_ID: u128 = 0;
static mut PLAYER_NUMBER: usize = 1;
static mut PLAYER_KEY: u128 = 0;
const MAX_DEAD_KEYS_TO_KEEP: usize = 200;
const DELETE_PLAYER_NUMBER: usize = 0;

pub const SHIP: u8 = 1;
pub const BULLET: u8 = 2;
pub const ROCK: u8 = 3;
pub const STATS: u8 = 4;
const FIELD_PLAYER: usize = 0;
const FIELD_ID: usize = 1;
const FIELD_TYPE: usize = 2;
const FIELD_X: usize = 3;
const FIELD_Y: usize = 4;
const FIELD_OTHER: usize = 5;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("{:?}", args);
        let player_number = args[1].parse::<usize>().unwrap();
        set_player_id(player_number);
    } else {
        std::fs::remove_file("player1.txt").ok();
        std::fs::remove_file("player2.txt").ok();
    }

    let r = Runtime::new();

    emscripten_main_loop::run(r);
}

pub fn get_player_id() -> usize {
    unsafe {
        return PLAYER_NUMBER;
    }
}

pub fn set_player_id(id: usize) {
    unsafe {
        PLAYER_NUMBER = id;
    }
}

pub fn get_player_key() -> u128 {
    unsafe {
        return PLAYER_KEY;
    }
}

pub fn set_player_key(id: u128) {
    unsafe {
        PLAYER_KEY = id;
    }
}

pub fn get_next_id() -> u128 {
    unsafe {
        let next = PLAYER_NUMBER as u128 * 524288 + GLOBAL_ID;
        GLOBAL_ID = GLOBAL_ID + 1;
        next
    }
}

lazy_static! {
    static ref DATA_TO_RETURN: Mutex<String> = Mutex::new(String::with_capacity(4096));
    static ref DATA_FROM_OTHERS: Mutex<String> = Mutex::new(String::with_capacity(4096));
    static ref DATA_STATS: Mutex<String> = Mutex::new(String::with_capacity(4096));
}

#[cfg(not(target_os = "emscripten"))]
pub fn get_player_number() -> i32 {
    get_player_id() as i32
}

//#[cfg(not(target_os = "emscripten"))]
pub fn get_current_data() -> CString {
    let mut xx = DATA_FROM_OTHERS.lock().unwrap();
    let s = CString::new(xx.as_str()).unwrap();
    xx.clear();
    return s;
}

//#[cfg(not(target_os = "emscripten"))]
pub fn write_current_data(output: *const c_char) {
    unsafe {
        let rust = CStr::from_ptr(output);
        let mut data = DATA_TO_RETURN.lock().unwrap();
        data.clear();
        data.push_str(rust.to_str().unwrap());
        data.push_str("--------------------------------");
    }
}

pub fn write_stats_data(output: *const c_char) {
    unsafe {
        let rust = CStr::from_ptr(output);
        let mut data = DATA_STATS.lock().unwrap();
        data.clear();
        data.push_str(rust.to_str().unwrap());
        data.push(char::from(0));
        //data.push_str("--------------------------------");
    }
}

#[no_mangle]
pub extern "C" fn javascript_write(input: *const c_char) -> i32 {
    unsafe {
        let rust = CStr::from_ptr(input);
        let mut data = DATA_FROM_OTHERS.lock().unwrap();
        data.truncate(0);
        data.push_str(rust.to_str().unwrap());
    }
    0
}

#[no_mangle]
pub extern "C" fn javascript_read() -> *const c_char {
    unsafe {
        let xx = DATA_TO_RETURN.lock().unwrap();
        let got = CStr::from_bytes_with_nul_unchecked(xx.as_bytes());
        let on_heap = Box::new(got);
        return on_heap.as_ptr();
    }
}

#[no_mangle]
pub extern "C" fn javascript_read_stats() -> *const c_char {
    unsafe {
        let xx = DATA_STATS.lock().unwrap();
        let got = CStr::from_bytes_with_nul_unchecked(xx.as_bytes());
        let on_heap = Box::new(got);
        return on_heap.as_ptr();
    }
}


#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn get_player_number() -> i32;
}

#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn start_javascript_worker_thread() -> i32;
}

#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn start_javascript_play_sound(sound_id: i32) -> i32;
}

impl Runtime {
    fn new() -> Runtime {
        let sdl = sdl2::init().unwrap();
        let mut player_id: i32 = 1;

        // Setup the video subsystem
        let video = sdl.video().unwrap();

        #[cfg(not(target_os = "emscripten"))]
            let context_params = (sdl2::video::GLProfile::Core, 3, 0);
        #[cfg(target_os = "emscripten")]
            let context_params = (sdl2::video::GLProfile::GLES, 3, 0);
        unsafe {
            let player_id = get_player_number();
            PLAYER_NUMBER = player_id as usize;
            println!("**************** PLAYER IS {}", PLAYER_NUMBER);
            let hello_work = "HELLO WORLD";
            write_current_data(CString::new(hello_work).to_owned().unwrap().as_ptr());
        }


        video.gl_attr().set_context_profile(context_params.0);
        video.gl_attr().set_context_major_version(context_params.1);
        video.gl_attr().set_context_minor_version(context_params.2);

        // Create a window
        let window = video
            .window(format!("aRustOid player {}", player_id).as_ref(), self::WIDTH, self::HEIGHT)
            .resizable()
            .opengl()
            .position_centered()
            .build().unwrap();


        // Create an OpenGL context
        let gl_context = window.gl_create_context().unwrap();
        let gl_orig: std::rc::Rc<gl::Gl> = std::rc::Rc::new(gl::Gl::load_with(|s| { video.gl_get_proc_address(s) as *const _ }));

        let gl = std::rc::Rc::clone(&gl_orig);

        unsafe { gl.Enable(gl::BLEND); }

        let mut shipy = 0.0;
        unsafe {
            shipy = (get_player_number() - 1) as f32 * 128.0;
        }
        let ship1 = Ship::new(&gl, 0.0, shipy, get_next_id(), get_player_id());

        set_player_key(ship1.id);
        let rock_maker = RockMaker::new("rock.png", &gl);


        let mut runtime = Runtime {
            now: Instant::now(),
            last: 0,
            last_rate: 0.0,
            sdl,
            _video: video,
            window,
            _gl_context: gl_context,
            gl: gl_orig,
            render_ship: HashMap::new(),
            render_bullet: HashMap::new(),
            rockmaker: rock_maker,
            rocks: HashMap::new(),
            dead_key_list: Vec::with_capacity(MAX_DEAD_KEYS_TO_KEEP),
            player: get_player_id(),
            random: rand::thread_rng(),
            ticker: 0,
            player_scores: HashMap::new(),
            level: 1,
        };
        runtime.render_ship.insert(ship1.id, ship1);
        runtime.player_scores.insert(runtime.player as u128, PlayerStats::new(get_next_id(), runtime.player));


        runtime
    }
}

impl emscripten_main_loop::MainLoop for Runtime {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        #[cfg(target_os = "emscripten")]
        if self.ticker == 0 {
            unsafe {
                start_javascript_worker_thread();
            }
        }

        if self.rocks.len() <= 1 {
            self.add_rocks();
        }

        self.ticker = self.ticker + 1;

        let start = self.now.elapsed().as_millis();
        let diff = start - self.last;
        self.last = start;
        let new_rate = diff as f32 / 16.0;
        let rate = self.last_rate + new_rate /2.0;
        self.last_rate = new_rate;

        let return_status = self.handle_keyboard();

        for r in self.render_ship.iter_mut() {
            r.1.update(rate, &self.gl);
        }
        for r in self.rocks.iter_mut() {
            r.1.update(rate, &self.gl);
        }
        self.render_ship.get_mut(&get_player_key()).unwrap().add_new_bullets(&self.gl, &mut self.render_bullet);

        self.do_collision_logic();

        self.do_bullet_updates(rate);

        unsafe {
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
        }

        for r in self.render_ship.iter_mut() {
            r.1.render(&self.gl);
        }
        for r in self.render_bullet.iter_mut() {
            r.1.render(&self.gl);
        }
        for r in self.rocks.iter_mut() {
            r.1.render(&self.gl);
        }
        self.window.gl_swap_window();

        let mut state: Vec<String> = vec![];
        for s in self.player_scores.iter() {
            if s.1.player_created_by == get_player_id() {
                state.push(format!("{} {}", self.player, s.1));
            }
        }

        for s in self.render_ship.iter() {
            if s.1.player_created_by == get_player_id() {
                state.push(format!("{} {}", self.player, s.1));
            }
        }
        for s in self.render_bullet.iter() {
            if s.1.player_created_by == get_player_id() {
                state.push(format!("{} {}", self.player, s.1));
            }
        }
        for s in self.rocks.iter() {
            if s.1.player_created_by == get_player_id() {
                state.push(format!("{} {}", self.player, s.1));
            }
        }
        for d in self.dead_key_list.iter() {
            let entry = format!("{} {} {}\n", DELETE_PLAYER_NUMBER, d.f_id, d.f_type);
            state.push(entry);
        }
        while self.dead_key_list.len() >= MAX_DEAD_KEYS_TO_KEEP {
            self.dead_key_list.pop();
        }

        #[cfg(not(target_os = "emscripten"))]
            self.test_with_files(&mut state);

        #[cfg(target_os = "emscripten")]
            self.handle_web(&mut state);

        self.update_remote_objects(&mut state);

        return_status
    }
}

impl Runtime {
    fn add_rocks(&mut self) {
        let mut random: ThreadRng = rand::thread_rng();
        let mut total_rocks = self.level * 2;

        let mut t:f32 = random.gen_range(0.0,360.0);
        let radius = HEIGHT as f32 / 2.0;
        while total_rocks > 0 {
            let x  = radius*t.to_radians().cos() ;
            let y = radius*t.to_radians().sin() ;
            t=t+random.gen_range(45.0,60.0);

            let dir = random_rock_dir(&mut random);
            let rock = self.rockmaker.new_rock(vec2(x, y), dir, 32.0, &self.gl, get_next_id(), get_player_id());
            self.rocks.insert(rock.id, rock);
            //println!("Added rock {},{} {}", x, y, total_rocks);
            total_rocks = total_rocks - 1;
        }
        self.level = self.level + 1;
    }
    fn handle_web(&mut self, state: &mut Vec<String>) {
        let to_web: String = format!("{}", state.join(""));
        write_current_data(CString::new(to_web).to_owned().unwrap().as_ptr());
        state.clear();

        let mut list: Vec<String> = Vec::new();

        self.player_scores.values().for_each(|x| {
            list.push(format!("{} score {} hits {}", x.player_created_by, x.score, x.lives));
        });
        write_stats_data(CString::new(list.join("\n")).to_owned().unwrap().as_ptr());

        let remote_updates = get_current_data();
        for s in remote_updates.to_string_lossy().split("\n") {
            if s.len() > 0 {
                state.push(s.to_string());
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::main_triangle;

    #[test]
    fn it_works() {
        main_triangle();
        assert_eq!(2 + 2, 4);
    }
}

impl Runtime {
    fn do_bullet_updates(&mut self, rate: f32) {
        let mut remove_item = Vec::<u128>::new();

        for k in self.render_bullet.iter_mut() {
            k.1.update(rate, &self.gl);
            if k.1.dead() {
                remove_item.push(*k.0);
            }
        }

        for i in remove_item {
            self.render_bullet.remove(&i);
            self.add_dead_entry(i, BULLET);
        }
    }
}

fn random_rock_dir(random: &mut ThreadRng) -> Vector2<f32> {
    let x: f32 = random.gen_range(0.0, 2.0) - 1.0;
    let y: f32 = random.gen_range(0.0, 2.0) - 1.0;
    return vec2(x, y);
}

impl Runtime {
    fn handle_keyboard(&mut self) -> MainLoopEvent {
        let mut return_status = emscripten_main_loop::MainLoopEvent::Continue;
        let mut events = self.sdl.event_pump().unwrap();
        let player = self.render_ship.get_mut(&get_player_key()).unwrap();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return_status = emscripten_main_loop::MainLoopEvent::Terminate;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    player.rotate(1.0)
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    player.rotate(-1.0)
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    player.forward(1.0)
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    player.fire(true);
                }
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => { player.rotate(0.0) }
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => { player.rotate(0.0) }
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => { player.forward(0.0) }
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => { player.forward(0.0) }
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => { player.fire(false) }
                _ => {}
            }
        }
        return_status
    }
}

impl Runtime {
    fn add_update_remote_objects(&mut self, f_player: usize, f_id: &u128, f_type: u8, f_x: f32, f_y: f32, f_other: f32) {
        match f_type {
            SHIP => {
                if !self.render_ship.contains_key(&f_id) {
                    //println!("******** ADD SHIP **************");
                    let ship = Ship::new(&self.gl, f_x, f_y, *f_id, f_player);
                    self.render_ship.insert(ship.id, ship);
                } else {
                    let ship = self.render_ship.get_mut(&f_id).unwrap();
                    ship.rotate_angle_total = f_other;
                    ship.xy.x = f_x;
                    ship.xy.y = f_y;
                }
            }
            BULLET => {
                if !self.dead_key_list.contains(&Dead { f_type: BULLET, f_id: *f_id }) {
                    if !self.render_bullet.contains_key(&f_id) {
                        //println!("******** ADD BULLET ************** {} {}", f_player, f_id);
                        let b = Bullet::new(vec2(f_x, f_y), vec2(0.0f32, 0.0), &self.gl,
                                            Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0)), "bullet.png", *f_id, f_player);
                        self.render_bullet.insert(b.id, b);
                    } else {
                        let b = self.render_bullet.get_mut(&f_id).unwrap();
                        b.xy.x = f_x;
                        b.xy.y = f_y;
                    }
                }
            }
            ROCK => {
                if !self.dead_key_list.contains(&Dead { f_type: ROCK, f_id: *f_id }) {
                    if !self.rocks.contains_key(&f_id) {
                        //println!("******** ADD REMOTE ROCK **************");
                        let rock = self.rockmaker.new_rock(vec2(f_x, f_y), vec2(0.0, 0.0), f_other, &self.gl, *f_id, f_player);
                        self.rocks.insert(rock.id, rock);
                    } else {
                        let rock = self.rocks.get_mut(&f_id).unwrap();
                        rock.xy.x = f_x;
                        rock.xy.y = f_y;
                    }
                }
            }
            STATS => {
                let key = f_player as u128;
                if !self.player_scores.contains_key(&key) {
                    let mut stats = PlayerStats::new(*f_id, f_player);
                    stats.score = f_x as i32;
                    stats.lives = f_y as i32;
                    self.player_scores.insert(key, stats);
                } else {
                    self.player_scores.entry(key).and_modify(|x| {
                        x.score = f_x as i32;
                        x.lives = f_y as i32;
                    });
                }
            }
            _ => {}
        }
    }
}

impl Runtime {
    fn add_dead_entry(&mut self, k: u128, f_type: u8) {
        let dead = Dead { f_id: k, f_type };

        let mut found = false;
        for d in self.dead_key_list.iter() {
            if d.f_id == k && d.f_type == f_type {
                found = true;
            }
        }
        if ! found {
            self.dead_key_list.insert(0,dead);
        }
    }
}

impl Runtime {
    fn update_remote_objects(&mut self, state: &mut Vec<String>) {
        for s in state.iter() {
            let fields: Vec<&str> = s.split(" ").collect();
            let f_player = fields[FIELD_PLAYER].parse::<usize>().unwrap();
            let f_id = fields[FIELD_ID].parse::<u128>().unwrap();
            let f_type = fields[FIELD_TYPE].parse::<u8>().unwrap();
            if f_player != get_player_id() {
                if f_player == DELETE_PLAYER_NUMBER {
                    match f_type {
                        SHIP => {
                            self.render_ship.remove(&f_id);
                        }
                        BULLET => {
                            self.render_bullet.remove(&f_id);
                            self.add_dead_entry(f_id, BULLET);
                        }
                        ROCK => {
                            self.rocks.remove(&f_id);
                            self.add_dead_entry(f_id, ROCK);
                        }
                        _ => {}
                    }
                } else {
                    let f_x = fields[FIELD_X].parse::<f32>().unwrap();
                    let f_y = fields[FIELD_Y].parse::<f32>().unwrap();
                    let f_other = fields[FIELD_OTHER].parse::<f32>().unwrap();
                    self.add_update_remote_objects(f_player, &f_id, f_type, f_x, f_y, f_other)
                }
            }
        }
    }
}

impl Runtime {
    #[cfg(not(target_os = "emscripten"))]
    fn test_with_files(&mut self, mut state: &mut Vec<String>) {
        write_to_file(self.player, &state).unwrap();
        state.clear();
        read_file(self.player, &mut state).unwrap();
    }
}

impl Runtime {
    fn add_score(&mut self, player: u128, add: i32) {
        self.player_scores.entry(player).and_modify(|x| x.score = x.score + add);
    }
    fn add_life_taken(&mut self, player: u128, add: i32) {
        self.player_scores.entry(player).and_modify(|x| x.lives = x.lives + add);
    }

    fn do_collision_logic(&mut self) {
        let mut new_rocks: Vec<Rock> = vec![];
        let mut remove_rock_list: Vec<u128> = vec![];
        let mut remove_bullet_list: Vec<u128> = vec![];
        let mut add_scores: HashMap<u128, i32> = HashMap::new();
        let mut add_lives: HashMap<u128, i32> = HashMap::new();
        for rock in self.rocks.iter_mut() {
            let mut remove_rock = false;
            for ship_index in self.render_ship.iter_mut() {
                if rock.1.collision.collide(&ship_index.1.collision) {
                    remove_rock = true;
                    *add_lives.entry(ship_index.1.player_created_by as u128).or_insert(0) = 1;
                }

                for bullet_index in self.render_bullet.iter_mut() {
                    if bullet_index.1.collision.collide(&rock.1.collision) {
                        remove_rock = true;
                        bullet_index.1.set_to_dead();
                        remove_rock_list.push(*bullet_index.0);
                        *add_scores.entry(bullet_index.1.player_created_by as u128).or_insert(0) += 1;
                    } else if bullet_index.1.player_created_by != get_player_id() && bullet_index.1.collision.collide(&ship_index.1.collision) {
                        bullet_index.1.set_to_dead();
                        remove_bullet_list.push(*bullet_index.0);
                        *add_lives.entry(ship_index.1.player_created_by as u128).or_insert(0) = 1;
                    }
                }
            }
            if remove_rock {
                rock.1.set_to_dead();
                if rock.1.size > 8.0 {
                    let rock0 = self.rockmaker.new_rock(rock.1.xy, random_rock_dir(&mut self.random), rock.1.size / 2.0, &self.gl, get_next_id(), get_player_id());
                    let rock1 = self.rockmaker.new_rock(rock.1.xy, random_rock_dir(&mut self.random), rock.1.size / 2.0, &self.gl, get_next_id(), get_player_id());
                    new_rocks.push(rock0);
                    new_rocks.push(rock1);
                    //println!("Added 2 rocks {}", new_rocks.len());
                }
                remove_rock_list.push(rock.1.id);
            }
        }

        let ship1 = self.render_ship.get(&get_player_key()).unwrap();
        let mut rollback = false;

        self.render_ship.values().filter(|ship2| ship2.id != ship1.id).for_each(|ship2| {
            if ship1.collision.collide(&ship2.collision) {
                rollback = true;
            }
        }
        );

        if rollback {
            *add_lives.entry(get_player_id() as u128).or_insert(0) = 1;
            self.render_ship.get_mut(&get_player_key()).unwrap().rollback_bounce_off();
        }

        #[cfg(target_os = "emscripten")]
        if remove_rock_list.len() > 0 {
            unsafe {
                start_javascript_play_sound(1);
            }
        }
        for k in remove_rock_list {
            self.rocks.remove(&k);
            self.add_dead_entry(k, ROCK);
        }
        for k in remove_bullet_list {
            self.render_bullet.remove(&k);
            self.add_dead_entry(k, BULLET);
        }


        for n in new_rocks {
            self.rocks.insert(n.id, n);
        }
        for a in add_scores {
            self.add_score(a.0, a.1);
            //println!("Add score {} {}", a.0, a.1)
        }
        for a in add_lives {
            self.add_life_taken(a.0, a.1);
            //println!("Add life taken {} {}", a.0, a.1)
        }
    }
}
