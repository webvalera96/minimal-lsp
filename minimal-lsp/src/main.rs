use std::error::Error;
use std::net::SocketAddr;

use std::collections::HashMap;


use env_logger;

use crossbeam_channel::{SendError};
use log::{debug, error, log_enabled, Level, info};
use lsp_server::{Connection, Message, IoThreads};
use lsp_types::notification::Notification as _;
use lsp_types::{
    notification::{
        DidChangeTextDocument, 
        DidOpenTextDocument,
        PublishDiagnostics,
    },
    DidChangeTextDocumentParams,
    DidOpenTextDocumentParams,
    ServerCapabilities,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    Url,
    Diagnostic,
    Range,
    Position,
    DiagnosticSeverity,
    PublishDiagnosticsParams
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

    // store documents from editor for processing
    let mut docs: HashMap<Url, String> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
            }
            Message::Notification(not) => {
                if let Err(err) = handle_notification(connection,&not, &mut docs) {
                    error!("[lsp] notification {} failed: {err}", not.method.to_string())
                }
            }
            Message::Response(resp) => {
                error!("[lsp] response: {resp:?}")
            }
        }
    }
    Ok(())
}

fn handle_notification(
    conn: &Connection,
    not: &lsp_server::Notification,
    docs: &mut HashMap<Url, String>,
) -> Result<(), String>{
    match not.method.as_str() {

        DidOpenTextDocument::METHOD => {

            if let Ok(p) = serde_json::from_value::<DidOpenTextDocumentParams>(not.params.clone()) {

                let uri = p.text_document.uri;
                docs.insert(uri.clone(), p.text_document.text);
                if let Err(message) = publish_dummy_diag(conn, &uri) {
                    return Err(message.to_string())
                }
            }
        }

        DidChangeTextDocument::METHOD => {
            if let Ok(p) = serde_json::from_value::<DidChangeTextDocumentParams>(not.params.clone()) {
                if let Some(change) = p.content_changes.into_iter().next() {
                    let uri = p.text_document.uri;
                    docs.insert(uri.clone(), change.text);
                    if let Err(message) = publish_dummy_diag(conn, &uri) {
                        return Err(message.to_string())
                    }
                } 
            }
        }
        _ => {}
    }
    Ok(())
}

fn publish_dummy_diag(conn: &Connection, uri: &Url) -> Result<(),SendError<Message>> {
    let diag = Diagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: None,
        code_description: None,
        source: Some("minimal_lsp".into()),
        message: "dummy diagnostic".into(),
        related_information: None,
        tags: None,
        data: None,
    };
    let params =
        PublishDiagnosticsParams { uri: uri.clone(), diagnostics: vec![diag], version: None };
    conn.sender.send(Message::Notification(lsp_server::Notification::new(
        PublishDiagnostics::METHOD.to_owned(),
        params,
    )))?;
    Ok(())
}