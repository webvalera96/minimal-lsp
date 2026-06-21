use std::error::Error;
use std::net::SocketAddr;

use env_logger;

use log::{debug, error, log_enabled, Level, info};
use lsp_server::{Connection, Message, IoThreads};
use lsp_types::notification::Notification as _;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    },
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    env_logger::init();

    debug!("starting minimal-lsp");

    let connection: Connection;
    let io_threads: IoThreads;

    // in release mod, work in stdio
    if log_enabled!(Level::Debug) {
        debug!("starting in listen mode");
        let addr: SocketAddr = "127.0.0.1:9090".parse()?;
        (connection, io_threads) = Connection::listen(addr)?;
    } else { // in debug mode, work as server
        (connection, io_threads) = Connection::stdio();
        info!("starting in stdio mode");
    }

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    };

    let init_result = serde_json::json!({
        "capabilities": capabilities,
    });

    let _init_params = connection.initialize(init_result)?;
    main_loop(&connection)?;
    io_threads.join()?;
    eprintln!("shutting down minimal-lsp");
    Ok(())
}

fn main_loop(connection: &Connection) -> Result<(), Box<dyn Error + Sync + Send>> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                eprintln!("[lsp] unhandled request: {}", req.method);
            }
            Message::Notification(not) => {
                handle_notification(not);
            }
            Message::Response(resp) => {
                eprintln!("[lsp] response: {resp:?}");
            }
        }
    }
    Ok(())
}

fn handle_notification(not: lsp_server::Notification) {
    match not.method.as_str() {
        DidOpenTextDocument::METHOD => {
            if let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(not.params) {
                eprintln!(
                    "[lsp] didOpen: uri={}, language={}, version={}",
                    params.text_document.uri,
                    params.text_document.language_id,
                    params.text_document.version,
                );
            }
        }
        DidChangeTextDocument::METHOD => {
            if let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(not.params) {
                eprintln!(
                    "[lsp] didChange: uri={}, version={}",
                    params.text_document.uri, params.text_document.version,
                );
            }
        }
        DidSaveTextDocument::METHOD => {
            if let Ok(params) = serde_json::from_value::<DidSaveTextDocumentParams>(not.params) {
                eprintln!("[lsp] didSave: uri={}", params.text_document.uri);
            }
        }
        DidCloseTextDocument::METHOD => {
            if let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(not.params) {
                eprintln!("[lsp] didClose: uri={}", params.text_document.uri);
            }
        }
        _ => {
            eprintln!("[lsp] unhandled notification: {}", not.method);
        }
    }
}
