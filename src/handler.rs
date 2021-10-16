use crate::{
    error::{KResult, KikanError},
    kikan::{Kikan, Move, Position, UnitId},
};
use mlua::{Error as LuaError, UserData};
use std::sync::{Arc, Mutex};

pub trait UnitHandler: Sized {
    // store id to handler
    fn init(&mut self) -> KResult<()>;
    fn get_position(&self) -> KResult<Position>;
    fn plan_move(&mut self, next_move: Move) -> KResult<()>;
    fn is_move_queue_empty(&self) -> KResult<bool>;
    fn clear_move_queue(&self) -> KResult<()>;
    fn package(self) -> Handler<Self> {
        Handler(self)
    }
}

pub struct LocalHandle {
    unit_id: Option<UnitId>,
    kikan: Arc<Mutex<Kikan>>,
}

impl LocalHandle {
    pub fn new(kikan: Arc<Mutex<Kikan>>) -> Self {
        Self { kikan, unit_id: None }
    }
}

impl UnitHandler for LocalHandle {
    fn init(&mut self) -> KResult<()> {
        if self.unit_id.is_some() {
            return Err(KikanError::AlreadyInited);
        }
        let mut kikan = self.kikan.lock().unwrap();
        let pos = kikan.gen_start_pos();
        let id = kikan.add_unit(pos)?;
        self.unit_id = Some(id);
        Ok(())
    }

    fn get_position(&self) -> KResult<Position> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan
            .lock()
            .unwrap()
            .get_unit_position(id)
            .ok_or(KikanError::GhostUnit)
    }

    fn plan_move(&mut self, next_move: Move) -> KResult<()> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan.lock().unwrap().plan_unit_move(id, next_move)
    }

    fn is_move_queue_empty(&self) -> KResult<bool> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan.lock().unwrap().is_unit_move_queue_empty(id)
    }

    fn clear_move_queue(&self) -> KResult<()> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan.lock().unwrap().clear_unit_move_queue(id)
    }
}

pub struct Handler<T>(pub T);

impl<T> UserData for Handler<T>
where
    T: UnitHandler,
{
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("pos", |_, this| {
            this.0.get_position().map_err(|e| LuaError::RuntimeError(e.to_string()))
        })
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("init", |_, this, _: ()| {
            this.0.init().map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method_mut("plan_move", |_, this, next_move: String| {
            let next_move: Move = next_move
                .parse()
                .map_err(|_| LuaError::RuntimeError("Invalid arg".to_string()))?;
            this.0
                .plan_move(next_move)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method("get_position", |_, this, _: ()| {
            this.0.get_position().map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method("is_any_planned_move", |_, this, (): ()| {
            this.0
                .is_move_queue_empty()
                .map(|re| !re)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method("clear_planned_move", |_, this, (): ()| {
            this.0
                .clear_move_queue()
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    }
}
