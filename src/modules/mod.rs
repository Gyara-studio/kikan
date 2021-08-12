pub struct ModId {}
pub enum ModState {
    Ready,
    Offline,
    NotReady,
}

pub trait Module {
    type Action;
    type ModUpdate;

    /// return mod id. should never be same on two mod.
    fn mod_id(&self) -> ModId;

    /// return mod score.
    fn score(&self) -> u32;

    /// ask mod to do something.
    fn act(&mut self, action: Self::Action) -> anyhow::Result<()>;

    /// get mod state [ModState].
    fn state(&self) -> ModState;

    /// gather update and send to remote.
    fn gather_update(&mut self) -> Self::ModUpdate;

    /// deal update from remote.
    fn recv_update(&mut self, update: Self::ModUpdate) -> anyhow::Result<()>;
}
