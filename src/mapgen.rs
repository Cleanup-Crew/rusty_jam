use crate::room::*;
use rand;
use rand::Rng;
use std::collections::HashMap;

pub struct Map {
    occupied: Vec<bool>,
    pub rooms: Vec<(RoomKind, (usize, usize))>,
    pub hallways: Vec<(usize, usize)>,
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
        let mut rng = rand::thread_rng();

        // place security room
        let security_room = &rooms[&RoomKind::Security];
        let (sec_x, sec_y) = (
            self.width / 2 - security_room.width / 2,
            self.height / 2 - security_room.height / 2,
        );
        self.rooms.push((RoomKind::Security, (sec_x, sec_y)));
        for y in sec_y..security_room.height + sec_y {
            for x in sec_x..security_room.width + sec_x {
                self.occupied[y * self.width + x] = true;
            }
        }

        // place random rooms
        for _ in 0..8 {
            // pick room (only one option for now)
            let room = &rooms[&RoomKind::Empty];

            let room_x = rng.gen_range(0..self.width - room.width + 1);
            let room_y = rng.gen_range(0..self.height - room.height + 1);

            let mut blocked = false;
            for y in room_y..room.height + room_y {
                for x in room_x..room.width + room_x {
                    if self.occupied[y * self.width + x] {
                        blocked = true;
                    }
                }
            }

            if blocked {
                continue;
            }
            self.rooms.push((RoomKind::Empty, (room_x, room_y)));
            for y in room_y..room.height + room_y {
                for x in room_x..room.width + room_x {
                    self.occupied[y * self.width + x] = true;
                }
            }
        }

        // place hallways
        let doors = self
            .rooms
            .iter()
            .map(|(room, (x, y))| {
                let room = &rooms[room];

                // Calculate actual door position by adding the relative position the the actual
                // piont the room is placed at
                room.doors
                    .iter()
                    .map(move |(dx, dy, dir)| (dx + x, dy + y, dir))
            })
            .flatten();

        for (dx, dy, dir) in doors {
            match dir {
                crate::room::Direction::North => self.hallways.push((dx, dy + 1)),
                crate::room::Direction::East => self.hallways.push((dx + 1, dy)),
                crate::room::Direction::South => self.hallways.push((dx, dy - 1)),
                crate::room::Direction::West => self.hallways.push((dx - 1, dy)),
            }
        }
    }
}
