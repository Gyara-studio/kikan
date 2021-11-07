use mlua::UserData;

use crate::{
    error::{KResult, KikanError},
    kikan::{Kikan, Position, UnitId},
};
use std::{collections::HashMap, num::NonZeroUsize};

pub mod engine;
pub mod kinetic_weapon;

pub type UnitScore = u32;

pub trait UnitPart {
    fn score(&self) -> UnitScore;
}

impl<T> UnitPart for Vec<T>
where
    T: UnitPart,
{
    fn score(&self) -> UnitScore {
        self.iter().map(|unit| unit.score()).sum()
    }
}

impl<U, T> UnitPart for HashMap<U, T>
where
    T: UnitPart,
{
    fn score(&self) -> UnitScore {
        self.iter().map(|(_, unit)| unit.score()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitStatus {
    Busy,
    Offline,
    Operational,
}

impl UnitStatus {
    pub fn is_operation(&self) -> bool {
        matches!(self, Self::Operational)
    }

    pub fn operational_or_err(self) -> KResult<()> {
        if let UnitStatus::Operational = self {
            Ok(())
        } else if let UnitStatus::Offline = self {
            Err(KikanError::ModOffline)
        } else {
            Err(KikanError::ModBusy)
        }
    }

    pub fn online_or_err(self) -> KResult<()> {
        if let UnitStatus::Offline = self {
            Err(KikanError::ModOffline)
        } else {
            Ok(())
        }
    }

    /// Returns `true` if the unit status is [`Busy`].
    ///
    /// [`Busy`]: UnitStatus::Busy
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Busy)
    }
}

pub trait UnitMod<A: UnitAction>: UnitPart + Send {
    fn status(&self) -> KResult<UnitStatus>;

    fn action(&mut self, action: A) -> KResult<Box<dyn Commit>>;

    fn action_done(&mut self) -> KResult<()>;

    fn mark_as_offline(&mut self) -> KResult<()>;
}

pub trait UnitAction: Clone + Send + Sync {}

pub trait Commit: Send + Sync {
    fn resolve_at(&self) -> NonZeroUsize;
    fn fill_unit_id(&mut self, id: UnitId);
    fn take_commit(&self, kikan: &mut Kikan) -> KResult<()>;
}

impl Commit for () {
    /// # Safety
    /// 1 is NonZeroU8
    fn resolve_at(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(1) }
    }

    fn take_commit(&self, _kikan: &mut Kikan) -> KResult<()> {
        Ok(())
    }

    fn fill_unit_id(&mut self, _: UnitId) {}
}

pub enum UnitModContainter {
    KineticWeapon(Box<dyn UnitMod<Position>>),
}

impl UnitModContainter {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::KineticWeapon(_) => "KineticWeapon",
        }
    }

    pub fn take_action(&mut self, action: UnitActionContainer) -> KResult<Box<dyn Commit>> {
        match self {
            Self::KineticWeapon(umod) => match action {
                UnitActionContainer::Pos(action) => umod.action(action),
                // _ => Err(KikanError::WrongUnitArgs(self.type_name().to_string())),
            },
        }
    }
}

impl UnitPart for UnitModContainter {
    fn score(&self) -> UnitScore {
        match self {
            Self::KineticWeapon(umod) => umod.score(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum UnitActionContainer {
    Pos(Position),
}

impl UnitActionContainer {
    pub fn from_position(pos: Position, umod: &UnitModContainter) -> KResult<Self> {
        match umod {
            UnitModContainter::KineticWeapon(_) => Ok(Self::Pos(pos)),
        }
    }
}

impl UserData for UnitActionContainer {}
