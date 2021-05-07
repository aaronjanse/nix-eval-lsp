#![warn(clippy::unwrap_used)]

mod builtins;
mod eval;
mod parse;
mod scope;
mod tests;
mod utils;
mod value;

use eval::Tree;
use gc::{Finalize, Trace};
use gc::{Gc, GcCell};
use log::{error, warn};
use lsp_server::{Connection, ErrorCode, Message, Request, RequestId, Response};
use lsp_types::{
    notification::*,
    request::{Completion, GotoDefinition, HoverRequest},
    *,
};
use rnix::parser::*;
use rnix::types::Wrapper;
use scope::*;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::{collections::HashMap, panic, path::PathBuf};
use value::*;

#[derive(Debug, Clone, Trace, Finalize)]
pub enum EvalError {
    Unimplemented(String),
    Unexpected(String),
    StackTrace(String),
    TypeError(String),
    Parsing,
    Unknown,
}

impl std::error::Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::Unimplemented(msg) => write!(f, "unimplemented: {}", msg),
            EvalError::Unexpected(msg) => write!(f, "unexpected: {}", msg),
            EvalError::StackTrace(msg) => write!(f, "{}", msg),
            EvalError::TypeError(msg) => write!(f, "type error: {}", msg),
            EvalError::Parsing => write!(f, "parsing error"),
            EvalError::Unknown => write!(f, "unknown value"),
        }
    }
}

impl From<&EvalError> for EvalError {
    fn from(x: &EvalError) -> Self {
        x.clone()
    }
}

type FileMap = HashMap<Url, (AST, String, Result<Tree, EvalError>)>;

fn main() {
    panic::set_hook(Box::new(|x| {
        error!("{}", x);
    }));

    let (connection, io_threads) = Connection::stdio();
    let capabilities = serde_json::to_value(&ServerCapabilities {
        completion_provider: Some(CompletionOptions::default()),
        definition_provider: Some(true),
        hover_provider: Some(true),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                change: Some(TextDocumentSyncKind::Full),
                open_close: Some(true),
                ..TextDocumentSyncOptions::default()
            },
        )),
        ..ServerCapabilities::default()
    })
    .expect("failed to convert capabilities to json");

    connection
        .initialize(capabilities)
        .expect("failed to initialize connection");

    main_loop(&connection).expect("main loop failed");

    io_threads.join().expect("failed to join io_threads");
}

fn main_loop(connection: &Connection) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut files = HashMap::new();
    let store = Gc::new(GcCell::new(HashMap::new()));

    let reply = |response: Response| {
        connection
            .sender
            .send(Message::Response(response))
            .expect("failed to respond");
    };

    for msg in &connection.receiver {
        match msg {
            Message::Response(_) => (),
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                if let Some((id, params)) = cast::<GotoDefinition>(&req) {
                    if let Some(loc) = handle_goto(&files, params) {
                        reply(Response::new_ok(id, loc))
                    } else {
                        reply(Response::new_ok(id, ()))
                    }
                } else if let Some((id, params)) = cast::<HoverRequest>(&req) {
                    match handle_hover(&files, params) {
                        Some((range, markdown)) => {
                            reply(Response::new_ok(
                                id,
                                Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: markdown,
                                    }),
                                    range,
                                },
                            ));
                        }
                        None => {
                            reply(Response::new_ok(id, ()));
                        }
                    }
                } else if let Some((id, params)) = cast::<Completion>(&req) {
                    let completions = handle_completion(&files, &params.text_document_position)
                        .unwrap_or_default();
                    reply(Response::new_ok(id, completions));
                } else {
                    reply(Response::new_err(
                        req.id,
                        ErrorCode::MethodNotFound as i32,
                        format!("unrecognized request {}", req.method),
                    ))
                }
            }
            Message::Notification(req) => {
                let mut handle = |text: String, uri: Url| {
                    let parsed = rnix::parse(&text);
                    let parsed_root = parsed.root().inner().ok_or(EvalError::Parsing);
                    let path = match PathBuf::from_str(uri.path()) {
                        Ok(x) => x,
                        Err(_) => return,
                    };
                    let gc_root = Gc::new(Scope::Root(path, Some(store.clone())));
                    let evaluated = parsed_root.and_then(|x| Tree::parse_legacy(&x, gc_root));
                    files.insert(uri, (parsed, text, evaluated));
                };

                match &*req.method {
                    DidOpenTextDocument::METHOD => {
                        let params: DidOpenTextDocumentParams =
                            match serde_json::from_value(req.params) {
                                Ok(x) => x,
                                Err(_) => continue,
                            };
                        handle(params.text_document.text, params.text_document.uri);
                    }
                    DidChangeTextDocument::METHOD => {
                        let params: DidChangeTextDocumentParams =
                            match serde_json::from_value(req.params) {
                                Ok(x) => x,
                                Err(_) => continue,
                            };
                        for change in params.content_changes.into_iter() {
                            handle(change.text, params.text_document.uri.clone());
                        }
                    }
                    _ => continue,
                }
            }
        }
    }
    Ok(())
}

fn cast<R>(req: &Request) -> Option<(RequestId, R::Params)>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.clone().extract(R::METHOD).ok()
}

fn handle_goto(files: &FileMap, params: TextDocumentPositionParams) -> Option<Location> {
    let (_, content, tree) = files.get(&params.text_document.uri)?;
    let offset = utils::lookup_pos(content, params.position)?;
    let tmp = Gc::new(tree.clone().ok()?);
    let tree = climb_tree(&tmp, offset);
    let def = tree.get_definition()?;
    let def_path = def.scope.root_path()?;
    let code = std::fs::read_to_string(&def_path).ok()?;
    Some(Location {
        uri: Url::parse(&format!("file://{}", def_path.to_string_lossy())).ok()?,
        range: utils::range(&code, def.range?),
    })
}

fn handle_hover(
    files: &FileMap,
    params: TextDocumentPositionParams,
) -> Option<(Option<Range>, String)> {
    let (_, content, tree) = files.get(&params.text_document.uri)?;
    let offset = utils::lookup_pos(content, params.position)?;
    let tree = match tree {
        Ok(x) => {
            let tmp = Gc::new(x.clone());
            climb_tree(&tmp, offset).clone()
        }
        Err(e) => return Some((None, format!("{:?}", e))),
    };
    let range = utils::range(content, tree.range?);
    let val = tree.eval().ok()?;
    Some((Some(range), val.format_markdown()))
}

fn handle_completion(
    files: &FileMap,
    params: &TextDocumentPositionParams,
) -> Option<Vec<CompletionItem>> {
    let (_, content, tree) = files.get(&params.text_document.uri)?;
    let offset = utils::lookup_pos(content, params.position)?;
    let tree = match tree {
        Ok(x) => {
            let tmp = Gc::new(x.clone());
            climb_tree(&tmp, offset).clone()
        }
        Err(_) => return None,
    };

    let (prefix, names, range) = tree.completions()?;
    let mut completions = Vec::new();
    for name in names {
        if name.starts_with(&prefix) {
            completions.push(CompletionItem {
                label: name.clone(),
                text_edit: Some(TextEdit {
                    range: utils::range(content, range),
                    new_text: name,
                }),
                kind: Some(CompletionItemKind::Variable),
                ..CompletionItem::default()
            });
        }
    }
    Some(completions)
}

fn climb_tree(here: &Gc<Tree>, offset: usize) -> &Gc<Tree> {
    for child in here.children().clone() {
        let range = match child.range {
            Some(x) => x,
            None => continue,
        };
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        if start <= offset && offset <= end {
            return climb_tree(child, offset);
        }
    }
    here
}
