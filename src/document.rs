#![allow(unused_imports)]
use mlua::{
    AnyUserData, ExternalError, IntoLua, Lua, LuaSerdeExt, MetaMethod, Result, Table, UserData,
    Value,
};
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};
use toml_edit::{Document, DocumentMut, Item, Key, Table as TomlTable, Value as TomlValue};

pub(crate) struct LuaDocument {
    doc: Rc<RefCell<DocumentMut>>,
}

impl LuaDocument {
    fn borrow(&self) -> Ref<'_, DocumentMut> {
        self.doc.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, DocumentMut> {
        self.doc.borrow_mut()
    }
}

impl LuaDocument {
    pub fn new(document: DocumentMut) -> Self {
        return LuaDocument {
            doc: RefCell::new(document).into(),
        };
    }
}

impl UserData for LuaDocument {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("format", |_lua, this, _arg: Value| {
            let mut doc = this.borrow_mut();
            doc.fmt();
            for (_, item) in doc.iter_mut() {
                if let Some(sub_table) = item.as_table_mut() {
                    sub_table.fmt();
                }
            }
            Ok(())
        });

        methods.add_meta_method(MetaMethod::ToString, |lua, this, _arg: Value| {
            this.borrow().to_string().into_lua(lua)
        });
    }

    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            item_to_lua(lua, this.borrow().as_item())
        });
        fields.add_field_function_get("value", |lua, this| {
            field_value_get("Document.value", 1, "self", lua, this)
        });
        fields.add_field_function_get("trailing", |lua, this| {
            field_trailing_get("Document.trailing", 1, "self", lua, this)
        });
        fields.add_field_function_set("trailing", |lua, this, value: mlua::String| {
            field_trailing_set("Document.trailing", 1, "self", lua, this, value)
        });
    }
}

// -- Polymorphic methods

fn field_trailing_set(
    function_name: &str,
    param_index: i8,
    param_name: &str,
    lua: &Lua,
    this: AnyUserData,
    trailing: mlua::String,
) -> Result<()> {
    if let Ok(this) = this.borrow_mut::<LuaDocument>() {
        this.borrow_mut()
            .set_trailing(trailing.to_str()?.to_string());
        return Ok(());
    }
    Err(mlua::Error::RuntimeError(format!(
        "toml-edit: {}: argument #{} '{}' expected to be one a toml_edit.Collection, got {}",
        function_name,
        param_index,
        param_name,
        this.type_name()
            .unwrap_or(Some("<unknown>".to_string()))
            .unwrap_or("<unknown>".to_string())
    )))
}

fn field_trailing_get(
    function_name: &str,
    param_index: i8,
    param_name: &str,
    lua: &Lua,
    this: AnyUserData,
) -> Result<Value> {
    if let Ok(this) = this.borrow::<LuaDocument>() {
        return this.borrow().trailing().as_str().into_lua(lua);
    }
    Err(mlua::Error::RuntimeError(format!(
        "toml-edit: {}: argument #{} '{}' expected to be one a toml_edit.Collection, got {}",
        function_name,
        param_index,
        param_name,
        this.type_name()
            .unwrap_or(Some("<unknown>".to_string()))
            .unwrap_or("<unknown>".to_string())
    )))
}

fn field_value_get(
    function_name: &str,
    param_index: i8,
    param_name: &str,
    lua: &Lua,
    this: AnyUserData,
) -> Result<Value> {
    if let Ok(this) = this.borrow::<LuaDocument>() {
        return item_to_lua(lua, this.borrow().as_item());
    }
    Err(mlua::Error::RuntimeError(format!(
        "toml-edit: {}: argument #{} '{}' expected to be one of toml_edit.Item | toml_edit.Collection, got {}",
        function_name,
        param_index,
        param_name,
        this.type_name().unwrap_or(Some("<unknown>".to_string())).unwrap_or("<unknown>".to_string())
    )))
}

// -- Lua <> TOML

fn item_to_lua(lua: &Lua, item: &Item) -> Result<Value> {
    match item {
        Item::None => Ok(Value::Nil),
        Item::Table(table) => table_to_lua(lua, table),
        Item::Value(value) => value_to_lua(lua, value),
        Item::ArrayOfTables(arr) => {
            let lua_table = lua.create_table_with_capacity(arr.len(), 0)?;
            for item in arr.iter() {
                lua_table.raw_push(table_to_lua(lua, item)?)?
            }
            Ok(Value::Table(lua_table))
        }
    }
}

fn table_to_lua(lua: &Lua, table: &TomlTable) -> Result<Value> {
    let lua_table = lua.create_table_with_capacity(0, table.len())?;
    for (key, item) in table.iter() {
        lua_table.raw_set(key, item_to_lua(lua, item)?)?;
    }
    Ok(Value::Table(lua_table))
}

fn value_to_lua(lua: &Lua, value: &TomlValue) -> Result<Value> {
    match value {
        TomlValue::String(inner) => inner.value().to_string().into_lua(lua),
        TomlValue::Integer(inner) => inner.value().into_lua(lua),
        TomlValue::Float(inner) => inner.value().into_lua(lua),
        TomlValue::Boolean(inner) => inner.value().into_lua(lua),
        // TODO: expose a true object maybe
        TomlValue::Datetime(inner) => inner.value().to_string().into_lua(lua),
        TomlValue::InlineTable(table) => {
            let lua_table = lua.create_table_with_capacity(0, table.len())?;
            for (key, item) in table.iter() {
                lua_table.raw_set(key, value_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        TomlValue::Array(arr) => {
            let lua_table = lua.create_table_with_capacity(arr.len(), 0)?;
            for item in arr.iter() {
                lua_table.raw_push(value_to_lua(lua, item)?)?
            }
            Ok(Value::Table(lua_table))
        }
    }
}
