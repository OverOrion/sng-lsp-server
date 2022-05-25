use std::{sync::{Arc, RwLock}, env};

use tower_lsp::{LspService, Server};
use once_cell::sync::OnceCell;

use lsp_syslog_ng::{Backend, ast::SyslogNgConfiguration};

extern crate glob;


static CONFIGURATION: OnceCell<Arc<RwLock<SyslogNgConfiguration>>> = OnceCell::new();


#[tokio::main]
async fn main() {

    env::set_var("RUST_BACKTRACE", "1");


    // Empty configuration

    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    env_logger::init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    CONFIGURATION.set(SyslogNgConfiguration::new()).expect("Global initialization failed");
    
    let (service, messages) = LspService::new(|client| Backend { client, configuration: &CONFIGURATION.get().expect("Acquiring configuration failed") });
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;

}