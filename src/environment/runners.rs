use revm::{EVM, db::{CacheDB, EmptyDB}};

pub struct RunnerCacheDB {
    db: CacheDB<EmptyDB>,
}