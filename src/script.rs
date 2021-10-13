use crate::{error::KikanError, handler::LocalHandle, kikan::Kikan};
use mlua::Lua;
use std::sync::{Arc, Mutex};

pub fn load_lua_script<T>(kikan: Arc<Mutex<Kikan>>, lua_script: T) -> Result<(), KikanError>
where
    T: AsRef<str>,
{
    let lua = Lua::new();

    // library
    // init unit
    let kikan_temp = Arc::clone(&kikan);
    let handler = LocalHandle::new(kikan_temp);
    lua.globals().set("api", handler)?;

    // run script
    let script = lua.load(lua_script.as_ref());
    script.exec()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kikan::Position;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn hello_world() {
        let kikan = Kikan::kikan_in_a_shell(|| Position(0, 0));
        load_lua_script(Arc::clone(&kikan), "api:init()").unwrap();
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
            load_lua_script(Arc::clone(&kikan), "api:init()").unwrap();
        }
        for i in 0..len {
            assert_eq!(
                kikan.lock().unwrap().get_unit_position(i),
                Some(Position(i as i32 + 1, 0))
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
        load_lua_script(Arc::clone(&kikan), script).unwrap();
        for _ in 0..4 {
            kikan.lock().unwrap().apply_move();
        }
        assert_eq!(kikan.lock().unwrap().get_unit_position(0), Some(Position(2, 2)));
    }
}
