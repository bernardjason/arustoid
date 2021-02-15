
pub struct Collision {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl Collision {
    pub fn new(x: f32, y: f32, radius: f32) -> Collision {
        Collision {
            x,
            y,
            radius,
        }
    }
    pub fn collide(&self, other: &Collision) -> bool {

        let square_dist = self.squared_distance(other.x,other.y);

        //println!("{},{} {},{} {}  ",self.x as i32,self.y as i32 , other.x as i32,other.y as i32,dist);
        if square_dist < self.radius * self.radius + other.radius * other.radius { //} + other.radius {
            //println!("HIT {} {} or {}  xy={},{}  and xy={},{}", square_dist as i32, self.radius, other.radius, self.x, self.y, other.x, other.y);
            true
        } else {
            false
        }
    }
    fn squared_distance(&self, x2: f32, y2: f32) -> f32 {
        let x = x2 - self.x;
        let y = y2 - self.y;
        return x * x + y * y;
    }
}

