#![allow(unused_imports)]
use mlua::{
    AnyUserData, ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, MaybeSend, MetaMethod, Result,
    Table, UserData, Value,
};
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::{cell::RefCell, ops::Deref};
use toml_edit::{Document, DocumentMut, Item, Key, Table as TomlTable, Value as TomlValue};

use crate::document::ItemLua::Standalone;

#[derive(Clone)]
pub(crate) struct DocumentLua {
    doc: Rc<RefCell<DocumentMut>>,
}

enum ItemPathLua {
    Key(mlua::String),
    Index(usize),
    Path(mlua::Table),
}

impl FromLua for ItemPathLua {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        match value {
            Value::String(it) => Ok(ItemPathLua::Key(it)),
            Value::Integer(it) => {
                let number: usize = it.try_into().map_err(|_| {
                    mlua::Error::RuntimeError(format!(
                        "toml-edit: expected 'path' to be one of string | number | table"
                    ))
                })?;
                Ok(ItemPathLua::Index(number))
            }
            Value::Table(it) => Ok(ItemPathLua::Path(it)),
            it => Err(mlua::Error::RuntimeError(format!(
                "toml-edit: expected 'path' to be one of string | number | table, got {}",
                it.type_name()
            ))),
        }
    }
}

enum ItemLua {
    Member(DocumentLua, ItemPathLua),
    Standalone(Item),
}

impl DocumentLua {
    fn borrow(&self) -> Ref<'_, DocumentMut> {
        self.doc.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, DocumentMut> {
        self.doc.borrow_mut()
    }
}

impl ItemLua {
    fn with_it<F, R>(&self, map: F) -> Option<R>
    where
        F: FnOnce(&Item) -> R,
    {
        match self {
            Self::Standalone(it) => Some(map(it)),
            Self::Member(doc, path) => {
                let borrowed_doc = doc.borrow();
                let root_item = borrowed_doc.as_item();
                match resolve_path(root_item, path) {
                    Some(item) => Some(map(item)),
                    None => None,
                }
            }
        }
    }

    fn type_name(&self) -> &'static str {
        self.with_it(|it| it.type_name()).unwrap_or("none")
        // match self {
        //     Self::Standalone(it) => it.type_name(),
        //     Self::Member(doc, path) => {
        //         let borrowed_doc = doc.borrow();
        //         let root_item = borrowed_doc.as_item();
        //
        //         resolve_path(root_item, path).map_or("none", |it| it.type_name())
        //     }
        // }
    }
}

impl UserData for ItemLua {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("type_name", |lua, this| this.type_name().into_lua(lua));
        fields.add_meta_field(MetaMethod::Type, "toml_edit.Item");
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("deserialize", |lua, this: AnyUserData| {
            method_deserialize("Item.deserialize", "self", 1, lua, this)
        });
    }
}

impl DocumentLua {
    pub fn new(document: DocumentMut) -> Self {
        return DocumentLua {
            doc: RefCell::new(document).into(),
        };
    }
}

impl UserData for DocumentLua {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field(MetaMethod::Type, "toml_edit.Document");
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

        methods.add_function("deserialize", |lua, this: AnyUserData| {
            method_deserialize("Document.deserialize", "self", 1, lua, this)
        });

        methods.add_method("get", |lua, this, args: ItemPathLua| {
            ItemLua::Member(this.clone(), args).into_lua(lua)
        });

        methods.add_meta_method(MetaMethod::ToString, |lua, this, _arg: Value| {
            this.borrow().to_string().into_lua(lua)
        });
    }
}

// -- Polymorphic methods

fn method_deserialize(
    function_name: &str,
    param_name: &str,
    param_index: i8,
    lua: &Lua,
    this: AnyUserData,
) -> Result<Value> {
    if let Ok(this) = this.borrow::<DocumentLua>() {
        return item_to_lua(lua, this.borrow().as_item());
    }
    if let Ok(this) = this.borrow::<ItemLua>() {
        return this.with_it(|item| {
            item_to_lua(lua, item)
        }).unwrap_or(Ok(Value::Nil))
    }
    Err(mlua::Error::RuntimeError(format!(
        "toml-edit: {}: argument #{} '{}' expected to be one a toml_edit.Document, got {}",
        function_name,
        param_index,
        param_name,
        this.type_name()
            .unwrap_or(Some("<unknown>".to_string()))
            .unwrap_or("<unknown>".to_string())
    )))
}

fn field_trailing_set(
    function_name: &str,
    param_index: i8,
    param_name: &str,
    _lua: &Lua,
    this: AnyUserData,
    trailing: mlua::String,
) -> Result<()> {
    if let Ok(this) = this.borrow_mut::<DocumentLua>() {
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
    if let Ok(this) = this.borrow::<DocumentLua>() {
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
    if let Ok(this) = this.borrow::<DocumentLua>() {
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

// -- Helpers
fn resolve_path<'a>(root: &'a Item, path: &ItemPathLua) -> Option<&'a Item> {
    // NOTE: keep in mind the -1 to suppor lau 1 based index
    match path {
        ItemPathLua::Key(key) => {
            let k = key.to_str().ok()?;
            root.get(&*k)
        }
        ItemPathLua::Index(idx) => root.get(*idx - 1),
        ItemPathLua::Path(table) => {
            let mut current = root;
            let pairs = table.pairs::<mlua::Value, mlua::Value>();

            for pair in pairs {
                let (_, val) = pair.ok()?;
                match val {
                    mlua::Value::String(s) => {
                        let s_ref = s.to_str().ok()?;
                        current = current.get(&*s_ref)?;
                    }
                    mlua::Value::Integer(i) => {
                        current = current.get(i as usize - 1)?;
                    }
                    _ => return None,
                }
            }
            Some(current)
        }
    }
}
