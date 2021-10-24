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
    fn plan_move(&self, next_move: Move) -> KResult<()>;
    fn is_moving(&self) -> KResult<bool>;
    fn package(self) -> Handler<Self> {
        Handler(self)
    }
    fn wait_for_update(&self);
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

    fn plan_move(&self, next_move: Move) -> KResult<()> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan.lock().unwrap().plan_unit_move(id, next_move)
    }

    fn is_moving(&self) -> KResult<bool> {
        let id = if let Some(id) = self.unit_id {
            id
        } else {
            return Err(KikanError::Uninited);
        };
        self.kikan.lock().unwrap().is_unit_moving(id)
    }

    fn wait_for_update(&self) {
        let mut reader = { self.kikan.lock().unwrap().wait_for_update() };
        reader.recv().ok();
    }
}

pub struct Handler<T>(pub T);

impl<T> UserData for Handler<T>
where
    T: UnitHandler,
{
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("pos", |_, this| Ok(this.0.get_position()?))
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("init", |_, this, _: ()| Ok(this.0.init()?));

        methods.add_method_mut("plan_move", |_, this, next_move: String| {
            let next_move: Move = next_move
                .parse()
                .map_err(|_| LuaError::RuntimeError("Invalid arg".to_string()))?;
            Ok(this.0.plan_move(next_move)?)
        });

        methods.add_method("get_position", |_, this, _: ()| Ok(this.0.get_position()?));

        methods.add_method("is_moving", |_, this, (): ()| {
            Ok(this.0.is_moving().map(|re| !re)?)
        });

        methods.add_method("wait_for_update", |_, this, (): ()| {
            this.0.wait_for_update();
            Ok(())
        });
    }
}
