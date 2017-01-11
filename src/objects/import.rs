extern crate mongodb;

use self::mongodb::db::Database;
use types::ThreadedAPI;

pub trait Importable {
    // TODO: Refactor path_from_root to be a reference that is cloned
    // on mutation. Likely needs to be place in an ARC
    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              import_refs: bool,
              run_start_time: i64,
              path_from_root: &Vec<String>)
              -> ();
}
