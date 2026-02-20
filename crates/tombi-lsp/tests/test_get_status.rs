use std::{fs, time::Duration};

use tombi_lsp::{
    Backend,
    handler::{handle_did_open, handle_document_symbol, handle_get_status},
};
use tower_lsp::{
    LspService,
    lsp_types::{
        DidOpenTextDocumentParams, DocumentSymbolParams, PartialResultParams,
        TextDocumentIdentifier, TextDocumentItem, Url, WorkDoneProgressParams,
    },
};

#[tokio::test]
async fn get_status_does_not_hang_for_pyproject() -> Result<(), Box<dyn std::error::Error>> {
    tombi_test_lib::init_log();

    let temp_dir = tempfile::tempdir()?;
    let pyproject_path = temp_dir.path().join("pyproject.toml");
    let pyproject_text = r#"
[tool.poe.tasks.pull-base]
help = "Pull the base image for Docker or Podman (whichever is available)"
script = "scripts.container:pull_base_image"
"#;
    fs::write(&pyproject_path, pyproject_text)?;

    let uri = Url::from_file_path(&pyproject_path).map_err(|_| {
        format!(
            "failed to convert path to URI: {}",
            pyproject_path.display()
        )
    })?;

    let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
    let backend = service.inner();

    handle_did_open(
        backend,
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: pyproject_text.to_string(),
            },
        },
    )
    .await;

    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        handle_document_symbol(
            backend,
            DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            },
        ),
    )
    .await
    .map_err(|_| "document_symbol timed out")??;

    let _status = tokio::time::timeout(
        Duration::from_secs(5),
        handle_get_status(backend, TextDocumentIdentifier { uri }),
    )
    .await
    .map_err(|_| "get_status timed out")??;

    Ok(())
}

#[tokio::test]
async fn get_status_does_not_deadlock_with_did_open() -> Result<(), Box<dyn std::error::Error>> {
    tombi_test_lib::init_log();

    let temp_dir = tempfile::tempdir()?;
    let pyproject_path = temp_dir.path().join("pyproject.toml");
    let pyproject_text = r#"
[tool.poe.tasks.pull-base]
help = "Pull the base image for Docker or Podman (whichever is available)"
script = "scripts.container:pull_base_image"
"#;
    fs::write(&pyproject_path, pyproject_text)?;

    let uri = Url::from_file_path(&pyproject_path).map_err(|_| {
        format!(
            "failed to convert path to URI: {}",
            pyproject_path.display()
        )
    })?;

    let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
    let backend = service.inner();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: pyproject_text.to_string(),
        },
    };

    let status_fut = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tokio::time::timeout(
            Duration::from_secs(5),
            handle_get_status(backend, TextDocumentIdentifier { uri }),
        )
        .await
    };

    let (_open_done, status_result) =
        tokio::join!(handle_did_open(backend, open_params), status_fut);

    let _ = status_result.map_err(|_| "get_status timed out while did_open was running")??;
    Ok(())
}
