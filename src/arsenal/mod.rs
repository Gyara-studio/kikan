use crate::error::{KResult, KikanError};
use std::collections::HashMap;

pub mod coremods;

pub type UnitScore = u32;

pub trait UnitPart: Sized {
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
}

pub trait UnitMod<A: UnitAction>: UnitPart {
    fn status(&mut self) -> KResult<UnitStatus>;

    fn action(&mut self, action: A) -> KResult<()>;

    fn action_done(&mut self) -> KResult<()>;

    fn mark_as_offline(&mut self) -> KResult<()>;
}

pub trait UnitAction: Clone + Send + Sync {}
