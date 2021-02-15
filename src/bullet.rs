use crate::gl_helper::sprite::Sprite;
use crate::{Render, SCALE_TO_SCREEN, BULLET, get_player_id};
use crate::gl;
use cgmath::{Matrix4, vec3, Vector2, };
use crate::collision::Collision;
use std::fmt;
use std::fmt::Formatter;

const SPEED: f32 = SCALE_TO_SCREEN * 2.0;
const FORWARD: f32 = 2.0;
const FIRE_AWAY_FROM_SHIP:f32 =14.0;

pub struct Bullet {
    pub(crate) id: u128,
    pub player_created_by:usize,
    sprite: Sprite,
    ticker: i32,
    pub xy: Vector2<f32>,
    pub dir: Vector2<f32>,
    pub collision: Collision,

}
impl fmt::Display for Bullet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {} 0\n", self.id,BULLET,self.xy.x,self.xy.y)
    }
}
impl Bullet {
    pub fn new(xy: Vector2<f32>, dir: Vector2<f32>, gl: &gl::Gl, transform: Matrix4<f32>, image_file: &str,id:u128,player_created_by:usize) -> Bullet {
        let mut b = Bullet {
            id,
            player_created_by,
            sprite: Sprite::new(&gl, 0.0, 0.0, image_file, 0.05, 0.15,None),
            ticker: 150,
            xy,
            dir,
            collision: Collision::new(xy.x, xy.y, 2.0),
        };
        b.sprite.transform = transform;
        b.sprite.forward = Matrix4::<f32>::from_translation(vec3(0.0, SPEED , 0.0));
        b.move_forward(FIRE_AWAY_FROM_SHIP);
        b
    }
    pub fn set_to_dead(&mut self) {
        self.ticker = -1;
    }
    pub fn dead(&self) -> bool {
        if self.ticker < 0 {
            true
        } else {
            false
        }
    }
}

impl Render for Bullet {
    fn update(&mut self, rate: f32, _gl: &gl::Gl) {
        if self.player_created_by == get_player_id() {
            self.move_forward(rate);
        } else {
            self.sprite.transform =
                Matrix4::<f32>::from_translation(vec3(self.xy.x * SCALE_TO_SCREEN, self.xy.y * SCALE_TO_SCREEN, 0.0))
        }
        self.collision.x = self.xy.x;
        self.collision.y = self.xy.y;
    }

    fn render(&mut self, gl: &gl::Gl) {
        self.sprite.render(gl);
    }

    fn rollback(&mut self) {
    }
}

impl Bullet {
    fn move_forward(&mut self, rate: f32) {
        self.sprite.forward = Matrix4::<f32>::from_translation(vec3(0.0, SPEED * rate, 0.0));
        self.sprite.transform = self.sprite.transform * self.sprite.forward;
        self.ticker = self.ticker - 1;

        self.xy += self.dir * rate * FORWARD;
    }
}
