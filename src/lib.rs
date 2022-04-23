pub mod language_types;
pub mod parser;
pub mod ast;
pub mod grammar;
mod file_utilities;


use std::sync::{Arc, RwLock};

use ast::SyslogNgConfiguration;
use grammar::{grammar_init};
use serde_json::Value;
use tower_lsp::jsonrpc::{Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};


pub enum ServerErrorCodes {
    CompletionError = 0,
}

pub struct Backend {
    pub client: Client,
    pub configuration: &'static Arc<RwLock<SyslogNgConfiguration>>
}

impl Backend {

    fn init_configuraton(&self, configuration: &str, URI: &TextDocumentIdentifier) {
        let config_lock = &self.configuration.clone();

        if let Ok(mut write_guard) = config_lock.write() {
            let mut assembled_config = &mut *write_guard;
            &assembled_config.add_configuration(&configuration, &URI);
        };
    }

    fn update_configuration(&self) {}

    fn process_config(&self) {}

    
    
    pub fn get_possible_completion(&self, params: &CompletionParams) -> Option<CompletionResponse> {
todo!()

    }

    
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, ip: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, format!("initialized: {:?}", ip.workspace_folders).to_owned())
            .await;
        Ok(InitializeResult {
            server_info: Some(ServerInfo{
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
        let content = &doc.text_document.text;
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
        self.client.log_message(MessageType::INFO, format!("{:?}", params));
        // Ok(Some(CompletionResponse::Array(vec![
           // CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            // CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        // ])))
        grammar_init();
        let res = Backend::get_possible_completion(&self, &params);

        match res {

            Some(val) => {
                Ok(Some(val))
            },

            None => Ok(None)

            // _ => Err(Error::new(tower_lsp::jsonrpc::ErrorCode::ServerError(ServerErrorCodes::CompletionError as i64)))
        }
    }
}