pub mod annotations {
    pub struct DefineAnnotation {
        pub key: String,
        pub value: String,
    }

    #[derive(Debug)]
    pub struct VersionAnnotation {
        pub major_version: u8,
        pub minor_version: u8,
    }

    pub type IncludeAnnotation = String;
}

#[derive(Debug)]
pub struct GlobalOption {
    name: String,
}

pub mod objects {
    use core::fmt;
    use std::collections::HashMap;

    use tower_lsp::lsp_types::{self, TextDocumentIdentifier, TextDocumentPositionParams};

    use crate::parser::ValueTypes;

    #[derive(PartialEq, Eq, Debug)]
    pub enum ObjectKind {
        Source,
        Destination,
        Log,
        Filter,
        Parser,
        RewriteRule,
        Template,
    }

    impl fmt::Display for ObjectKind {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ObjectKind::Source => write!(f, "source"),
                ObjectKind::Destination => write!(f, "destination"),
                ObjectKind::Log => write!(f, "log"),
                ObjectKind::Filter => write!(f, "filter"),
                ObjectKind::Parser => write!(f, "parser"),
                ObjectKind::RewriteRule => write!(f, "rewrite"),
                ObjectKind::Template => write!(f, "template"),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Driver {
        pub name: String,
        pub required_options: Vec<ValueTypes>,
        pub options: HashMap<String, Parameter>,
    }

    impl Driver {
        pub fn new(name: String, required_options: Vec<ValueTypes>, options: HashMap<String, Parameter>) -> Driver{
            Driver {
                name,
                required_options,
                options
            }
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }
        
        pub fn get_options(&self) -> &HashMap<String, Parameter> {
            &self.options
        }

        pub fn get_required_options(&self) -> &Vec<ValueTypes>{
            &self.required_options
        }

    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Parameter {
        pub option_name: String,
        pub value_type: ValueTypes,
        //pub inner_blocks: Option<Vec<Parameter>>,
    }
    impl Parameter {
        pub fn new(option_name: String, value_type: ValueTypes, //inner_blocks: Option<Vec<Parameter>>
        ) -> Parameter {
            Parameter {
                option_name,
                value_type,
                //inner_blocks,
            }
        }

        pub fn get_option_name(&self) -> &str {
            &self.option_name
        }

        pub fn get_value_type(&self) -> &ValueTypes {
            &self.value_type
        }
    }

    #[derive(Debug)]
    pub struct Object {
        id: String,
        kind: ObjectKind,
        drivers: Vec<Driver>,
        location: Option<(TextDocumentIdentifier, lsp_types::Range)>,
    }

    impl Object {
        pub fn new(
            id: String,
            kind: ObjectKind,
            drivers: Vec<Driver>,
            location: Option<(TextDocumentIdentifier, lsp_types::Range)>
        ) -> Object {
            Object {
                id,
                kind,
                drivers,
                location,
            }
        }
        pub fn new_without_location(
            id: String,
            kind: ObjectKind,
            options: Vec<Driver>,
        ) -> Object { Object::new(id, kind, options, None)
        }
        
        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn get_drivers(&self) -> &Vec<Driver> {
            &self.drivers
        }

        pub fn get_kind(&self) -> &ObjectKind {
            &self.kind
        }

        pub fn get_location(&self) -> &Option<(TextDocumentIdentifier, lsp_types::Range)> {
            &self.location
        }

        pub fn get_start_and_end_position(&self) -> Option<&lsp_types::Range> {
            if let Some(loc) = &self.location {
                return Some(&loc.1);
            }
            None
        }

        pub fn is_inside_document_position(
            &self,
            text_document_position: &TextDocumentPositionParams,
        ) -> bool {
            let (self_uri, self_range) = &self.location.as_ref().unwrap();

            let self_start_pos = self_range.start;
            let self_end_pos = self_range.end;

            text_document_position.text_document == *self_uri
                && self_start_pos <= text_document_position.position
                && text_document_position.position <= self_end_pos
        }

        pub fn set_location(&mut self, uri: &TextDocumentIdentifier, range: &lsp_types::Range) {
            self.location = Some((uri.clone(), range.clone()));
        }
    }
}
