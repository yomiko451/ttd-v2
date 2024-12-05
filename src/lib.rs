mod app;
mod sync;
mod todo;

pub use crate::{
    app::{App, CURRENT_PATH},
    sync::{sync_app_data, SyncAction, SyncState},
    todo::Todo,
};
