use std::collections::HashMap;

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

pub fn grammar_get_root_level_keywords() -> &'static [&'static str] {
    &[
        "source",
        "filter",
        "parser",
        "rewrite",
        "destination",
        "log",
        "junction",
    ]
}

fn grammar_get_destinations() -> Option<&'static Value> {
    let options = get_options().as_object()?;
    Some(options.get("destination")?)
}

fn grammar_get_sources() -> Option<&'static Value> {
    let options = get_options().as_object()?;
    Some(options.get("source")?)
}

fn grammar_get_parsers() -> Option<&'static Value> {
    let options = get_options().as_object()?;
    Some(options.get("parser")?)
}

pub fn get_possible_object_names(object_kind: &str) -> Option<Vec<&str>> {
    get_possible_values_for_type(object_kind)
}

fn get_possible_values_for_type(object_type: &str) -> Option<Vec<&str>> {
    let mut result = Vec::new();

    let target = match object_type {
        "destination" => grammar_get_destinations()?.as_object()?,
        "source" => grammar_get_sources()?.as_object()?,
        "parser" => grammar_get_parsers()?.as_object()?,
        _ => return None,
    };

    for (name, value) in target.iter() {
        result.push(name.as_str())
    }

    Some(result)
}

fn remove_surronding_quotes(inp: &str) -> &str {
    if let (Some(left_quote_ind), Some(right_quote_ind)) = (inp.find('"'), inp.rfind('"')) {
        assert!(left_quote_ind == 0 && right_quote_ind == inp.len() - 1);

        if inp != "\"\"" {
            &inp[1..inp.len() - 1]
        } else {
            inp
        }
    } else {
        inp
    }
}

pub fn grammar_get_all_options(object_type: &str, name: &str) -> Option<HashMap<String, String>> {
    let options = get_options().as_object()?;
    let object_options = options.get(object_type)?.as_object()?;
    let object_options = object_options.get(name)?.as_object()?;

    let object_options_array = object_options.get("options")?.as_array()?;

    let mut result = HashMap::new();
    for kv_arr in object_options_array {
        let mut current_option = kv_arr.as_array()?.get(0)?.as_str()?;

        // option_name1/deprecated_name2/depracated_name3/...
        if current_option.contains("/") {
            let split = current_option.split("/");
            let vec: Vec<&str> = split.collect();

            current_option = vec[0];
        } else {
        }

        let current_option = current_option;
        let option_type = kv_arr.as_array()?.get(1)?.as_array()?.get(0);

        match option_type {
            None => {
                break;
            }
            Some(value) => {
                let option_type = value.as_str()?;

                result.insert(
                    // option, (<option_type>)
                    remove_surronding_quotes(current_option),
                    format!("({})", remove_surronding_quotes(option_type)).to_string(),
                );
            }
        }
    }

    Some(result)
}
