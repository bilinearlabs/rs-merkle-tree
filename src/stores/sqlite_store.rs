// Copyright 2025 Bilinear Labs - MIT License

//! SQLite store implementation.

#[cfg(feature = "sqlite_store")]
use crate::{MerkleError, Node, Store};
#[cfg(feature = "sqlite_store")]
use rusqlite::{params, Connection, OptionalExtension};

#[cfg(feature = "sqlite_store")]
pub struct SqliteStore {
    conn: Connection,
    // Keeping an in-memory counter to avoid querying on every access.
    num_leaves: u64,
}

#[cfg(feature = "sqlite_store")]
impl SqliteStore {
    const KEY_NUM_LEAVES: &'static str = "NUM_LEAVES";

    fn db_error<E: std::fmt::Display>(err: E) -> MerkleError {
        MerkleError::StoreError(err.to_string())
    }

    fn decode_node(bytes: &[u8]) -> Result<Node, MerkleError> {
        let arr: [u8; Node::LEN] = bytes
            .try_into()
            .map_err(|_| MerkleError::StoreError("invalid node length".into()))?;
        Ok(Node::from(arr))
    }

    // Use ":memory:" for in-memory database.
    pub fn new(file_path: &str) -> Self {
        let conn = Connection::open(file_path).expect("failed to open sqlite DB");

        conn.execute_batch("PRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;")
            .expect("failed to set WAL mode and synchronous pragma");

        // Create schema if not exists.
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS nodes (
                 level INTEGER NOT NULL,
                 idx   INTEGER NOT NULL,
                 node  BLOB    NOT NULL CHECK(length(node) = 32),
                 PRIMARY KEY(level, idx)
             );
             CREATE TABLE IF NOT EXISTS metadata (
                 key   TEXT PRIMARY KEY,
                 value BLOB NOT NULL
             );
             COMMIT;",
        )
        .expect("failed to create tables");

        // Load persisted leaf count
        let num_leaves: u64 = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = ?1",
                params![Self::KEY_NUM_LEAVES],
                |row| {
                    let bytes: [u8; 8] = row
                        .get::<_, Vec<u8>>(0)?
                        .as_slice()
                        .try_into()
                        .map_err(|_| rusqlite::Error::InvalidQuery)?;
                    Ok(u64::from_be_bytes(bytes))
                },
            )
            .optional()
            .expect("failed to query num leaves")
            .unwrap_or(0);

        // If the count is 0, clear the db, just in case.
        if num_leaves == 0 {
            conn.execute_batch("DELETE FROM nodes; DELETE FROM metadata;")
                .expect("failed to clear inconsistent DB state");
        }

        Self { conn, num_leaves }
    }
}

#[cfg(feature = "sqlite_store")]
impl Store for SqliteStore {
    fn get(&self, levels: &[u32], indices: &[u64]) -> Result<Vec<Option<Node>>, MerkleError> {
        if levels.len() != indices.len() {
            return Err(MerkleError::StoreError(
                "levels and indices must have the same length".into(),
            ));
        }

        if levels.is_empty() {
            return Ok(Vec::new());
        }

        // TODO: All this is a bit messy. Try to abstract away the order so that i dont have
        // to care here to order the results.
        let mut sql = String::from("SELECT level, idx, node FROM nodes WHERE (level, idx) IN (");
        for i in 0..levels.len() {
            if i > 0 {
                sql.push(',');
            }
            sql.push_str("(?, ?)");
        }
        sql.push(')');

        let mut stmt = self.conn.prepare_cached(&sql).map_err(Self::db_error)?;

        // Build param list interleaving level and idx as i64
        let mut binds: Vec<i64> = Vec::with_capacity(levels.len() * 2);
        for (&lvl, &idx) in levels.iter().zip(indices.iter()) {
            binds.push(lvl as i64);
            binds.push(idx as i64);
        }

        use std::collections::HashMap;
        let mut map: HashMap<(u32, u64), Node> = HashMap::with_capacity(levels.len());

        let rows_iter = stmt
            .query_map(rusqlite::params_from_iter(binds.iter()), |row| {
                let l: i64 = row.get(0)?;
                let i: i64 = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                Ok(((l as u32, i as u64), blob))
            })
            .map_err(Self::db_error)?;

        for pair in rows_iter {
            let ((l, i), blob) = pair.map_err(Self::db_error)?;
            let node = Self::decode_node(&blob)?;
            map.insert((l, i), node);
        }

        // Build result vector in the same order as the inputs.
        Ok(levels
            .iter()
            .zip(indices.iter())
            .map(|(&l, &i)| map.get(&(l, i)).copied())
            .collect())
    }

    fn put(&mut self, items: &[(u32, u64, Node)]) -> Result<(), MerkleError> {
        let tx = self.conn.transaction().map_err(Self::db_error)?;

        {
            let mut insert_stmt = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO nodes (level, idx, node) VALUES (?1, ?2, ?3)",
                )
                .map_err(Self::db_error)?;

            for (level, index, node) in items {
                insert_stmt
                    .execute(params![*level as i64, *index as i64, node.as_ref()])
                    .map_err(Self::db_error)?;
            }
        }

        let counter = items.iter().filter(|(level, _, _)| *level == 0).count() as u64;
        if counter > 0 {
            let new_leaves = self.num_leaves + counter;
            tx.execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                params![Self::KEY_NUM_LEAVES, new_leaves.to_be_bytes().to_vec()],
            )
            .map_err(Self::db_error)?;

            self.num_leaves = new_leaves;
        }

        tx.commit().map_err(Self::db_error)?;
        Ok(())
    }

    fn get_num_leaves(&self) -> u64 {
        self.num_leaves
    }
}
