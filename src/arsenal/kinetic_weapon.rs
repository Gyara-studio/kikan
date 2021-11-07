use crate::{
    error::KResult,
    kikan::{Kikan, Position, UnitId},
};
use std::num::NonZeroUsize;

use super::{Commit, UnitAction};

pub struct KineticWeaponCommit {
    pub(crate) delay: Box<dyn Fn(usize) -> usize + Sync + Send>,
    pub distance: usize,
    pub target: Position,
    pub damage: u32,
}

impl Commit for KineticWeaponCommit {
    fn resolve_at(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.delay)(self.distance)).unwrap_or_else(|| unsafe { NonZeroUsize::new_unchecked(1) })
    }

    fn take_commit(&self, _kikan: &mut Kikan) -> KResult<()> {
        Ok(())
    }

    /// this mod will use no unit id.
    fn fill_unit_id(&mut self, _: UnitId) {}
}

impl UnitAction for Position {}
