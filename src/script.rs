use crate::{error::KikanError, kikan::UnitHandler};
use mlua::{Lua, UserData};

pub fn load_lua_script<T, H>(handler: H, lua_script: T) -> Result<(), KikanError>
where
    T: AsRef<str>,
    H: 'static + UnitHandler + UserData,
{
    let lua = Lua::new();
    // library
    // init unit
    lua.globals().set("api", handler)?;

    // run script
    let script = lua.load(lua_script.as_ref());
    script.exec()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        handler::LocalHandle,
        kikan::{Kikan, Position},
    };
    use std::sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    };

    #[test]
    fn hello_world() {
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        load_lua_script(handler, "api:init()").unwrap();
        assert!(kikan.lock().unwrap().get_unit_position(0).is_some());
    }

    #[test]
    fn start_pos() {
        let count = Arc::new(AtomicI32::new(0));
        let count_s = Arc::clone(&count);
        let start_pos = move || {
            let mut count_t = count_s.load(Ordering::Acquire);
            count_t += 1;
            count_s.store(count_t, Ordering::Release);
            Position(count_t, 0)
        };
        let kikan = Kikan::kikan_in_a_shell(start_pos);
        let len = 0;
        for _ in 0..len {
        let handler = LocalHandle::new(Arc::clone(&kikan));
            load_lua_script(handler, "api:init();api:move('E');").unwrap();
        }
        for i in 0..len {
            assert_eq!(
                kikan.lock().unwrap().get_unit_position(i),
                Some(Position(i as i32 + 1, 1))
            );
        }
    }

    #[test]
    fn unit_move() {
        let script = "
            api:init();
            api:plan_move('N');
            api:plan_move('N');
            api:plan_move('E');
            api:plan_move('E');
        ";
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        load_lua_script(handler, script).unwrap();
        for _ in 0..4 {
            kikan.lock().unwrap().apply_move();
        }
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some(Position(2, 2)));
    }
}
