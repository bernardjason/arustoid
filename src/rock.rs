use crate::gl_helper::sprite::Sprite;
use crate::{Render, SCALE_TO_SCREEN, WIDTH, HEIGHT, ROCK, get_player_id};
use crate::gl;
use cgmath::{Matrix4, vec3, Vector2, };
use crate::collision::Collision;
use crate::gl_helper::texture::{create_texture_png, create_texture_jpg};
use std::fmt;
use std::fmt::Formatter;

const SPEED: f32 = SCALE_TO_SCREEN ;
const FORWARD: f32 = 1.0;

pub struct RockMaker {
    texture:u32,
}
impl RockMaker {
    pub fn new(image_file:&str,gl: &gl::Gl) -> RockMaker {
        let texture =
            if image_file.ends_with(".png") {
                create_texture_png(&gl, image_file)
            } else {
                create_texture_jpg(&gl, image_file)
            };

        RockMaker{
           texture,
        }
    }
    pub fn new_rock(&self,xy: Vector2<f32>, dir: Vector2<f32>,size:f32, gl: &gl::Gl,id:u128,player_created_by:usize) -> Rock {
        Rock::new(xy, dir, size , gl, self.texture,id,player_created_by)
    }
}
pub struct Rock {
    pub(crate) id: u128,
    pub(crate) player_created_by:usize,
    hit:bool,
    pub size:f32,
    sprite: Sprite,
    pub xy: Vector2<f32>,
    pub dir: Vector2<f32>,
    pub collision: Collision,

}
impl fmt::Display for Rock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {} {}\n", self.id,ROCK,self.xy.x,self.xy.y,self.size)
    }
}
impl Rock {
    pub fn new(xy: Vector2<f32>, dir: Vector2<f32>,size:f32, gl: &gl::Gl ,texture:u32,id:u128,player_created_by:usize) -> Rock {
        Rock {
            id,
            player_created_by,
            hit:false,
            size,
            sprite: Sprite::new(&gl, xy.x, xy.y, "".as_ref(), size /32.0, size/32.0,Some(texture)),
            xy,
            dir,
            collision: Collision::new(xy.x, xy.y, size * 0.6),
        }
    }
    pub fn set_to_dead(&mut self) {
        self.hit = true;
    }
}

impl Render for Rock {
    fn update(&mut self, rate: f32, _gl: &gl::Gl) {
        if self.player_created_by == get_player_id() {
            self.move_forward(rate);
        } else {
            self.sprite.transform =
                Matrix4::<f32>::from_translation(vec3(self.xy.x * SCALE_TO_SCREEN, self.xy.y * SCALE_TO_SCREEN, 0.0));
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

impl Rock {
    fn xmove_forward(&mut self, rate: f32) { }
    fn move_forward(&mut self, rate: f32) {
        self.sprite.forward = Matrix4::<f32>::from_translation(vec3(self.dir.x * SPEED * rate, self.dir.y * SPEED * rate, 0.0));
        self.sprite.transform = self.sprite.transform * self.sprite.forward;

        self.xy += self.dir * rate * FORWARD;

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
        }

    }
}
