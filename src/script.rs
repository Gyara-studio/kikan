use crate::kikan::{Kikan, Move, UnitId};
use mlua::{prelude::LuaError, Lua};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Lua error: {0}")]
    LuaError(#[from] LuaError),
}

pub fn load_lua_script<T, F: 'static>(
    kikan: Arc<Mutex<Kikan>>,
    lua_script: T,
    mut start_pos: F,
) -> Result<(), ScriptError>
where
    T: AsRef<str>,
    F: FnMut(UnitId) -> (i32, i32),
{
    let lua = Lua::new();

    // library
    // init unit
    let kikan_temp = Arc::clone(&kikan);
    let mut init_flag = false;
    let init_unit = lua.create_function_mut(move |_, _: ()| {
        if init_flag {
            return Err(LuaError::RuntimeError("can only init once".to_string()));
        }
        init_flag = true;
        let id = kikan_temp.lock().unwrap().add_unit(start_pos(0));
        Ok(id)
    })?;
    lua.globals().set("init_unit", init_unit)?;

    // move to
    let kikan_temp = Arc::clone(&kikan);
    let move_unit = lua.create_function(move |_, (id, direction): (UnitId, String)| {
        let next_move: Move = direction.parse().unwrap();
        kikan_temp.lock().unwrap().plan_unit_move(id, next_move).unwrap();
        Ok(())
    })?;
    lua.globals().set("move_unit", move_unit)?;

    // run script
    let script = lua.load(lua_script.as_ref());
    script.exec()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn hello_world() {
        let kikan = Kikan::kikan_in_a_shell();
        load_lua_script(Arc::clone(&kikan), "init_unit()", |_| (0, 0)).unwrap();
        assert!(kikan.lock().unwrap().get_unit_position(0).is_some());
    }

    #[test]
    fn start_pos() {
        let count = Arc::new(AtomicI32::new(0));
        let kikan = Kikan::kikan_in_a_shell();
        let len = 10;
        for _ in 0..len {
            let count = Arc::clone(&count);
            let start_pos = move |_| {
                let mut count_t = count.load(Ordering::Acquire);
                count_t += 1;
                count.store(count_t, Ordering::Release);
                (count_t, 0)
            };
            load_lua_script(Arc::clone(&kikan), "init_unit()", start_pos).unwrap();
        }
        for i in 0..len {
            assert_eq!(kikan.lock().unwrap().get_unit_position(i), Some((i as i32 + 1, 0)));
        }
    }

    #[test]
    fn unit_move() {
        let script = "
            local id = init_unit();
            move_unit(id, 'N');
        ";
        let kikan = Kikan::kikan_in_a_shell();
        load_lua_script(Arc::clone(&kikan), script, |_| (0, 0)).unwrap();
        kikan.lock().unwrap().apply_move();
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some((1, 0)));
    }
}
