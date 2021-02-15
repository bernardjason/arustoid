use cgmath::{Basis2, Deg, Matrix4, Rotation, Rotation2, vec3, Vector2, Zero};

use crate::{Render, SCALE_TO_SCREEN, SHIP, get_player_id, get_next_id, WIDTH, HEIGHT, get_player_number};
use crate::bullet::Bullet;
use crate::collision::Collision;
use crate::gl;
use crate::gl_helper::sprite::Sprite;
use std::fmt;
use std::fmt::Formatter;
use std::collections::HashMap;

const FORWARD:f32 = 1.0;
const SPEED:f32 = SCALE_TO_SCREEN;

pub struct Ship {
    pub id: u128,
    pub player_created_by:usize,
    sprite: Sprite,
    rollback_to:Matrix4<f32>,
    rollback_xy: Vector2<f32>,
    fire: bool,
    fire_counter: i32,
    add_bullet: bool,
    forward: f32,
    rotate: f32,
    pub rotate_angle_total:f32,
    pub xy: Vector2<f32>,
    pub dir: Vector2<f32>,
    pub collision: Collision,
}
impl fmt::Display for Ship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {} {}\n", self.id,SHIP,self.xy.x,self.xy.y,self.rotate_angle_total)
    }
}
impl Ship {
    pub fn new(gl: &gl::Gl, x: f32, y: f32,id:u128,player_created_by:usize) -> Ship {
        Ship {
            id,
            player_created_by,
            sprite: Sprite::new(&gl, x, y, "rocket.png", 1.0, 1.0,None),
            rollback_to:Matrix4::<f32>::from_translation(vec3(0.0,0.0,0.0)),
            rollback_xy: Vector2::<f32>::new(x, y),
            xy: Vector2::<f32>::new(x, y),
            dir: Vector2::<f32>::new(0.0,1.0),
            fire: false,
            fire_counter: 0,
            add_bullet: false,
            forward: 0.0,
            rotate: 0.0,
            rotate_angle_total:0.0,
            collision: Collision::new(x, y, 18.0),
        }
    }
    pub fn rotate(&mut self, by: f32) {
        self.rotate = by;
    }
    pub fn forward(&mut self, by: f32) {
        if by == 0.0 || self.forward >= 0.0 {
            self.forward = SPEED * by;
        }
    }
    pub fn fire(&mut self, fire: bool) {
        self.fire = fire;
    }
    pub fn add_new_bullets(&mut self, gl: &gl::Gl, render: &mut HashMap<u128,Bullet>) {
        if self.add_bullet {
            let b = Bullet::new(self.xy,self.dir,gl, self.sprite.transform, "bullet.png",get_next_id(),get_player_id());
            render.insert(b.id,b);
            self.add_bullet = false;
        }
    }
    pub fn rollback_bounce_off(&mut self) {
        //self.forward = -1.0 * SPEED;
        //self.move_players_ship(2.0);
        self.xy.x = 0.0;
        unsafe {
            self.xy.y = (get_player_number()-1) as f32 * 128.0;
        }
        self.sprite.transform = Matrix4::<f32>::from_translation(vec3(self.xy.x * SCALE_TO_SCREEN, self.xy.y * SCALE_TO_SCREEN, 0.0));
        self.sprite.rotate = Matrix4::<f32>::from_angle_z(Deg(self.rotate_angle_total ));
        self.sprite.transform = self.sprite.transform * self.sprite.rotate ;
    }
}

impl Render for Ship {
    fn update(&mut self, rate: f32, _gl: &gl::Gl) {
        if self.player_created_by == get_player_id() {
            self.move_players_ship(rate);
        } else {
            self.sprite.transform =
                Matrix4::<f32>::from_translation(vec3(self.xy.x * SCALE_TO_SCREEN, self.xy.y * SCALE_TO_SCREEN, 0.0)) *
                Matrix4::<f32>::from_angle_z(Deg(self.rotate_angle_total ));
        }
        self.collision.x = self.xy.x;
        self.collision.y = self.xy.y;
    }


    fn render(&mut self, gl: &gl::Gl) {
        self.sprite.render(gl);
    }

    fn rollback(&mut self) {
        self.sprite.transform = self.rollback_to;
        self.xy = self.rollback_xy;
        self.collision.x = self.xy.x;
        self.collision.y = self.xy.y;
    }
}

impl Ship {
    fn move_players_ship(&mut self, rate: f32) {
        self.rollback_to = self.sprite.transform;
        self.rollback_xy = self.xy;
        self.sprite.forward = Matrix4::<f32>::from_translation(vec3(0.0, self.forward * rate, 0.0));
        self.sprite.rotate = Matrix4::<f32>::from_angle_z(Deg(self.rotate * rate));
        self.sprite.transform = self.sprite.transform * self.sprite.rotate * self.sprite.forward;

        let basis: Basis2<f32> = Rotation2::from_angle(Deg(self.rotate * rate));
        self.dir = basis.rotate_vector(self.dir);
        if !self.forward.is_zero() {
            let forward = if self.forward > 0.0 {
                FORWARD
            } else {
                -FORWARD
            };
            self.xy += self.dir * forward * rate;
        }
        self.rotate_angle_total = self.rotate_angle_total + self.rotate * rate;

        let x_over = self.xy.x.abs() > WIDTH as f32 / 2.0;
        let y_over = self.xy.y.abs() > HEIGHT as f32 / 2.0;
        if x_over || y_over {
            if x_over {
                self.xy.x = self.xy.x * -1.0;
            }
            if y_over {
                self.xy.y = self.xy.y * -1.0;
            }

            self.sprite.transform = Matrix4::<f32>::from_translation(vec3(self.xy.x * SCALE_TO_SCREEN, self.xy.y * SCALE_TO_SCREEN, 0.0));
            self.sprite.rotate = Matrix4::<f32>::from_angle_z(Deg(self.rotate_angle_total ));
            self.sprite.transform = self.sprite.transform * self.sprite.rotate ;
        }


        if self.fire && self.fire_counter <= 0 {
            self.fire_counter = 30;
            self.add_bullet = true;
        }
        self.fire_counter = self.fire_counter - 1;
    }

}
