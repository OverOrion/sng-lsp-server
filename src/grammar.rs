use once_cell::sync::OnceCell;
use serde_json::Value;

const CONFIG_OPTIONS_DATABASE: &str = include_str!("../config-options-database/database.json");
pub static CONFIG_OPTIONS: OnceCell<Value> = OnceCell::new();


pub fn grammar_init() -> () {
    CONFIG_OPTIONS.set(serde_json::from_str(CONFIG_OPTIONS_DATABASE).unwrap());
}

fn get_options() -> &'static Value {
    CONFIG_OPTIONS.get().expect("Getting grammar failed")
}


pub fn grammar_get_destinations() -> &'static Value {
    let options = get_options();
    &options["destination"]
}

pub fn grammar_get_sources() -> &'static Value {
    let options = get_options();
    &options["source"]
}

pub fn  grammar_get_parsers() -> &'static Value {
    let options = get_options();
    &options["parser"]
}