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

    #[derive(Debug)]
    pub struct Parameter {
        pub option_name: String,
        pub value_type: ValueTypes,
    }
    impl Parameter {
        pub fn new(option_name: String, value_type: ValueTypes) -> Parameter {
            Parameter {
                option_name,
                value_type,
            }
        }
    }

    #[derive(Debug)]
    pub struct Object {
        id: String,
        kind: ObjectKind,
        options: Vec<Parameter>,
        location: Option<(TextDocumentIdentifier, lsp_types::Range)>,
    }

    impl Object {
        pub fn new(
            id: String,
            kind: ObjectKind,
            options: Vec<Parameter>,
            location: Option<(TextDocumentIdentifier, lsp_types::Range)>
        ) -> Object {
            Object {
                id,
                kind,
                options,
                location,
            }
        }
        pub fn new_without_location(
            id: String,
            kind: ObjectKind,
            options: Vec<Parameter>,
        ) -> Object { Object::new(id, kind, options, None)
        }
        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn get_options(&self) -> &Vec<Parameter> {
            &self.options
        }

        pub fn get_kind(&self) -> &ObjectKind {
            &self.kind
        }

        pub fn is_inside_document_position(
            &self,
            text_document_position: &TextDocumentPositionParams,
        ) -> bool {
            let (self_uri, self_range) = &self.location.unwrap();

            let self_start_pos = self_range.start;
            let self_end_pos = self_range.end;

            text_document_position.text_document == *self_uri
                && self_start_pos <= text_document_position.position
                && text_document_position.position <= self_end_pos
        }

        fn set_location(&mut self, uri: &TextDocumentIdentifier, range: &lsp_types::Range) {
            self.location = Some((uri.clone(), range.clone()));
        }
    }
}
