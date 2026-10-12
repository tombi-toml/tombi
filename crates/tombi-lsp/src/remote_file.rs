use std::str::FromStr;

use tombi_schema_store::get_tombi_schemastore_content;
use tower_lsp_server::gen_lsp_types::{
    ApplyWorkspaceEditParams, ApplyWorkspaceEditRequest, CreateFile, CreateFileOptions,
    DocumentChange, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit,
    TextDocumentIdentifier, TextEdit, Uri, WorkspaceEdit,
};

use crate::Backend;

pub async fn open_remote_file(
    backend: &Backend,
    uri: &tombi_uri::Uri,
) -> Result<Option<tombi_uri::Uri>, tower_lsp_server::jsonrpc::Error> {
    match uri.scheme() {
        "http" | "https" => {
            // Check if cache file exists
            if let Some(cache_path) = tombi_cache::get_cache_file_path(uri).await
                && cache_path.is_file()
                && let Ok(cached_uri) = tombi_uri::Uri::from_file_path(&cache_path)
            {
                return Ok(Some(cached_uri));
            }
            let remote_uri = build_remote_document_uri(uri)?;
            let content = fetch_remote_content(uri).await?;
            open_remote_content(backend, &remote_uri, content).await?;
            Ok(Some(remote_uri))
        }
        "tombi" => {
            let remote_uri = build_remote_document_uri(uri)?;
            let Some(content) = get_tombi_schemastore_content(uri) else {
                return Ok(None);
            };
            open_remote_content(backend, &remote_uri, content).await?;
            Ok(Some(remote_uri))
        }
        _ => Ok(None),
    }
}

async fn open_remote_content(
    backend: &Backend,
    remote_url: &Uri,
    content: impl Into<String>,
) -> Result<(), tower_lsp_server::jsonrpc::Error> {
    create_empty_file(backend, remote_url).await?;
    insert_content(backend, remote_url, content).await?;

    Ok(())
}

#[inline]
fn build_remote_document_uri(
    uri: &tombi_uri::Uri,
) -> Result<tombi_uri::Uri, tower_lsp_server::jsonrpc::Error> {
    tombi_uri::Uri::from_str(&format!("untitled://{}", uri.path())).map_err(|err| {
        tower_lsp_server::jsonrpc::Error::invalid_params(format!(
            "failed to build untitled URL for remote content: {err}"
        ))
    })
}

async fn create_empty_file(
    backend: &Backend,
    remote_url_path: &Uri,
) -> Result<(), tower_lsp_server::jsonrpc::Error> {
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
        document_changes: Some(vec![DocumentChange::CreateFile(create_file)]),
        change_annotations: None,
    };

    // Apply the workspace edit
    let _ = backend
        .client
        .send_request::<ApplyWorkspaceEditRequest>(ApplyWorkspaceEditParams {
            label: Some("Create remote file".to_string()),
            edit,
            metadata: None,
        })
        .await;

    Ok(())
}

async fn insert_content(
    backend: &Backend,
    remote_url_path: &Uri,
    content: impl Into<String>,
) -> Result<(), tower_lsp_server::jsonrpc::Error> {
    // Then, create the text document edit
    let text_document_edit = TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier {
            text_document_identifier: TextDocumentIdentifier {
                uri: remote_url_path.clone(),
            },
            version: Some(0),
        },
        edits: vec![
            TextEdit {
                range: Default::default(),
                new_text: content.into(),
            }
            .into(),
        ],
    };

    // Create a workspace edit with both changes
    let edit = WorkspaceEdit {
        changes: None,
        document_changes: Some(vec![DocumentChange::TextDocumentEdit(text_document_edit)]),
        change_annotations: None,
    };

    // Apply the workspace edit
    let _ = backend
        .client
        .send_request::<ApplyWorkspaceEditRequest>(ApplyWorkspaceEditParams {
            label: Some("Create remote file".to_string()),
            edit,
            metadata: None,
        })
        .await;

    Ok(())
}

async fn fetch_remote_content(url: &Uri) -> Result<String, tower_lsp_server::jsonrpc::Error> {
    use tombi_schema_store::HttpClient;

    let client = tombi_schema_store::DefaultHttpClient::new();
    let bytes = client.get_bytes(url.as_str()).await.map_err(|error| {
        log::error!("failed to fetch content: {error}");
        tower_lsp_server::jsonrpc::Error::new(tower_lsp_server::jsonrpc::ErrorCode::InternalError)
    })?;
    let content = String::from_utf8(bytes.to_vec()).map_err(|error| {
        log::error!("remote content is not valid UTF-8: {error}");
        tower_lsp_server::jsonrpc::Error::new(tower_lsp_server::jsonrpc::ErrorCode::InternalError)
    })?;

    // Check if the content is valid JSON
    tombi_json::ValueNode::from_str(&content).map_err(|e| {
        log::error!("failed to parse {url} content: {}", e);
        tower_lsp_server::jsonrpc::Error::new(tower_lsp_server::jsonrpc::ErrorCode::InternalError)
    })?;

    Ok(content)
}
