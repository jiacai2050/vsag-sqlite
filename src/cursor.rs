use std::{ffi::c_int, marker::PhantomData};

use rusqlite::{
    ffi,
    types::{Null, ValueRef},
    vtab::{Context, VTabCursor, Values},
    Error,
};
use tracing::{debug, info};
use vsag::KnnSearchOutput;

use crate::{Column, VectorStore, VsagTable};

#[repr(C)]
pub struct VsagCursor<'vtab> {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab_cursor,
    store: VectorStore,
    cursor_id: usize,
    dim: usize,
    limit: Option<usize>,
    result: Option<KnnSearchOutput>,
    result_idx: usize,
    phantom: PhantomData<&'vtab VsagTable>,
}

impl VsagCursor<'_> {
    pub fn new(cursor_id: usize, dim: usize, store: VectorStore) -> Self {
        Self {
            base: ffi::sqlite3_vtab_cursor::default(),
            store,
            cursor_id,
            dim,
            limit: None,
            result: None,
            result_idx: 0,
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
        debug!("VTabCursor::filter({}, {:?})", idx_num, idx_str,);

        let query: String = args.get(0)?;
        let query: Vec<f32> = ron::from_str(&query).map_err(|e| {
            Error::ModuleError(format!("vec column is not vector of f32, value:{query}."))
        })?;
        if query.len() != self.dim {
            return Err(Error::ModuleError(format!(
                "query column should have {} dimensions, value:{}.",
                self.dim,
                query.len()
            )));
        }

        let params = r#"{ "hnsw": { "ef_search": 100 } }"#;
        let result = self
            .store
            .knn_search(&query, self.limit.unwrap_or(100), params)
            .map_err(|e| Error::ModuleError(format!("Knn search failed, err:{e:?}")))?;
        self.result = Some(result);
        Ok(())
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        debug!("VTabCursor::next");
        self.result_idx += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        let eof = self
            .result
            .as_ref()
            .map_or_else(|| true, |res| self.result_idx >= res.ids.len());
        debug!("VTabCursor::eof {eof}");

        eof
    }

    fn column(&self, ctx: &mut Context, i: c_int) -> rusqlite::Result<()> {
        let result = self.result.as_ref().unwrap();
        let idx = self.result_idx as usize;
        debug!("VTabCursor::column, i:{i}, idx:{idx}",);

        match Column::try_from(i)? {
            Column::Id => ctx.set_result(&result.ids[idx]),
            Column::Distance => ctx.set_result(&result.distances[idx]),
            Column::Vector => ctx.set_result(&Null),
        }
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        debug!("VTabCursor::rowid");

        let result = self.result.as_ref().unwrap();
        Ok(result.ids[self.result_idx])
    }
}
