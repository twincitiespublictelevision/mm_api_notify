extern crate core_data_client;

use std::sync::Arc;
use core_data_client::Client;

pub type ThreadedAPI = Arc<Client<'static>>;
