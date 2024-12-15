#![allow(warnings)]
mod cursor;

use std::any::Any;
use std::cell::RefCell;
use std::os::raw::{c_char, c_int};
use std::rc::Rc;
use vsag::VsagIndex;

use crate::cursor::VsagCursor;
use rusqlite::types::ValueRef;
use rusqlite::vtab::{
    parameter, read_only_module, update_module, CreateVTab, IndexConstraintOp, IndexInfo, VTab,
    VTabKind,
};
use rusqlite::{ffi, vtab::UpdateVTab};
use rusqlite::{Connection, Error, Result};
use tracing::debug;

#[no_mangle]
pub unsafe extern "C" fn sqlite3_extension_init(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    Connection::extension_init2(db, pz_err_msg, p_api, extension_init)
}

fn extension_init(db: Connection) -> Result<bool> {
    db.create_module("vsag", update_module::<VsagTable>(), None)?;
    tracing_subscriber::fmt::init();

    Ok(true)
}

pub type VectorStore = Rc<VsagIndex>;

#[repr(C)]
pub struct VsagTable {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    /// Core structure
    store: VectorStore,
    /// Vector dimension size in store
    dim: usize,
    /// Number of cursors created
    n_cursor: usize,
}

/// Column indexes for the vsag virtual table.
#[repr(i32)]
pub enum Column {
    Id = 0,
    Vector,
    Distance,
}

impl TryFrom<i32> for Column {
    type Error = rusqlite::Error;
    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Column::Id),
            1 => Ok(Column::Vector),
            2 => Ok(Column::Distance),
            _ => Err(rusqlite::Error::ModuleError(format!(
                "Invalid column number: {}",
                value
            ))),
        }
    }
}

unsafe impl<'vtab> VTab<'vtab> for VsagTable {
    type Aux = ();

    type Cursor = VsagCursor<'vtab>;

    fn connect(
        db: &mut rusqlite::vtab::VTabConnection,
        aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> Result<(String, Self)> {
        let schema = r#"CREATE TABLE x(id PRIMARY KEY, vec, distance)"#;
        let mut dim = None;
        let args = &args[3..];
        for c_slice in args {
            let (param, value) = parameter(c_slice)?;
            match param {
                "dimension" => {
                    let value = value.parse::<usize>().map_err(|e| {
                        Error::ModuleError(format!("dimension should be a number, value:{value}."))
                    })?;
                    dim = Some(value);
                }
                _ => {
                    return Err(Error::ModuleError(format!(
                        "unknown param: {param}, value:{value}."
                    )))
                }
            }
        }
        let dim = match dim {
            None => {
                return Err(Error::ModuleError(
                    "`dimension` param is not set!".to_string(),
                ))
            }
            Some(v) => v,
        };
        let store = Rc::new(
            VsagIndex::new(
                "hnsw",
                &format!(
                    r#"
{{
  "dtype": "float32",
  "metric_type": "l2",
  "dim": {},
  "hnsw": {{
    "max_degree": 16,
    "ef_construction": 200
  }}
}}"#,
                    dim
                ),
            )
            .map_err(|e| Error::ModuleError(format!("create index failed, err:{e:?}.")))?,
        );
        let table = Self {
            base: ffi::sqlite3_vtab::default(),
            store,
            dim,
            n_cursor: 0,
        };
        for (i, arg) in args.iter().enumerate() {
            debug!(
                "connect args, i={i}, value={:?}",
                String::from_utf8_lossy(arg)
            );
        }
        Ok((schema.to_string(), table))
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<()> {
        info.set_estimated_cost(1_000_000.);
        let mut const_and_usages = info.constraints_and_usages();
        for (cons, mut usage) in const_and_usages {
            let col = cons.column();
            let op = cons.operator();
            let usable = cons.is_usable();
            debug!("best_index cons: {col:?}, {usable}, {op:?}");
            if !usable {
                continue;
            }
            if col == Column::Vector as c_int {
                if !matches!(op, IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_MATCH) {
                    return Err(Error::ModuleError(format!(
                        "Only support match operator for `vec` column"
                    )));
                }
                usage.set_argv_index(1);
                usage.set_omit(true);
            } else {
                return Err(Error::ModuleError(format!(
                    "Only support filter by `vec` column"
                )));
            }
        }
        Ok(())
    }

    fn open(&'vtab mut self) -> Result<Self::Cursor> {
        debug!(cursor = self.n_cursor, "VsagTable::open");
        self.n_cursor += 1;
        Ok(VsagCursor::new(self.n_cursor, self.dim, self.store.clone()))
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
        let id: i64 = args.get(Column::Id as usize + 2)?;
        let vec: String = args.get(Column::Vector as usize + 2)?;
        let vec: Vec<f32> = ron::from_str(&vec).map_err(|e| {
            Error::ModuleError(format!("vec column is not vector of f32, value:{vec}."))
        })?;
        if vec.len() != self.dim {
            return Err(Error::ModuleError(format!(
                "vec column should have {} dimensions, value:{}.",
                self.dim,
                vec.len()
            )));
        }
        debug!("VTabLog::insert({id} {vec:?})",);
        self.store
            .build(1 /*num_vectors*/, self.dim, &vec![id], &vec)
            .map_err(|e| Error::ModuleError(format!("add vec into index failed, err:{e:?}.")));
        Ok(id)
    }

    fn update(&mut self, args: &rusqlite::vtab::Values<'_>) -> Result<()> {
        Err(rusqlite::Error::ModuleError(
            "Update statment not supported yet.".to_string(),
        ))
    }
}
