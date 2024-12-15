use std::{ffi::c_int, marker::PhantomData};

use rusqlite::{
    ffi,
    types::{Null, ValueRef},
    vtab::{Context, VTabCursor, Values},
    Error,
};
use tracing::{debug, info};

use crate::{Column, VectorStore, VsagTable};

/// A cursor for the Series virtual table
#[repr(C)]
pub struct VsagCursor<'vtab> {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab_cursor,
    store: VectorStore,
    cursor_id: usize,
    row_id: i64,
    limit: Option<usize>,
    phantom: PhantomData<&'vtab VsagTable>,
}

impl VsagCursor<'_> {
    pub fn new(cursor_id: usize, store: VectorStore) -> Self {
        Self {
            base: ffi::sqlite3_vtab_cursor::default(),
            store,
            cursor_id,
            row_id: -1,
            limit: None,
            phantom: PhantomData,
        }
    }
}

unsafe impl VTabCursor for VsagCursor<'_> {
    fn filter(
        &mut self,
        idx_num: c_int,
        idx_str: Option<&str>,
        args: &Values<'_>,
    ) -> rusqlite::Result<()> {
        if args.len() != 1 {
            return Err(Error::ModuleError(format!("no where condition found!")));
        }
        let vec: String = args.get(0)?;

        let vec: Vec<f32> = ron::from_str(&vec).map_err(|e| {
            Error::ModuleError(format!("vec column is not vector of f32, value:{vec}."))
        })?;
        debug!("VTabCursor::filter({}, {:?}, {vec:?})", idx_num, idx_str,);
        self.next();
        Ok(())
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        debug!("VTabCursor::next");
        self.row_id += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        let eof = (self.row_id as usize) >= self.store.len();
        debug!("VTabCursor::eof {eof}");

        eof
    }

    fn column(&self, ctx: &mut Context, i: c_int) -> rusqlite::Result<()> {
        let idx = self.row_id as usize;
        debug!(
            "VTabCursor::column, i:{i}, idx:{idx}, store:{:?}",
            self.store
        );
        match Column::try_from(i)? {
            Column::Id => ctx.set_result(&self.store[idx].0),
            Column::Vector => {
                let vec = &self.store[idx].1;
                let s = ron::to_string(vec).unwrap();
                debug!(s=?s);
                ctx.set_result(&s)
            }
            Column::Score => ctx.set_result(&2),
        }
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        debug!("VTabCursor::rowid");
        let idx = self.row_id as usize;
        Ok(self.store[idx].0)
    }
}
