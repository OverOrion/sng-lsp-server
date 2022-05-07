pub mod ast;
mod file_utilities;
pub mod grammar;
pub mod language_types;
pub mod parser;

use std::sync::{Arc, RwLock};

use ast::{SyslogNgConfiguration, ParsedConfiguration};
use grammar::grammar_init;
use parser::parse_conf;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub enum ServerErrorCodes {
    CompletionError = 0,
}

pub struct Backend {
    pub client: Client,
    pub configuration: &'static Arc<RwLock<SyslogNgConfiguration>>,
}

impl Backend {
    // fn init_configuraton(&self, configuration: &str, URI: &TextDocumentIdentifier) {
    //     let config_lock = &self.configuration.clone();

    //     if let Ok(mut write_guard) = config_lock.write() {
    //         let mut assembled_config = &mut *write_guard;
    //         &assembled_config.add_configuration(&configuration, &URI);
    //     };
    // }

    fn update_configuration(&self) {}

    fn process_config(&self, content: &str, file_url: &str) {
        let config_lock = &self.configuration.clone();

        if let Ok(mut write_guard) = config_lock.write() {
            let mut conf = &mut *write_guard;
            conf.add_configuration(content);

            // parse_conf(&content, file_url, conf);
        };

    }

    pub fn set_workspace_folder(&self, url: &Url) {
        let config_lock = &self.configuration.clone();

        if let Ok(mut write_guard) = config_lock.write() {
            let mut conf = &mut *write_guard;
            conf.set_workspace_folder(url)
        };
    }

    pub fn get_possible_completion(&self, params: &CompletionParams) -> Option<CompletionResponse> {
        let config_lock = &self.configuration.clone();

        if let Ok(read_guard) = config_lock.read() {
            let conf: &dyn ParsedConfiguration = &*read_guard;
            return conf.get_code_completion(params);
        }

       None 
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, initialize_params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("initialized: {:?}", initialize_params.root_uri).to_owned(),
            )
            .await;
        if let Some(workspace_folder) = &initialize_params.root_uri {
            self.set_workspace_folder(&workspace_folder);
        }
        grammar_init();
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "syslog-ng LSP server".to_string(),
                version: Some("0.1".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                }),
                execute_command_provider: None,
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, doc: DidOpenTextDocumentParams) {
        //TODO build AST

        // find main


        let content = &doc.text_document.text;
        let file_url = &doc.text_document.uri.as_str();
        self.process_config(&content, &file_url);


        // 
       
        self.client
            .log_message(MessageType::INFO, "file opened: ".to_owned() + &content)
            .await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, format!("{:?}", params));
        let res = Backend::get_possible_completion(&self, &params);

        match res {
            Some(val) => Ok(Some(val)),

            None => Ok(None), // _ => Err(Error::new(tower_lsp::jsonrpc::ErrorCode::ServerError(ServerErrorCodes::CompletionError as i64)))
        }
    }
}
