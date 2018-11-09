pub mod app;
pub mod map;



trait Drawable {
    fn get_entities(&self) -> Vec<[f64;4]>;
    fn get_world_size(&self) -> [f64;2];
}

