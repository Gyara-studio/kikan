use mlua::UserData;

use crate::error::{KResult, KikanError};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(pub i32, pub i32);

impl UserData for Position {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0));
        fields.add_field_method_get("y", |_, this| Ok(this.1));
    }
}

/// pos:
/// ↑
/// x
/// 0 y →
#[derive(Debug, Clone)]
pub(crate) struct Unit {
    pub(crate) pos: Position,
    pub(crate) move_queue: Vec<Move>,
}

pub type UnitId = u32;

impl Unit {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            move_queue: Vec::new(),
        }
    }

    fn plan_move(&mut self) -> Position {
        if !self.move_queue.is_empty() {
            let next_move = self.move_queue.remove(0);
            let Position(x, y) = self.pos;
            return match next_move {
                Move::N => Position(x + 1, y),
                Move::S => Position(x - 1, y),
                Move::W => Position(x, y - 1),
                Move::E => Position(x, y + 1),
            };
        }
        self.pos
    }

    fn apply_move(&mut self, new_pos: Position) {
        self.pos = new_pos;
    }
}

pub struct PosConfig {
    pub number: u32,
}

pub struct Kikan {
    count: UnitId,
    units: HashMap<UnitId, Unit>,
    start_pos: Box<dyn FnMut() -> Position>,
}

impl Kikan {
    pub fn kikan_in_a_shell<F>(start_pos: F) -> Arc<Mutex<Self>>
    where
        F: FnMut() -> Position + 'static,
    {
        let kikan = Kikan {
            count: 0,
            units: HashMap::new(),
            start_pos: Box::new(start_pos),
        };
        Arc::new(Mutex::new(kikan))
    }

    pub fn add_unit(&mut self, pos: Position) -> KResult<UnitId> {
        if self.units.iter().any(|(_, v)| v.pos == pos) {
            return Err(KikanError::AlreadyUnitHere);
        }
        let id = self.count;
        self.count += 1;
        let unit = Unit::new(pos);
        self.units.insert(id, unit);
        Ok(id)
    }

    pub fn plan_unit_move(&mut self, unit_id: UnitId, next_move: Move) -> KResult<()> {
        let unit = self.units.get_mut(&unit_id).ok_or(KikanError::GhostUnit)?;
        unit.move_queue.push(next_move);
        Ok(())
    }

    pub fn get_unit_position(&self, unit_id: UnitId) -> Option<Position> {
        let unit = self.units.get(&unit_id)?;
        Some(unit.pos)
    }

    pub fn apply_move(&mut self) {
        // pos to id
        let mut new_pos: HashMap<Position, UnitId> = HashMap::new();
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

    pub fn gen_start_pos(&mut self) -> Position {
        (self.start_pos)()
    }

    pub fn is_unit_move_queue_empty(&self, unit_id: UnitId) -> KResult<bool> {
        Ok(self
            .units
            .get(&unit_id)
            .ok_or(KikanError::GhostUnit)?
            .move_queue
            .is_empty())
    }

    pub fn clear_unit_move_queue(&mut self, unit_id: UnitId) -> KResult<()> {
        let unit = self.units.get_mut(&unit_id).ok_or(KikanError::GhostUnit)?;
        unit.move_queue.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kikan() -> Kikan {
        Kikan {
            count: 0,
            units: HashMap::new(),
            start_pos: Box::new(|| Position(0, 0)),
        }
    }

    #[test]
    fn unit_move() {
        let mut kikan = test_kikan();
        let u0 = kikan.add_unit(Position(0, 0)).unwrap();

        let m0 = Move::N;
        kikan.plan_unit_move(u0, m0).unwrap();
        kikan.apply_move();
        let pos0 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos0, Position(1, 0));

        let m1 = Move::E;
        kikan.plan_unit_move(u0, m1).unwrap();
        kikan.apply_move();
        let pos1 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos1, Position(1, 1));

        let m2 = Move::S;
        kikan.plan_unit_move(u0, m2).unwrap();
        kikan.apply_move();
        let pos2 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos2, Position(0, 1));

        let m3 = Move::W;
        kikan.plan_unit_move(u0, m3).unwrap();
        kikan.apply_move();
        let pos3 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos3, Position(0, 0));
    }

    #[test]
    fn unit_crash() {
        let mut kikan = test_kikan();
        let u0 = kikan.add_unit(Position(0, 0)).unwrap();
        let u1 = kikan.add_unit(Position(0, 1)).unwrap();

        let m0_0 = Move::E;
        let m0_1 = Move::S;
        kikan.plan_unit_move(u0, m0_0).unwrap();
        kikan.plan_unit_move(u0, m0_1).unwrap();

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, Position(0, 0));
        assert_eq!(pos_u1, Position(0, 1));

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos_u0, Position(-1, 0));
    }

    #[test]
    fn unit_crash_2() {
        let mut kikan = test_kikan();
        let u0 = kikan.add_unit(Position(1, 0)).unwrap();
        let u1 = kikan.add_unit(Position(0, 1)).unwrap();

        let m0_0 = Move::S;
        kikan.plan_unit_move(u0, m0_0).unwrap();

        let m1_0 = Move::W;
        kikan.plan_unit_move(u1, m1_0).unwrap();

        kikan.apply_move();
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, Position(1, 0));
        assert_eq!(pos_u1, Position(0, 1));
    }
}
