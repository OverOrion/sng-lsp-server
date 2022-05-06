
use std::{cmp::Ordering, convert::From, sync::{RwLock, Arc}, collections::HashMap};


use tower_lsp::lsp_types::{DidChangeTextDocumentParams, CompletionResponse, Diagnostic, CompletionParams, Position, TextDocumentIdentifier, CompletionItem, self, DiagnosticSeverity, Url};

use crate::{language_types::{objects::{Object, ObjectKind, self}, GlobalOption, annotations::{VersionAnnotation, IncludeAnnotation}}, grammar::{grammar_get_all_options, grammar_get_root_level_keywords}, parser::{Annotation, try_parse_configuration}, file_utilities::{get_block_by_position, get_driver_before_position}};



pub enum Context {
    Root,

    // Same as enum ObjectKind {}
    Source,
    Destination,
    Log,
    Filter,
    Parser,
    RewriteRule,
    Template
}

impl From<&ObjectKind> for Context {
    fn from (item: &ObjectKind) -> Self {
        match item {
                ObjectKind::Source => Context::Source,
                ObjectKind::Destination => Context::Destination,
                ObjectKind::Log => Context::Log,
                ObjectKind::Filter => Context::Filter,
                ObjectKind::Parser => Context::Parser,
                ObjectKind::RewriteRule => Context::RewriteRule,
                ObjectKind::Template => Context::Template,

                _ => Context::Root
        }
    }
}

pub trait AST{
    fn get_global_options(&self) -> &Vec<GlobalOption>;
    fn get_objects(&self) -> &Vec<Object>;

    fn get_objects_by_kind(&self, kind: &ObjectKind) -> Vec<&Object>;
}



#[derive(Debug)]
pub struct Snippet {
    pub content: String,
    pub include_range: lsp_types::Range,
    pub snippet_uri: TextDocumentIdentifier,
    pub diagnostics: Vec<Diagnostic>,

    pub included_snippets: Option<Vec<Snippet>>,
    pub resolved_content: String,
    pub depth: u8,
}

impl Snippet {

    fn check_possible_errors(&self, depth: u8) -> Option<Diagnostic> {
        const MAX_DEPTH: u8 = 15;
        let source = "syslog-ng LSP server";

        if depth > MAX_DEPTH {
            return Some(Diagnostic::new(
                    self.get_whole_content_range(),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    Some(source.to_string()),
                    format!("Include limit ({}) has been reached, diagnostics might be unreliable. Make sure there are no circular @include directives", MAX_DEPTH),
                    None,
                    None
                ));
        }
        
        if let Some(version_range) = self.get_range_by_pattern("@version") {
            return Some(
                Diagnostic::new(
                    version_range,
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    Some(source.to_string()),
                    format!("Snippets can not contain @version"),
                    None,
                    None,
                ));
        }
        
        None
    }

    fn resolve_include(&mut self, depth: u8) -> Result<String, Diagnostic> {
        if let Some(diag) = self.check_possible_errors(depth) {
            self.diagnostics.push(diag.clone());
            return Err(diag);
        }

        let mut merged_content = String::new();

        if self.has_includes() {
            let included_snippets :&mut Vec<Snippet> = self.included_snippets.as_mut().unwrap();
            // recursively

            // sort them
            included_snippets.sort();

            // get list of included files
            // resolve them
            for snippet in included_snippets.iter_mut() {
                let res = snippet.resolve_include(depth+1);
                match res {
                    Ok(sub_snippet_merged_content) => {
                        merged_content.push_str(&sub_snippet_merged_content);
                    }
                    Err(sub_snippet_diag) => {
                        // report diag to includer
                        return Err(Diagnostic::new(
                            snippet.include_range,
                            Some(DiagnosticSeverity::ERROR),
                            None,
                            None,
                            format!("Included file {:#?} has errors in it", snippet.get_snippet_uri()),
                            None,
                            None
                        ));
                    }
                }
            }
        }


        // resolve self
        self.resolved_content = merged_content;
        // try_parse_snippet(&self.resolved_content);
todo!();
        // Ok(())


    }

    fn has_includes(&self) -> bool {
        return self.content.contains("@include");
    }

    pub fn get_resolved_merged(&self) -> String {

        let mut merged = String::new();

        if let Some(includes) = &self.included_snippets {
            for snippet in includes {
                let res = snippet.get_resolved_merged();
                merged.push_str(&res);
            }
        }

        merged.push_str(&self.content);
        merged

    }


    fn get_whole_content_range(&self) -> lsp_types::Range {

        let num_of_lines = self.content.lines().count();

        lsp_types::Range::new(
            Position{line: 0, character: 0 },
            Position{line: num_of_lines as u32 + 1, character: 0}
        )
    }

    fn get_range_by_pattern(&self, pattern: &str) -> Option<lsp_types::Range> {
        let mut starting_line: usize  = 0;
        
        for line in self.content.lines() {
            if line.contains(pattern) {
                return Some(lsp_types::Range::new(
                    Position{ line: starting_line as u32, character: 0 },
                    Position{line: starting_line as u32 + 1, character: 0}));
            }
            else {
                starting_line += 1;
            }
        }
        None
    }

    /// Get a reference to the snippet's snippet uri.
    pub fn get_snippet_uri(&self) -> &TextDocumentIdentifier {
        &self.snippet_uri
    }
}

impl Ord for Snippet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.snippet_uri.uri.cmp(&other.snippet_uri.uri)
    }
}

impl PartialOrd for Snippet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Snippet {
    fn eq(&self, other: &Self) -> bool {
        self.snippet_uri == other.snippet_uri
    }
}

impl Eq for Snippet {}



#[derive(Debug)]
pub struct SyslogNgConfiguration {
    configuration: String,
    // configuration_URI: TextDocumentIdentifier,
    version: VersionAnnotation,

    includes: Vec<IncludeAnnotation>,
    snippets: HashMap<String, Snippet>,

    workspace_folder: Option<Url>,
    is_valid: bool,
    global_options: Vec<GlobalOption>,
    objects: Vec<Object>,
    diagnostics: Vec<(String, Diagnostic)>
}

impl SyslogNgConfiguration {
    fn init_new() -> SyslogNgConfiguration{
        SyslogNgConfiguration{
            configuration: String::new(),
            version: VersionAnnotation{
                major_version: 0,
                minor_version: 0
            },
            // configuration_URI: TextDocumentIdentifier::new(Url::parse("syslog-ng.conf").unwrap()),
            includes: Vec::new(),
            snippets: HashMap::new(),

            workspace_folder: None,

            is_valid: false,
            global_options: Vec::new(),
            objects: Vec::new(),
            diagnostics: Vec::new(),
            
        }
    }

    pub fn new() -> Arc<RwLock<SyslogNgConfiguration>> {
        Arc::new(RwLock::new(SyslogNgConfiguration::init_new()))
    }

    // pub fn add_configuration(&mut self, conf: &str, URI: &TextDocumentIdentifier) {
    pub fn add_configuration(&mut self, conf: &str) {
        // if has @version => main config
        if conf.contains("@version") {
            self.configuration.push_str(conf);
            // self.configuration_URI = URI.clone();

            let conf_ro = &self.configuration.clone();

            try_parse_configuration(conf_ro, self)
        }
    }

    pub fn add_snippet(&mut self, snippet: Snippet) {
        self.snippets.insert(snippet.get_snippet_uri().uri.to_string(), snippet);

    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        match annotation {
            Annotation::VA(version) => self.version = version,
            Annotation::IA(include) => {
                if let Some(include) = include {
                    self.includes.push(include)
                }
            },
        }
    }

    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(obj);
    }

    pub fn transform_grammar_option_to_completion_response(label: &str, details: &str) -> CompletionItem {
        // inp := option_name(<option_type>)
        CompletionItem::new_simple(label.to_string(), details.to_owned())
    }

    pub fn set_workspace_folder(&mut self, url: &Url) {
        self.workspace_folder = Some(url.to_owned())
    }
}

impl AST for SyslogNgConfiguration {
    fn get_global_options(&self) -> &Vec<GlobalOption> {
        &self.global_options
    }

    fn get_objects(&self) -> &Vec<Object> {
        &self.objects
    }

    fn get_objects_by_kind(&self, kind: &ObjectKind) -> Vec<&objects::Object>{
        self.objects
        .iter()
        .filter(
            |o| o.get_kind() == kind)
        .collect()
    }
}

pub trait ParsedConfiguration: AST {

    fn validate(&self);
    
    fn get_diagnostics(&self) -> Vec<Diagnostic>;
    fn get_code_completion(&self, params: &CompletionParams) -> Option<CompletionResponse>;
    fn get_context(&self, params: &CompletionParams) -> Context;

    fn is_inside_concrete_driver(&self, params:&CompletionParams) -> Option<String>;


    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams);

    fn add_diagnostics(&mut self, diag: Diagnostic);



}

impl ParsedConfiguration for SyslogNgConfiguration {
    fn validate(&self) {
        todo!()
    }

    fn get_diagnostics(&self) -> Vec<Diagnostic> {
        todo!()
    }

    fn get_code_completion(&self, params: &CompletionParams) -> Option<CompletionResponse> {
        let mut response:Vec<CompletionItem> = Vec::new();
        let mut object_type = String::from("");
        
        // let object_in = 
        let context = self.get_context(params);
        match context {
            Context::Root => {
                for kw in grammar_get_root_level_keywords().into_iter() {
                    let item = SyslogNgConfiguration::transform_grammar_option_to_completion_response(*kw, *kw);
                    response.push(item);
                }
                return Some(CompletionResponse::Array(response));
            }

            Context::Source => object_type.push_str("source"),
            Context::Destination => object_type.push_str("destination"),
            Context::Parser => object_type.push_str("parser"),

            // Get exsiting object suggestions
            Context::Log => todo!(),

            Context::Filter => todo!(),
            Context::RewriteRule => todo!(),
            Context::Template => todo!(),
        }

        let uri = params.text_document_position.text_document.uri.as_str();
        let line_num = params.text_document_position.position.line;

        let driver = get_driver_before_position(uri, line_num);
        let inner_block = get_block_by_position(uri, line_num);
        if let Some(driver) = driver {
            let mut res:Vec<CompletionItem> 
            = grammar_get_all_options(&object_type, &driver, &inner_block)?
            .into_iter()
            .map(|(label, details)| SyslogNgConfiguration::transform_grammar_option_to_completion_response(&label, &details))
            .collect();
            response.append(&mut res);
            return Some(CompletionResponse::Array(response));
        }

        None

        // from user
    }

    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams) {
        todo!()
    }
    

    fn get_context(&self, params: &CompletionParams) -> Context {
        let text_document_position = &params.text_document_position;

        for obj in self.get_objects() {
            if obj.is_inside_document_position(text_document_position) {
                return Context::from(obj.get_kind());
            }
        }

        // root
        Context::Root
    }

    fn is_inside_concrete_driver(&self, params: &CompletionParams) -> Option<String> {

        let uri = params.text_document_position.text_document.uri.as_str();
        let line_num = params.text_document_position.position.line;

        if let Some(driver) = get_block_by_position(uri, line_num) {
            return Some(driver);
        }


        None
    }

    fn add_diagnostics(&mut self, diag: Diagnostic) {
        todo!()



        

    }
    
}