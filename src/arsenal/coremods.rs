pub mod engine {
    use crate::{
        arsenal::{UnitAction, UnitMod, UnitPart, UnitScore, UnitStatus},
        error::KResult,
    };
    use std::str::FromStr;

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

    /// engine
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
        fn status(&mut self) -> KResult<UnitStatus> {
            if self.offline {
                Ok(UnitStatus::Offline)
            } else if self.now_on.is_some() {
                Ok(UnitStatus::Busy)
            } else {
                Ok(UnitStatus::Operational)
            }
        }

        fn action(&mut self, action: Move) -> KResult<()> {
            self.status()?.operational_or_err()?;
            self.now_on = Some(action);
            Ok(())
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
}
