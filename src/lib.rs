#![allow(warnings)]
mod cursor;

use std::any::Any;
use std::os::raw::{c_char, c_int};

use crate::cursor::VsagCursor;
use rusqlite::types::ValueRef;
use rusqlite::vtab::{update_module, CreateVTab, IndexInfo, VTab, VTabKind};
use rusqlite::{ffi, vtab::UpdateVTab};
use rusqlite::{Connection, Error, Result};

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
    db.create_module("vsag_table", update_module::<VsagTable>(), None)?;

    Ok(true)
}

pub struct VsagTable {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    /// Core structure
    vectors: Vec<(i64, Vec<f32>)>,
    /// Number of cursors created
    n_cursor: usize,
}

unsafe impl<'vtab> VTab<'vtab> for VsagTable {
    type Aux = ();

    type Cursor = VsagCursor<'vtab>;

    fn connect(
        db: &mut rusqlite::vtab::VTabConnection,
        aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        let schema = r#"CREATE TABLE x(id PRIMARY KEY, vec, score)"#;
        let table = Self {
            base: ffi::sqlite3_vtab::default(),
            vectors: vec![],
            n_cursor: 0,
        };
        for (i, arg) in args.iter().enumerate() {
            println!(
                "connect args, i={i}, value={:?}",
                String::from_utf8_lossy(arg)
            );
        }
        Ok((schema.to_string(), table))
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        info.set_estimated_cost(500.);
        info.set_estimated_rows(500);
        Ok(())
    }

    fn open(&'vtab mut self) -> Result<Self::Cursor> {
        self.n_cursor += 1;
        Ok(VsagCursor::new(self.n_cursor))
    }
}

impl CreateVTab<'_> for VsagTable {
    const KIND: VTabKind = VTabKind::Default;
}

impl UpdateVTab<'_> for VsagTable {
    fn delete(&mut self, arg: rusqlite::types::ValueRef<'_>) -> Result<()> {
        Err(Error::ModuleError(
            "Delete statment not supported yet.".to_string(),
        ))
    }

    // The first two arguments are the rowid and the new rowid, the rest are the values of the columns.
    // https://www.sqlite.org/vtab.html#the_xupdate_method
    fn insert(&mut self, args: &rusqlite::vtab::Values<'_>) -> Result<i64> {
        assert_eq!(args.len(), 5);
        let id: i64 = args.get(2)?;
        let vec: String = args.get(3)?;
        let vec: Vec<f32> = ron::from_str(&vec).map_err(|e| {
            Error::ModuleError(format!("vec column is not vector of f32, value:{vec}."))
        })?;
        self.vectors.push((id, vec));
        println!("VTabLog::insert({id} {vec:?})",);
        Ok(1)
    }

    fn update(&mut self, args: &rusqlite::vtab::Values<'_>) -> Result<()> {
        Err(rusqlite::Error::ModuleError(
            "Update statment not supported yet.".to_string(),
        ))
    }
}
