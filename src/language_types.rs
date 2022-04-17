pub mod annotations{
    pub struct DefineAnnotation {
        pub key: String,
        pub value: String
    }

    #[derive(Debug)]
    pub struct VersionAnnotation{
        pub major_version: u8,
        pub minor_version: u8,
    }

    pub struct IncludeAnnotation{
        pub path: String,
        pub content: String
    }

    pub struct ModuleAnnotation{

    }
}

#[derive(Debug)]
pub struct GlobalOption {
    name: String
}

pub mod objects{
    use tower_lsp::lsp_types::TextDocumentPositionParams;

    use crate::ast::Context;


    #[derive(PartialEq, Eq)]
pub enum ObjectKind{
        Source,
        Destination,
        Log,
        Filter,
        Parser,
        RewriteRule,
        Template
    }

    pub struct Parameter{

    }

    pub trait Object: std::fmt::Debug {
        fn get_id(&self) -> &str;
        fn get_optional_parameters(&self) -> Vec<Parameter>;
        fn get_mandatory_parameters(&self) -> Vec<Parameter>;
        fn get_kind(&self) -> ObjectKind;
        fn contains_document_position(&self, text_document_position: &TextDocumentPositionParams) -> bool;
    }

}