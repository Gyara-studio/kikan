pub use crate::arsenal::engine::Move;
use crate::{
    arsenal::{engine::STE0, Commit, UnitMod},
    error::{KResult, KikanError},
};
use bus::{Bus, BusReader};
use mlua::UserData;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};

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
pub struct Unit {
    pub(crate) pos: Position,
    pub(crate) engine: Box<dyn UnitMod<Move> + Send>,
}

pub type UnitId = u32;

impl Unit {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            engine: Box::new(STE0::default()),
        }
    }

    fn plan_move(&mut self, next_move: Move) -> KResult<Box<dyn Commit>> {
        self.engine.status()?.operational_or_err()?;
        self.engine.action(next_move)
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
    commits: VecDeque<Vec<Box<dyn Commit>>>,
    move_commits: HashMap<UnitId, Position>,
    start_pos: Box<dyn Fn() -> Position + Send>,
    update_bus: Bus<()>,
}

impl Kikan {
    pub fn kikan_in_a_shell<F>(start_pos: F) -> Arc<Mutex<Self>>
    where
        F: Fn() -> Position + 'static + Send,
    {
        let kikan = Kikan {
            count: 0,
            units: HashMap::new(),
            commits: VecDeque::with_capacity(1024),
            move_commits: HashMap::default(),
            start_pos: Box::new(start_pos),
            update_bus: Bus::new(42), // every thing
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
        let mut commit = unit.plan_move(next_move)?;
        commit.fill_unit_id(unit_id);
        self.add_commit(commit);
        Ok(())
    }

    pub fn get_unit_position(&self, unit_id: UnitId) -> Option<Position> {
        let unit = self.units.get(&unit_id)?;
        Some(unit.pos)
    }

    fn apply_move(&mut self) {
        // pos to id
        let mut new_pos: HashMap<Position, UnitId> = HashMap::new();
        // which unit can be updated
        let mut new_pos_avaliable: HashSet<UnitId> = HashSet::new();

        for id in self.units.keys() {
            let now_pos = self.get_unit_position(*id).unwrap();
            let next_pos = self.move_commits.remove(id).unwrap_or(now_pos);
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

    pub fn is_unit_moving(&self, id: UnitId) -> KResult<bool> {
        let unit = self.units.get(&id).ok_or(KikanError::GhostUnit)?;
        unit.engine.status().map(|st| st.is_busy())
    }

    pub fn wait_for_update(&mut self) -> BusReader<()> {
        self.update_bus.add_rx()
    }

    pub fn update(&mut self) -> KResult<()> {
        let mut res = Vec::new();
        if let Some(commits) = self.commits.pop_front() {
            for commit in commits {
                res.push(commit.take_commit(self));
            }
        };
        self.apply_move();
        self.update_bus.broadcast(());
        res.into_iter().collect()
    }

    pub fn get_unit_by_id(&mut self, id: UnitId) -> KResult<&mut Unit> {
        self.units.get_mut(&id).ok_or(KikanError::GhostUnit)
    }

    pub fn commit_move(&mut self, id: UnitId, pos: Position) {
        self.move_commits.insert(id, pos);
    }

    pub fn add_commit(&mut self, commit: Box<dyn Commit>) {
        let at: usize = commit.resolve_at().get();
        let seat = if let Some(seat) = self.commits.get_mut(at) {
            seat
        } else {
            if self.commits.len() <= at {
                self.commits.resize_with(at + 1, Default::default)
            }
            let seat = Vec::new();
            self.commits.insert(at, seat);
            self.commits.get_mut(at).unwrap()
        };
        seat.push(commit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kikan() -> Kikan {
        Kikan {
            count: 0,
            units: HashMap::default(),
            commits: VecDeque::default(),
            move_commits: HashMap::default(),
            start_pos: Box::new(|| Position(0, 0)),
            update_bus: Bus::new(42),
        }
    }

    #[test]
    fn unit_move() {
        let mut kikan = test_kikan();
        let u0 = kikan.add_unit(Position(0, 0)).unwrap();

        let m0 = Move::N;
        kikan.plan_unit_move(u0, m0).unwrap();
        for _ in 0..20 {
            kikan.update().unwrap();
        }
        let pos0 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos0, Position(1, 0));

        let m1 = Move::E;
        kikan.plan_unit_move(u0, m1).unwrap();
        for _ in 0..20 {
            kikan.update().unwrap();
        }
        let pos1 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos1, Position(1, 1));

        let m2 = Move::S;
        kikan.plan_unit_move(u0, m2).unwrap();
        for _ in 0..20 {
            kikan.update().unwrap();
        }
        let pos2 = kikan.get_unit_position(u0).unwrap();
        assert_eq!(pos2, Position(0, 1));

        let m3 = Move::W;
        kikan.plan_unit_move(u0, m3).unwrap();
        for _ in 0..20 {
            kikan.update().unwrap();
        }
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

        for _ in 0..20 {
            kikan.update().unwrap();
        }
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, Position(0, 0));
        assert_eq!(pos_u1, Position(0, 1));

        kikan.plan_unit_move(u0, m0_1).unwrap();
        for _ in 0..20 {
            kikan.update().unwrap();
        }
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

        for _ in 0..20 {
            kikan.update().unwrap();
        }
        let pos_u0 = kikan.get_unit_position(u0).unwrap();
        let pos_u1 = kikan.get_unit_position(u1).unwrap();

        assert_eq!(pos_u0, Position(1, 0));
        assert_eq!(pos_u1, Position(0, 1));
    }

    #[test]
    fn bus_buffer() {
        let mut kikan = test_kikan();
        for _ in 0..45 {
            kikan.update().unwrap();
        }
    }
}
