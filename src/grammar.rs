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


pub fn grammar_get_all_options(object_type: &str, name:&str) -> Option<Vec<String>> {
    let options = get_options();

    let object_options = options.get(object_type)?.get(name)?.get("options")?;

    let object_options_array = object_options.as_array()?;

    
    let mut result = Vec::new();
    for kv_arr in object_options_array {

        let mut current_option = kv_arr.as_array()?.get(0)?.as_str()?;

        // option_name1/deprecated_name2/depracated_name3/...
        if current_option.contains("/"){
            let split  = current_option.split("/");
            let vec: Vec<&str> = split.collect();

            current_option = vec[0];
        }else { }

        let option_type =  kv_arr.as_array()?.get(1)?.as_str()?;
        result.push(
            // option(<option_type>)
            format!("{}({})", current_option, option_type).to_string()
        );
    }
    Some(result)
}