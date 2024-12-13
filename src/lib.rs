#![allow(warnings)]
mod cursor;

use std::os::raw::{c_char, c_int};

use crate::cursor::VsagCursor;
use rusqlite::vtab::{CreateVTab, IndexInfo, VTab, VTabKind};
use rusqlite::{ffi, vtab::UpdateVTab};
use rusqlite::{Connection, Result};

#[expect(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn sqlite3_extension_init(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    Connection::extension_init2(db, pz_err_msg, p_api, extension_init)
}

fn extension_init(db: Connection) -> Result<bool> {
    rusqlite::vtab::vtablog::load_module(&db)?;
    Ok(true)
}

pub struct VsagTable {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
}

unsafe impl<'vtab> VTab<'vtab> for VsagTable {
    type Aux = ();

    type Cursor = VsagCursor<'vtab>;

    fn connect(
        db: &mut rusqlite::vtab::VTabConnection,
        aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        todo!()
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        todo!()
    }

    fn open(&'vtab mut self) -> Result<Self::Cursor> {
        todo!()
    }
}

impl CreateVTab<'_> for VsagTable {
    const KIND: VTabKind = VTabKind::Default;
}
impl UpdateVTab<'_> for VsagTable {
    fn delete(&mut self, arg: rusqlite::types::ValueRef<'_>) -> Result<()> {
        todo!()
    }

    fn insert(&mut self, args: &rusqlite::vtab::Values<'_>) -> Result<i64> {
        todo!()
    }

    fn update(&mut self, args: &rusqlite::vtab::Values<'_>) -> Result<()> {
        todo!()
    }
}
