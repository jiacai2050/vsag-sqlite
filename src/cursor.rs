use std::{ffi::c_int, marker::PhantomData};

use rusqlite::{
    ffi,
    types::ValueRef,
    vtab::{Context, VTabCursor, Values},
};

use crate::{VectorStore, VsagTable};

/// A cursor for the Series virtual table
#[repr(C)]
pub struct VsagCursor<'vtab> {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab_cursor,
    store: VectorStore,
    cursor_id: usize,
    row_id: usize,
    phantom: PhantomData<&'vtab VsagTable>,
}

impl VsagCursor<'_> {
    pub fn new(cursor_id: usize, store: VectorStore) -> Self {
        Self {
            base: ffi::sqlite3_vtab_cursor::default(),
            store,
            cursor_id,
            row_id: 0,
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
        println!(
            "VTabCursor::filter({}, {:?}, {:?})",
            idx_num,
            idx_str,
            args.iter().collect::<Vec<ValueRef<'_>>>()
        );
        Ok(())
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        println!("VTabCursor::next");
        Ok(())
    }

    fn eof(&self) -> bool {
        println!("VTabCursor::eof");

        true
    }

    fn column(&self, ctx: &mut Context, i: c_int) -> rusqlite::Result<()> {
        println!("VTabCursor::column, i:{i}");

        Ok(())
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        Ok(1)
    }
}
