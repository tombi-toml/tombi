use itertools::Either;
use reqwest::Client;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::SchemaContext;
use tower_lsp::lsp_types::{
    request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse},
    CreateFile, CreateFileOptions, DocumentChangeOperation, DocumentChanges, Location, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};

use crate::{
    backend::Backend,
    goto_type_definition::{get_type_definition, TypeDefinition},
    handler::hover::get_hover_keys_and_range,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_goto_type_definition(
    backend: &Backend,
    params: GotoTypeDefinitionParams,
) -> Result<Option<GotoTypeDefinitionResponse>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_goto_type_definition");
    tracing::trace!(?params);

    let GotoTypeDefinitionParams {
        text_document_position_params:
            tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document,
                position,
                ..
            },
        ..
    } = params;

    let config = backend.config().await;

    if !config
        .lsp()
        .and_then(|server| server.goto_type_definition.as_ref())
        .and_then(|goto_type_definition| goto_type_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.goto_type_definition.enabled` is false");
        return Ok(None);
    }

    let position = tombi_text::Position::from(position);
    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = backend
        .schema_store
        .build_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;
    let position = position;

    let Some((keys, range)) = get_hover_keys_and_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    if keys.is_empty() && range.is_none() {
        return Ok(None);
    }

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    Ok(
        match get_type_definition(
            &document_tree,
            position,
            &keys,
            &SchemaContext {
                toml_version,
                root_schema: source_schema.as_ref().and_then(|s| s.root_schema.as_ref()),
                sub_schema_url_map: source_schema.as_ref().map(|s| &s.sub_schema_url_map),
                store: &backend.schema_store,
            },
        )
        .await
        {
            Some(TypeDefinition {
                schema_url, range, ..
            }) => {
                let url: Url = schema_url.into();
                if matches!(url.scheme(), "http" | "https") {
                    let remote_url_path = format!("untitled://{}", url.path());
                    let remote_url = Url::parse(&remote_url_path).unwrap();
                    let content = fetch_remote_content(&url).await?;
                    open_remote_file(backend, &remote_url, content).await?;

                    Some(GotoTypeDefinitionResponse::Scalar(Location {
                        uri: remote_url,
                        range: range.into(),
                    }))
                } else {
                    Some(GotoTypeDefinitionResponse::Scalar(Location {
                        uri: url,
                        range: range.into(),
                    }))
                }
            }
            _ => None,
        },
    )
}

async fn fetch_remote_content(url: &Url) -> Result<String, tower_lsp::jsonrpc::Error> {
    let client = Client::new();
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

async fn open_remote_file(
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
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            },
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
