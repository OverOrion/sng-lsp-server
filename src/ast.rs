
use std::{sync::{RwLock, Arc}, ops::Range};
use std::convert::From;


use tower_lsp::lsp_types::{DidChangeTextDocumentParams, CompletionResponse, Diagnostic, CompletionParams, Position, TextDocumentIdentifier, Url, MessageType, CompletionItem};

use crate::{language_types::{objects::{Object, ObjectKind}, GlobalOption, annotations::VersionAnnotation}, grammar::{grammar_get_all_options, grammar_get_root_level_keywords}};


pub struct LSPMessage {
    pub message_type: MessageType,
    pub message: String
}

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

impl From<ObjectKind> for Context {
    fn from (item: ObjectKind) -> Self {
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
    fn get_objects(&self) -> &Vec<Box< dyn Object + Send + Sync>>;

    fn get_objects_by_kind(&self, kind: ObjectKind) -> Vec<&Box<dyn Object + Send + Sync>>;
}

#[derive(Debug)]
pub struct Snippet {
    pub content: String,
    pub include_position: Position,
    pub include_range: Range<Position>,
    snippet_URI: TextDocumentIdentifier,

    pub merged_content: String,

    pub included_snippets: Option<Vec<Snippet>>,
    pub depth: u8
}

impl Snippet {

    fn resolve_include(&mut self, depth: u8) -> Result<(), LSPMessage>{
        const MAX_DEPTH: u8 = 15;

        if depth > MAX_DEPTH {
            Err(LSPMessage {
                message_type: MessageType::ERROR,
                message: format!("Include limit ({}) has been reached, diagnostics might be unreliable", MAX_DEPTH)
            })
        
        } else if self.content.contains("@version") {
            Err(LSPMessage{
                message_type: MessageType::ERROR,
                message: format!("Snippets can not contain @version")

            })

        } else {
            if self.has_includes() {
                // recursively 

                // get list of included files

                // sort them
                
                // resolve them

            }


            // resolve self
            //parse_snippet(&self);

            Ok(())
        }

    }

    fn has_includes(&self) -> bool {
        return self.content.contains("@include");
    }

}



#[derive(Debug)]
pub struct SyslogNgConfiguration {
    configuration: String,
    // configuration_URI: TextDocumentIdentifier,
    version: VersionAnnotation,
    snippets: Vec<Snippet>,


    is_valid: bool,
    global_options: Vec<GlobalOption>,
    objects: Vec<Box< dyn Object + Send + Sync>>,
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
            snippets: Vec::new(),

            is_valid: false,
            global_options: Vec::new(),
            objects: Vec::new(),
            
        }
    }

    pub fn new() -> Arc<RwLock<SyslogNgConfiguration>> {
        Arc::new(RwLock::new(SyslogNgConfiguration::init_new()))

    }

    pub fn add_configuration(&mut self, conf: &str, URI: &TextDocumentIdentifier) {
        // if has @version => main config
        if conf.contains("@version") {
            self.configuration.push_str(conf);
            // self.configuration_URI = URI.clone();
        }
    }

    pub fn add_snippet(&mut self, snippet: Snippet) {
        self.snippets.push(snippet);

    }

    pub fn transform_grammar_option_to_completion_response(label: &str, details: &str) -> CompletionItem {
        // inp := option_name(<option_type>)
        CompletionItem::new_simple(label.to_string(), details.to_owned())
    }
}

impl AST for SyslogNgConfiguration {
    fn get_global_options(&self) -> &Vec<GlobalOption> {
        &self.global_options
    }

    fn get_objects(&self) -> &Vec<Box< dyn Object + Send + Sync>> {
        &self.objects
    }

    fn get_objects_by_kind(&self, kind: ObjectKind) -> Vec<&Box<dyn Object + Send + Sync>> {
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

    fn is_inside_concrete_driver(&self, params:&CompletionParams) -> bool;


    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams);



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
        


        let context = ParsedConfiguration::get_context(self, params);

        match context {
            Context::Root => {
                for kw in grammar_get_root_level_keywords().into_iter() {
                    let item = SyslogNgConfiguration::transform_grammar_option_to_completion_response(*kw, *kw);
                    response.push(item);
                }
                return Some(CompletionResponse::Array(response));
            }

            Context::Source => {    


            },
            Context::Destination => todo!(),
            Context::Parser => todo!(),

            Context::Log => todo!(),

            Context::Filter => todo!(),
            Context::RewriteRule => todo!(),
            Context::Template => todo!(),
        }

        // from db
        let results = grammar_get_all_options("destination", "tcp")?;

        let mut response = Vec::new();

        for kv in results {
            let item = SyslogNgConfiguration::transform_grammar_option_to_completion_response(&kv);
            response.push(item);
        }

        if response.len() > 0 {
            Some(CompletionResponse::Array(response))

        }else {
            None
        }

        // from user
    }

    fn apply_diff(&mut self, content_changes: DidChangeTextDocumentParams) {
        todo!()
    }
    

    fn get_context(&self, params: &CompletionParams) -> Context {
        let text_document_position = &params.text_document_position;

        for obj in self.get_objects() {
            if obj.contains_document_position(text_document_position) {
                return Context::from(obj.get_kind());
            }
        }

        // root
        Context::Root
    }

    fn is_inside_concrete_driver(&self, params:&CompletionParams) -> bool {

        if 

        false
    }
    
}