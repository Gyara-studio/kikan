use crate::{
    arsenal::{Commit, UnitAction, UnitMod, UnitPart, UnitScore, UnitStatus},
    error::{KResult, KikanError},
    kikan::{Position, UnitId},
};
use std::{num::NonZeroUsize, str::FromStr};

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

impl UnitAction for Move {}

pub struct MoveCommit {
    resolve_delay: NonZeroUsize,
    unit_id: Option<UnitId>,
    next_move: Move,
}

impl MoveCommit {
    fn new(resolve_delay: usize, next_move: Move) -> Self {
        let resolve_delay = NonZeroUsize::new(resolve_delay).unwrap();
        Self {
            resolve_delay,
            next_move,
            unit_id: None,
        }
    }
}

impl Commit for MoveCommit {
    fn resolve_at(&self) -> NonZeroUsize {
        self.resolve_delay
    }

    fn fill_unit_id(&mut self, id: UnitId) {
        self.unit_id = Some(id);
    }

    fn take_commit(&self, kikan: &mut crate::kikan::Kikan) -> KResult<()> {
        let unit_id = self.unit_id.unwrap();
        let Position(x, y) = kikan.get_unit_position(unit_id).ok_or(KikanError::GhostUnit)?;
        let pos = match self.next_move {
            Move::N => Position(x + 1, y),
            Move::S => Position(x - 1, y),
            Move::W => Position(x, y - 1),
            Move::E => Position(x, y + 1),
        };
        kikan.commit_move(unit_id, pos);
        let unit = kikan.get_unit_by_id(unit_id)?;
        unit.engine.action_done()
    }
}

/// engine
#[derive(Debug, Default, Clone, Copy)]
pub struct STE0 {
    now_on: Option<Move>,
    offline: bool,
}

impl UnitPart for STE0 {
    fn score(&self) -> UnitScore {
        0
    }
}

impl UnitMod<Move> for STE0 {
    fn status(&self) -> KResult<UnitStatus> {
        if self.offline {
            Ok(UnitStatus::Offline)
        } else if self.now_on.is_some() {
            Ok(UnitStatus::Busy)
        } else {
            Ok(UnitStatus::Operational)
        }
    }

    fn action(&mut self, action: Move) -> KResult<Box<dyn Commit>> {
        self.status()?.operational_or_err()?;
        self.now_on = Some(action);
        let commit = MoveCommit::new(10, action);
        Ok(Box::new(commit))
    }

    fn action_done(&mut self) -> KResult<()> {
        self.status()?.online_or_err()?;
        self.now_on = None;
        Ok(())
    }

    fn mark_as_offline(&mut self) -> KResult<()> {
        self.status()?.online_or_err()?;
        self.now_on = None;
        self.offline = true;
        Ok(())
    }
}
