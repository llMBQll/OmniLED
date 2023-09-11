use log::{trace, warn};
use mlua::{Function, IntoLuaMulti, Lua, OwnedFunction, UserData, Value};

#[derive(Clone)]
pub struct Signal {
    slots: Vec<OwnedFunction>,
    name: String,
}

impl Signal {
    pub fn new(name: String) -> Self {
        Self {
            slots: Vec::new(),
            name,
        }
    }

    pub fn connect(&mut self, slot: OwnedFunction) {
        trace!("Connecting {:?} to {}", slot, self.name);
        match self.slots.iter().find(|x| x.to_ref() == slot.to_ref()) {
            Some(_) => warn!("{:?} is already connected to {}", slot, self.name),
            None => {
                self.slots.push(slot);
            }
        };
    }

    pub fn disonnect(&mut self, slot: OwnedFunction) {
        trace!("Disconnecting {:?} from {}", slot, self.name);

        let len = self.slots.len();
        self.slots.retain(|x| x.to_ref() != slot.to_ref());
        if len == self.slots.len() {
            warn!("{:?} is not connected to {}", slot, self.name)
        }
    }

    pub fn emit<'a, T: IntoLuaMulti<'a>>(&self, lua: &'a Lua, args: T) -> mlua::Result<()> {
        let args = args.into_lua_multi(lua)?;
        trace!("Emitting '{}' with {:?}", self.name, args);

        for slot in &self.slots {
            slot.call(args.clone())?;
        }

        Ok(())
    }
}

impl UserData for Signal {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("connect", |_, this, slot: Function| {
            this.connect(slot.into_owned());
            Ok(())
        });

        methods.add_method_mut("disconnect", |_, this, slot: Function| {
            this.disonnect(slot.into_owned());
            Ok(())
        });

        methods.add_method("emit", |lua, this, args: Value| this.emit(lua, args));
    }
}
