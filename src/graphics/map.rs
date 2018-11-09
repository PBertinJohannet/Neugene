pub const WIDTH: i32 = 150;
pub const HEIGHT: i32 = 150;
use rand::Rng;


pub type Map = [[u32; HEIGHT as usize]; WIDTH as usize];

// `MapTrait` provide a safe and convenient way to
// access `Map`
pub trait MapTrait {
    fn get(&self, x : i32, y : i32) -> Option<u32>;
    fn set(&mut self, x : i32, y : i32, val : u32);
    fn update(&mut self);
    fn copy_from(&mut self, s : &Map);
}


pub fn zerotable() -> Map {
    [[0; HEIGHT as usize]; WIDTH as usize]
}

pub fn randomtable() -> Map {
    let mut r = rand::thread_rng();
    let mut ret = zerotable();
    for i in 0..WIDTH as usize { for j in 0..HEIGHT as usize {
        ret[i][j] = r.gen::<u32>() & 1;
    }}
    ret
}

fn check(x: i32, y: i32) -> bool {
    if x<0 || x>=WIDTH || y<0 || y>=HEIGHT {
        return false;
    }
    true
}

impl MapTrait for Map {
    fn get(&self, x: i32, y: i32) -> Option<u32> {
        if check(x, y) {
            return Some(self[x as usize][y as usize]);
        }
        None
    }

    fn set(&mut self, x: i32, y: i32, val: u32) {
        if check(x, y) {
            self[x as usize][y as usize] = val;
        }
    }

    fn update(&mut self) {
        // `nei` contains the number of neighbours of each cell
        let mut nei = zerotable();
        // calculate `nei`
        for i in 0..WIDTH { for j in 0..HEIGHT {
            for x in i-1..i+2 { for y in j-1..j+2 {
                match self.get(x, y) {
                    Some(u) => nei[i as usize][j as usize] += u,
                    None => (),
                }
            }}
            nei[i as usize][j as usize] -= self.get(i, j).unwrap();
        }}

        // update `self` with game of life's rules
        for i in 0..WIDTH as usize { for j in 0..HEIGHT as usize {
            if self[i][j] == 1 {
                if nei[i][j] < 2 || nei[i][j] > 3 {
                    self[i][j] = 0
                }
            } else if nei[i][j] == 3 {
                self[i][j] = 1;
            }
        }}
    }

    fn copy_from(&mut self, s: &Map) {
        for i in 0..WIDTH as usize { for j in 0..HEIGHT as usize {
            self[i][j] = s[i][j]
        }}
    }
}