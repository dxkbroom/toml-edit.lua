#![allow(unused_imports)]
use mlua::{ExternalError, IntoLua, Lua, LuaSerdeExt, MetaMethod, Result, Table, UserData, Value};
use std::cell::{Ref, RefMut};
use std::{cell::RefCell, ops::Deref};
use std::rc::Rc;
use toml_edit::{Document, DocumentMut, Key};

pub(crate) struct LuaDocument {
    doc: Rc<RefCell<DocumentMut>>
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
        return LuaDocument { doc: RefCell::new(document).into()}
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

}
