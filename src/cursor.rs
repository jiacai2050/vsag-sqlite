use std::marker::PhantomData;

use rusqlite::{ffi, vtab::VTabCursor};

use crate::VsagTable;

/// A cursor for the Series virtual table
#[repr(C)]
pub struct VsagCursor<'vtab> {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab_cursor,
    cursor_id: usize,
    row_id: usize,
    phantom: PhantomData<&'vtab VsagTable>,
}

impl VsagCursor<'_> {
    pub fn new(cursor_id: usize) -> Self {
        Self {
            base: ffi::sqlite3_vtab_cursor::default(),
            cursor_id,
            row_id: 0,
            phantom: PhantomData,
        }
    }
}

unsafe impl VTabCursor for VsagCursor<'_> {
    fn filter(
        &mut self,
        idx_num: std::os::raw::c_int,
        idx_str: Option<&str>,
        args: &rusqlite::vtab::Values<'_>,
    ) -> rusqlite::Result<()> {
        todo!()
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        todo!()
    }

    fn eof(&self) -> bool {
        todo!()
    }

    fn column(
        &self,
        ctx: &mut rusqlite::vtab::Context,
        i: std::os::raw::c_int,
    ) -> rusqlite::Result<()> {
        todo!()
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        todo!()
    }
}
