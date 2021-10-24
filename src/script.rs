use crate::{error::KikanError, handler::UnitHandler};
use mlua::Lua;

mod utils {
    use crate::kikan::Position;
    use mlua::UserData;

    pub struct Utils {}
    impl UserData for Utils {
        fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}

        fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
            methods.add_method("new_position", |_, _, (x, y): (i32, i32)| Ok(Position(x, y)))
        }
    }
}

pub fn load_lua_script<T, H>(handler: H, lua_script: T) -> Result<(), KikanError>
where
    T: AsRef<str>,
    H: 'static + UnitHandler,
{
    let lua = Lua::new();
    // library
    // init unit
    let handler = handler.package();
    lua.globals().set("api", handler)?;

    // insert help functions
    let utils = utils::Utils {};
    lua.globals().set("utils", utils)?;

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
            api:wait_for_update();
            api:plan_move('N');
            api:wait_for_update();
            api:plan_move('E');
            api:wait_for_update();
            api:plan_move('E');
            api:wait_for_update();
        ";
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        let _unit = {
            let kikan = Arc::clone(&kikan);
            std::thread::spawn(move || loop {
                kikan.lock().unwrap().apply_move();
                kikan.lock().unwrap().update().unwrap();
            })
        };
        load_lua_script(handler, script).unwrap();
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some(Position(2, 2)));
    }

    #[test]
    fn double_init() {
        let script = "
            api:init();
            api:init();
        ";
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        assert!(load_lua_script(handler, script).is_err());
    }

    #[test]
    fn position() {
        let script = "
            api:init();
            local pos = api:get_position();
            local x = pos.x;
            while x > 0 do
                api:plan_move('S');
                x = x - 1;
            end
        ";
        let kikan = Kikan::kikan_in_a_shell(|| Position(1, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        load_lua_script(handler, script).unwrap();
        for _ in 0..2 {
            kikan.lock().unwrap().apply_move();
        }
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some(Position(0, 0)));
    }

    #[test]
    fn wait_for_update() {
        let script = "
            api:init();
            api:plan_move('s');
            api:wait_for_update();
            api:plan_move('s');
        ";
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        let handler = LocalHandle::new(Arc::clone(&kikan));
        let _unit = {
            let kikan = Arc::clone(&kikan);
            std::thread::spawn(move || loop {
                kikan.lock().unwrap().update().unwrap();
            })
        };
        load_lua_script(handler, script).unwrap();
        for _ in 0..2 {
            kikan.lock().unwrap().apply_move();
        }
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some(Position(-2, 0)));
    }
}
