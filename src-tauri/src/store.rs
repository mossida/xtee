use std::{sync::Arc, time::Duration};

use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

pub type Store = tauri_plugin_store::Store<Wry>;

pub fn store(app: &AppHandle) -> Result<Arc<Store>, Error> {
    app.store_builder("store.json")
        .auto_save(Duration::from_millis(100))
        .build()
}
