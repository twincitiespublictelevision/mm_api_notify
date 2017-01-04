///
/// Holds config values
///
pub const DB_NAME: &'static str = "core_data_model";
pub const DB_USERNAME: &'static str = "tptdrupal";
pub const DB_PASSWORD: &'static str = "^tpt5drupal7*";

pub const MIN_RUNTIME_DELTA: i64 = 30;

pub const DEFAULT_POOL_SIZE: usize = 2;

pub fn pool_size_for(requested_for_type: &str) -> usize {
    match requested_for_type {
        "show_page_list" => 1,
        "show_list" => 2,
        "show" => 4,
        "season" => 4,
        "special" => 4,
        _ => DEFAULT_POOL_SIZE,
    }
}
