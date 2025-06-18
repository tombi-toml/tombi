use std::str::FromStr;

use ahash::AHashMap;
use itertools::Itertools;
use tombi_schema_store::get_tombi_scheme_content;
use tower_lsp::lsp_types::{
    CreateFile, CreateFileOptions, DocumentChangeOperation, DocumentChanges,
    GotoDefinitionResponse, OneOf, OptionalVersionedTextDocumentIdentifier, ResourceOp,
    TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use crate::Backend;

pub async fn into_definition_locations(
    backend: &Backend,
    definitions: Option<Vec<tombi_extension::DefinitionLocation>>,
) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
    let Some(definitions) = definitions else {
        return Ok(None);
    };

    let mut url_set = AHashMap::new();
    for definition in &definitions {
        if let Ok(Some(remote_url)) = open_remote_file(backend, &definition.uri).await {
            url_set.insert(definition.uri.clone(), remote_url);
        }
    }

    let definitions = definitions
        .into_iter()
        .map(|mut definition| {
            if let Some(remote_url) = url_set.get(&definition.uri) {
                definition.uri = remote_url.clone();
            }
            definition
        })
        .collect_vec();

    match definitions.len() {
        0 => Ok(None),
        1 => Ok(Some(GotoDefinitionResponse::Scalar(
            definitions.into_iter().next().unwrap().into(),
        ))),
        _ => Ok(Some(GotoDefinitionResponse::Array(
            definitions
                .into_iter()
                .map(|definition| definition.into())
                .collect(),
        ))),
    }
}

pub async fn open_remote_file(
    backend: &Backend,
    url: &Url,
) -> Result<Option<Url>, tower_lsp::jsonrpc::Error> {
    match url.scheme() {
        "http" | "https" => {
            let remote_url = Url::parse(&format!("untitled://{}", url.path())).unwrap();
            let content = fetch_remote_content(url).await?;
            open_remote_content(backend, &remote_url, content).await?;
            Ok(Some(remote_url))
        }
        "tombi" => {
            let remote_url = Url::parse(&format!("untitled://{}", url.path())).unwrap();
            let Some(content) = get_tombi_scheme_content(url) else {
                return Ok(None);
            };
            open_remote_content(backend, &remote_url, content).await?;
            Ok(Some(remote_url))
        }
        _ => Ok(None),
    }
}

async fn open_remote_content(
    backend: &Backend,
    remote_url: &Url,
    content: impl Into<String>,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    let remote_url_path = Url::parse(&format!("untitled://{}", remote_url.path())).unwrap();

    create_empty_file(backend, &remote_url_path).await?;
    insert_content(backend, &remote_url_path, content).await?;

    Ok(())
}

async fn create_empty_file(
    backend: &Backend,
    remote_url_path: &Url,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    // First, create the file
    let create_file = CreateFile {
        uri: remote_url_path.clone(),
        options: Some(CreateFileOptions {
            overwrite: Some(true),
            ignore_if_exists: Some(false),
        }),
        annotation_id: None,
    };

    // Create a workspace edit with both changes
    let edit = WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Operations(vec![
            DocumentChangeOperation::Op(ResourceOp::Create(create_file)),
        ])),
        change_annotations: None,
    };

    // Apply the workspace edit
    let _ = backend
        .client
        .send_request::<tower_lsp::lsp_types::request::ApplyWorkspaceEdit>(
            tower_lsp::lsp_types::ApplyWorkspaceEditParams {
                label: Some("Create remote file".to_string()),
                edit,
            },
        )
        .await;

    Ok(())
}

async fn insert_content(
    backend: &Backend,
    remote_url_path: &Url,
    content: impl Into<String>,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    // Then, create the text document edit
    let text_document_edit = TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier {
            uri: remote_url_path.clone(),
            version: Some(0),
        },
        edits: vec![OneOf::Left(TextEdit {
            range: tombi_text::Range::default().into(),
            new_text: content.into(),
        })],
    };

    // Create a workspace edit with both changes
    let edit = WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Edits(vec![text_document_edit])),
        change_annotations: None,
    };

    // Apply the workspace edit
    let _ = backend
        .client
        .send_request::<tower_lsp::lsp_types::request::ApplyWorkspaceEdit>(
            tower_lsp::lsp_types::ApplyWorkspaceEditParams {
                label: Some("Create remote file".to_string()),
                edit,
            },
        )
        .await;

    Ok(())
}

async fn fetch_remote_content(url: &Url) -> Result<String, tower_lsp::jsonrpc::Error> {
    let client = reqwest::Client::new();
    let content = match client.get(url.to_string()).send().await {
        Ok(response) => match response.text().await {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Error fetching content: {}", e);
                return Err(tower_lsp::jsonrpc::Error::new(
                    tower_lsp::jsonrpc::ErrorCode::InternalError,
                ));
            }
        },
        Err(e) => {
            tracing::error!("Error fetching content: {}", e);
            return Err(tower_lsp::jsonrpc::Error::new(
                tower_lsp::jsonrpc::ErrorCode::InternalError,
            ));
        }
    };

    // Check if the content is valid JSON
    tombi_json::ValueNode::from_str(&content.clone()).map_err(|e| {
        tracing::error!("Error parsing {url} content: {}", e);
        tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InternalError)
    })?;

    Ok(content)
}
