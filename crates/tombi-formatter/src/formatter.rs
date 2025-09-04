pub mod definitions;

use std::fmt::Write;

use itertools::Either;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
use tombi_config::{DateTimeDelimiter, IndentStyle, TomlVersion};
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use unicode_segmentation::UnicodeSegmentation;

use crate::Format;

pub struct Formatter<'a> {
    toml_version: TomlVersion,
    indent_depth: u8,
    skip_indent: bool,
    single_line_mode: bool,
    definitions: &'a crate::FormatDefinitions,
    #[allow(dead_code)]
    options: &'a crate::FormatOptions,
    source_uri_or_path: Option<Either<&'a tombi_uri::Uri, &'a std::path::Path>>,
    schema_store: &'a tombi_schema_store::SchemaStore,
    buf: String,
}

impl<'a> Formatter<'a> {
    #[inline]
    pub fn new(
        toml_version: TomlVersion,
        definitions: &'a crate::FormatDefinitions,
        options: &'a crate::FormatOptions,
        source_uri_or_path: Option<Either<&'a tombi_uri::Uri, &'a std::path::Path>>,
        schema_store: &'a tombi_schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            indent_depth: 0,
            skip_indent: false,
            single_line_mode: false,
            definitions,
            options,
            source_uri_or_path,
            schema_store,
            buf: String::new(),
        }
    }

    /// Format a TOML document and return the result as a string
    pub async fn format(mut self, source: &str) -> Result<String, Vec<Diagnostic>> {
        let (source_schema, tombi_document_comment_directive) = if let Some(parsed) =
            tombi_parser::parse_document_header_comments(source).cast::<tombi_ast::Root>()
        {
            let root = parsed.tree();
            (
                self.schema_store
                    .resolve_source_schema_from_ast(&root, self.source_uri_or_path)
                    .await
                    .ok()
                    .flatten(),
                tombi_validator::comment_directive::get_tombi_document_comment_directive(&root)
                    .await,
            )
        } else {
            (None, None)
        };

        if let Some(tombi_document_comment_directive) = &tombi_document_comment_directive {
            if let Some(format) = &tombi_document_comment_directive.format {
                if format.disabled() == Some(true) {
                    match self.source_uri_or_path.map(|path| match path {
                        Either::Left(url) => url.to_string(),
                        Either::Right(path) => path.to_string_lossy().to_string(),
                    }) {
                        Some(source_url_or_path) => {
                            tracing::info!(
                                "Skip formatting for \"{source_url_or_path}\" due to `format.disable`"
                            );
                        }
                        None => {
                            tracing::info!("Skip formatting for stdin due to `format.disable`");
                        }
                    }
                    return Ok(source.to_string());
                }
            }
        }

        self.toml_version = tombi_document_comment_directive
            .as_ref()
            .and_then(|directive| directive.toml_version)
            .unwrap_or_else(|| {
                source_schema
                    .as_ref()
                    .and_then(|schema| {
                        schema
                            .root_schema
                            .as_ref()
                            .and_then(|root| root.toml_version())
                    })
                    .unwrap_or(self.toml_version)
            });

        let root = tombi_parser::parse(source, self.toml_version)
            .try_into_root()
            .map_err(|errors| {
                let mut diagnostics = vec![];
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }

                diagnostics
            })?;

        let source_path = self.source_uri_or_path.and_then(|path| match path {
            Either::Left(url) => url.to_file_path().ok(),
            Either::Right(path) => Some(path.to_path_buf()),
        });

        let root = tombi_ast_editor::Editor::new(
            root,
            source_path.as_deref(),
            &tombi_schema_store::SchemaContext {
                toml_version: self.toml_version,
                root_schema: source_schema
                    .as_ref()
                    .and_then(|schema| schema.root_schema.as_ref()),
                sub_schema_uri_map: source_schema
                    .as_ref()
                    .map(|schema| &schema.sub_schema_uri_map),
                store: self.schema_store,
                strict: tombi_document_comment_directive
                    .as_ref()
                    .and_then(|directive| {
                        directive.schema.as_ref().and_then(|schema| schema.strict)
                    }),
            },
        )
        .edit()
        .await;

        tracing::trace!("TOML AST after editing: {:#?}", root);

        let line_ending = {
            root.format(&mut self).unwrap();
            self.line_ending()
        };

        Ok(self.buf + line_ending)
    }

    /// Format a node and return the result as a string
    pub(crate) fn format_to_string<T: Format>(
        &mut self,
        node: &T,
    ) -> Result<String, std::fmt::Error> {
        let old_buf = std::mem::take(&mut self.buf);
        let old_indent = self.indent_depth;
        let old_skip = self.skip_indent;

        node.format(self)?;
        let result = std::mem::take(&mut self.buf);

        self.buf = old_buf;
        self.indent_depth = old_indent;
        self.skip_indent = old_skip;

        Ok(result)
    }

    pub(crate) fn format_tombi_comment_directive_content(
        &mut self,
        content: &str,
    ) -> Result<String, std::fmt::Error> {
        let Ok(root) =
            tombi_parser::parse(content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION).try_into_root()
        else {
            return Ok(content.trim().to_string());
        };
        self.single_line_mode = true;
        let formatted = self.format_to_string(&root)?;
        self.single_line_mode = false;
        Ok(formatted)
    }

    #[inline]
    pub(crate) fn toml_version(&self) -> TomlVersion {
        self.toml_version
    }

    #[inline]
    pub(crate) fn line_width(&self) -> u8 {
        self.definitions.line_width.unwrap_or_default().value()
    }

    #[inline]
    pub fn line_ending(&self) -> &'static str {
        self.definitions.line_ending.unwrap_or_default().into()
    }

    #[inline]
    pub(crate) fn single_line_mode(&self) -> bool {
        self.single_line_mode
    }

    #[inline]
    pub(crate) fn date_time_delimiter(&self) -> Option<&'static str> {
        match self.definitions.date_time_delimiter.unwrap_or_default() {
            DateTimeDelimiter::T => Some("T"),
            DateTimeDelimiter::Space => Some(" "),
            DateTimeDelimiter::Preserve => None,
        }
    }

    #[inline]
    pub(crate) fn quote_style(&self) -> tombi_config::QuoteStyle {
        self.definitions.quote_style.unwrap_or_default()
    }

    #[inline]
    pub(crate) const fn trailing_comment_space(&self) -> &'static str {
        self.definitions.trailing_comment_space()
    }

    #[inline]
    pub(crate) const fn singleline_array_bracket_inner_space(&self) -> &'static str {
        self.definitions.singleline_array_bracket_inner_space()
    }

    #[inline]
    pub(crate) const fn singleline_array_space_after_comma(&self) -> &'static str {
        self.definitions.singleline_array_space_after_comma()
    }

    #[inline]
    pub(crate) const fn singleline_inline_table_brace_inner_space(&self) -> &'static str {
        self.definitions.singleline_inline_table_brace_inner_space()
    }

    #[inline]
    pub(crate) const fn singleline_inline_table_space_after_comma(&self) -> &'static str {
        self.definitions.singleline_inline_table_space_after_comma()
    }

    #[inline]
    pub(crate) fn ident(&self, depth: u8) -> String {
        match self.definitions.indent_style.unwrap_or_default() {
            IndentStyle::Space => " ".repeat(
                (self.definitions.indent_width.unwrap_or_default().value() * depth) as usize,
            ),
            IndentStyle::Tab => "\t".repeat(depth as usize),
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.reset_indent();
    }

    #[inline]
    pub(crate) fn write_indent(&mut self) -> Result<(), std::fmt::Error> {
        if self.skip_indent {
            self.skip_indent = false;

            Ok(())
        } else {
            write!(self, "{}", self.ident(self.indent_depth))
        }
    }

    #[inline]
    pub(crate) fn inc_indent(&mut self) {
        self.indent_depth += 1;
    }

    #[inline]
    pub(crate) fn dec_indent(&mut self) {
        self.indent_depth = self.indent_depth.saturating_sub(1);
    }

    #[inline]
    pub(crate) fn skip_indent(&mut self) {
        self.skip_indent = true;
    }

    #[inline]
    pub(crate) fn reset_indent(&mut self) {
        self.indent_depth = 0;
    }

    #[inline]
    pub(crate) fn current_line_width(&self) -> usize {
        self.buf
            .split("\n")
            .last()
            .unwrap_or_default()
            .graphemes(true)
            .count()
    }
}

impl std::fmt::Write for Formatter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s)
    }
}
