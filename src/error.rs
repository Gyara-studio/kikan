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
    #[error("Missing unit part `{0}`")]
    MissingUnitPart(&'static str),
    #[error("Unit mod not found `{0}`")]
    MissingUnitMod(String),
    #[error("Wrong Unit args `{0}`")]
    WrongUnitArgs(String),
}

impl From<KikanError> for LuaError {
    fn from(e: KikanError) -> Self {
        Self::RuntimeError(e.to_string())
    }
}

pub type KResult<T> = Result<T, KikanError>;
