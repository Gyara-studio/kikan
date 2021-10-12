use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy)]
pub enum Move {
    N, // ↑
    S, // ↓
    W, // ←
    E, // →
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "N" | "n" => Self::N,
            "S" | "s" => Self::S,
            "W" | "w" => Self::W,
            "E" | "e" => Self::E,
            _ => return Err(()),
        })
    }
}

/// pos:
/// ↑
/// x
/// 0 y →
#[derive(Debug, Clone)]
pub(crate) struct Unit {
    pub(crate) pos: (i32, i32),
    pub(crate) move_queue: Vec<Move>,
}

pub type UnitId = u32;

impl Unit {
    pub fn new(pos: (i32, i32)) -> Self {
        Self {
            pos,
            move_queue: Vec::new(),
        }
    }

    fn plan_move(&mut self) -> (i32, i32) {
        if !self.move_queue.is_empty() {
            let next_move = self.move_queue.remove(0);
            let (x, y) = self.pos;
            return match next_move {
                Move::N => (x + 1, y),
                Move::S => (x - 1, y),
                Move::W => (x, y - 1),
                Move::E => (x, y + 1),
            };
        }
        self.pos
    }

    fn apply_move(&mut self, new_pos: (i32, i32)) {
        self.pos = new_pos;
    }
}

#[derive(Debug, Default)]
pub struct Kikan {
    count: UnitId,
    units: HashMap<UnitId, Unit>,
}

impl Kikan {
    pub fn kikan_in_a_shell() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn add_unit(&mut self, pos: (i32, i32)) -> UnitId {
        let id = self.count;
        self.count += 1;
        let unit = Unit::new(pos);
        self.units.insert(id, unit);
        id
    }

    pub fn plan_unit_move(&mut self, unit_id: UnitId, next_move: Move) -> Option<()> {
        let unit = self.units.get_mut(&unit_id)?;
        unit.move_queue.push(next_move);
        Some(())
    }

    pub fn get_unit_position(&mut self, unit_id: UnitId) -> Option<(i32, i32)> {
        let unit = self.units.get(&unit_id)?;
        Some(unit.pos)
    }

    pub fn apply_move(&mut self) {
        // pos to id
        let mut new_pos: HashMap<(i32, i32), UnitId> = HashMap::new();
        // which unit can be updated
        let mut new_pos_avaliable: HashSet<UnitId> = HashSet::new();

        for (id, unit) in self.units.iter_mut() {
            let next_pos = unit.plan_move();
            if let Some(another_unit) = new_pos.get(&next_pos) {
                new_pos_avaliable.remove(another_unit);
            } else {
                new_pos.insert(next_pos, *id);
                new_pos_avaliable.insert(*id);
            }
        }
        for (pos, id) in new_pos.into_iter() {
            if new_pos_avaliable.contains(&id) {
                self.units.get_mut(&id).expect("Ghost unit!").apply_move(pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_move() {
        let mut kikan = Kikan::default();
        let u0 = kikan.add_unit((0, 0));

        let m0 = Move::N;
        kikan.plan_unit_move(u0, m0).unwrap();
        kikan.apply_move();
        let pos0 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos0, (1, 0));

        let m1 = Move::E;
        kikan.plan_unit_move(u0, m1).unwrap();
        kikan.apply_move();
        let pos1 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos1, (1, 1));

        let m2 = Move::S;
        kikan.plan_unit_move(u0, m2).unwrap();
        kikan.apply_move();
        let pos2 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos2, (0, 1));

        let m3 = Move::W;
        kikan.plan_unit_move(u0, m3).unwrap();
        kikan.apply_move();
        let pos3 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos3, (0, 0));
    }

    #[test]
    fn unit_crash() {
        let mut kikan = Kikan::default();
        let u0 = kikan.add_unit((0, 0));
        let u1 = kikan.add_unit((0, 1));

        let m0_0 = Move::E;
        let m0_1 = Move::S;
        kikan.plan_unit_move(u0, m0_0).unwrap();
        kikan.plan_unit_move(u0, m0_1).unwrap();

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, (0, 0));
        assert_eq!(pos_u1, (0, 1));

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos_u0, (-1, 0));
    }

    #[test]
    fn unit_crash_2() {
        let mut kikan = Kikan::default();
        let u0 = kikan.add_unit((1, 0));
        let u1 = kikan.add_unit((0, 1));

        let m0_0 = Move::S;
        kikan.plan_unit_move(u0, m0_0).unwrap();

        let m1_0 = Move::W;
        kikan.plan_unit_move(u1, m1_0).unwrap();

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, (1, 0));
        assert_eq!(pos_u1, (0, 1));
    }
}
