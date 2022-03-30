
use tower_lsp::lsp_types::{DidChangeTextDocumentParams, CompletionResponse, Diagnostic, CompletionParams};

use crate::language_types::{objects::{Object, ObjectKind}, GlobalOption, annotations::VersionAnnotation};
pub trait AST{
    fn get_global_options(&self) -> &Vec<GlobalOption>;
    fn get_objects(&self) -> &Vec<Box< dyn Object>>;

    fn get_objects_by_kind(&self, kind: ObjectKind) -> Vec<Box<dyn Object>>;
}

pub struct SyslogNgConfiguration {
    configuration: String,
    version: VersionAnnotation,


    is_valid: bool,
    global_options: Vec<GlobalOption>,
    objects: Vec<Box< dyn Object>>,
    ast: Box<dyn AST>,
}

impl AST for SyslogNgConfiguration {
    fn get_global_options(&self) -> &Vec<GlobalOption> {
        &self.global_options
    }

    fn get_objects(&self) -> &Vec<Box< dyn Object>> {
        &self.objects
    }

    fn get_objects_by_kind(&self, kind: ObjectKind) -> Vec<Box<dyn Object>> {
        self.objects
        .iter()
        .filter(
            |o| o.get_kind() == kind)
        .collect()
    }
}

pub trait ParsedConfiguration: AST {

    fn validate(&self);
    
    fn get_AST(&self) -> &Box<dyn AST>;
    fn get_diagnostics(&self) -> Vec<Diagnostic>;
    fn get_code_completion(&self, params: &CompletionParams) -> Option<Vec<CompletionResponse>>;


    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams);



}

impl ParsedConfiguration for SyslogNgConfiguration {
    fn validate(&self) {
        todo!()
    }

    fn get_AST(&self) -> &Box<dyn AST> {
        &self.ast
    }

    fn get_diagnostics(&self) -> Vec<Diagnostic> {
        todo!()
    }

    fn get_code_completion(&self, params: &CompletionParams) -> Option<Vec<CompletionResponse>> {
        todo!()
    }

    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams) {
        todo!()
    }
}