use crate::room::*;
use rand;
use std::collections::HashMap;

pub struct Map {
    occupied: Vec<bool>,
    rooms: Vec<(Room, (usize, usize))>,
    hallways: Vec<(usize, usize)>,
    width: usize,
    height: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            occupied: vec![false; width * height],
            rooms: vec![],
            hallways: vec![],
            width,
            height,
        }
    }

    pub fn generate(&mut self, rooms: &HashMap<RoomKind, Room>) {
        let rng = rand::thread_rng();

        // place security room
        let (x, y) = (self.width / 2, self.height / 2);
    }
}
