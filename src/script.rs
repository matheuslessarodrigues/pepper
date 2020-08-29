#![macro_use]

use std::{error::Error, fmt, fs::File, io::Read, path::Path, sync::Arc};

use mlua::prelude::{
    FromLua, FromLuaMulti, Lua, LuaError, LuaLightUserData, LuaResult, LuaString, LuaValue,
    ToLuaMulti,
};

use crate::{
    buffer::BufferCollection,
    buffer_view::{BufferViewCollection, BufferViewHandle},
    config::Config,
    connection::TargetClient,
    editor::EditorLoop,
    editor_operation::EditorOperationSerializer,
    keymap::KeyMapCollection,
    script_bindings,
};

pub type ScriptResult<T> = LuaResult<T>;

pub struct ScriptError<T>(T);
impl<T> ScriptError<T>
where
    T: 'static + fmt::Display,
{
    pub fn from(e: T) -> LuaError {
        LuaError::ExternalError(Arc::new(ScriptError(e)))
    }
}
impl<T> fmt::Debug for ScriptError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl<T> fmt::Display for ScriptError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T> Error for ScriptError<T> where T: fmt::Display {}

pub struct ScriptStr<'lua>(LuaString<'lua>);
impl<'lua> ScriptStr<'lua> {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    pub fn to_str(&self) -> ScriptResult<&str> {
        self.0.to_str()
    }
}
impl<'lua> FromLua<'lua> for ScriptStr<'lua> {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = lua_value {
            Ok(ScriptStr(s))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: lua_value.type_name(),
                to: stringify!(ScriptStr),
                message: None,
            })
        }
    }
}

pub struct ScriptContext<'a> {
    pub editor_loop: &'a mut EditorLoop,
    pub target_client: TargetClient,
    pub operations: &'a mut EditorOperationSerializer,

    pub config: &'a Config,
    pub keymaps: &'a mut KeyMapCollection,
    pub buffers: &'a mut BufferCollection,
    pub buffer_views: &'a mut BufferViewCollection,
    pub current_buffer_view_handle: &'a mut Option<BufferViewHandle>,
}

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self::try_new().unwrap()
    }

    pub fn try_new() -> ScriptResult<Self> {
        let libs = mlua::StdLib::TABLE
            | mlua::StdLib::STRING
            | mlua::StdLib::UTF8
            | mlua::StdLib::MATH
            | mlua::StdLib::PACKAGE;
        let lua = Lua::new_with(libs)?;

        let mut this = Self { lua };
        script_bindings::bind_all(&mut this)?;

        Ok(this)
    }

    pub fn register_ctx_function<'lua, A, R, F>(
        &'lua mut self,
        name: &str,
        func: F,
    ) -> ScriptResult<()>
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&mut ScriptContext, A) -> ScriptResult<R>,
    {
        let func = self.lua.create_function(move |lua, args| {
            let ctx: LuaLightUserData = lua.named_registry_value("ctx")?;
            let ctx = unsafe { &mut *(ctx.0 as *mut _) };
            func(ctx, args)
        })?;
        self.lua.globals().set(name, func)?;
        Ok(())
    }

    pub fn eval(&mut self, mut ctx: ScriptContext, source: &str) -> ScriptResult<()> {
        self.update_ctx(&mut ctx)?;
        self.lua.load(source).exec()?;
        Ok(())
    }

    pub fn eval_entry_file(&mut self, mut ctx: ScriptContext, path: &Path) -> ScriptResult<()> {
        let mut file = File::open(path).map_err(|e| LuaError::ExternalError(Arc::new(e)))?;
        let metadata = file
            .metadata()
            .map_err(|e| LuaError::ExternalError(Arc::new(e)))?;
        let mut source = String::with_capacity(metadata.len() as _);
        file.read_to_string(&mut source)
            .map_err(|e| LuaError::ExternalError(Arc::new(e)))?;

        self.update_ctx(&mut ctx)?;

        let chunk = self.lua.load(&source);
        let chunk = if let Some(name) = path.to_str() {
            chunk.set_name(name)?
        } else {
            chunk
        };

        chunk.exec()?;
        Ok(())
    }

    fn update_ctx(&mut self, ctx: &mut ScriptContext) -> ScriptResult<()> {
        self.lua
            .set_named_registry_value("ctx", LuaLightUserData(ctx as *mut ScriptContext as _))
    }
}