use mlua::Error as LuaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KikanError {
    #[error("Lua error: {0}")]
    LuaError(#[from] LuaError),
    #[error("One unit can only init once")]
    AlreadyInited,
    #[error("Unit not been inited")]
    Uninited,
    #[error("Ghost unit")]
    GhostUnit,
    #[error("There is already a unit")]
    AlreadyUnitHere,
    #[error("This mod is busy")]
    ModBusy,
    #[error("This mod is offline")]
    ModOffline,
}

impl From<KikanError> for LuaError {
    fn from(e: KikanError) -> Self {
        Self::RuntimeError(e.to_string())
    }
}

pub type KResult<T> = Result<T, KikanError>;
