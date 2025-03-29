pub mod definitions;

use std::fmt::Write;

use config::{DateTimeDelimiter, IndentStyle, LineEnding, TomlVersion};
use diagnostic::{Diagnostic, SetDiagnostics};
use itertools::{Either, Itertools};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use crate::Format;

pub struct Formatter<'a> {
    toml_version: TomlVersion,
    indent_depth: u8,
    skip_indent: bool,
    definitions: crate::FormatDefinitions,
    #[allow(dead_code)]
    options: &'a crate::FormatOptions,
    source_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
    schema_store: &'a schema_store::SchemaStore,
    buf: String,
}

impl<'a> Formatter<'a> {
    #[inline]
    pub fn new(
        toml_version: TomlVersion,
        definitions: crate::FormatDefinitions,
        options: &'a crate::FormatOptions,
        source_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
        schema_store: &'a schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            indent_depth: 0,
            skip_indent: false,
            definitions,
            options,
            source_url_or_path,
            schema_store,
            buf: String::new(),
        }
    }

    fn parsed_and_ast(&self, source: &str) -> (parser::Parsed<ast::Root>, ast::Root) {
        let parsed = parser::parse(source);

        let Some(parsed) = parsed.cast::<ast::Root>() else {
            unreachable!("TOML Root node is always a valid AST node even if source is empty.")
        };

        let root = parsed.tree();
        tracing::trace!("TOML AST before editing: {:#?}", root);

        (parsed, root)
    }

    /// Format a TOML document and return the result as a string
    ///
    /// This method is async because it may need to fetch the schema from the schema store.
    pub async fn format(mut self, source: &str) -> Result<String, Vec<Diagnostic>> {
        let (parsed, root) = self.parsed_and_ast(source);

        let source_schema = if let Some(schema) = self
            .schema_store
            .try_get_source_schema_from_ast(&root, self.source_url_or_path)
            .await
            .ok()
            .flatten()
        {
            Some(schema)
        } else if let Some(source_url_or_path) = self.source_url_or_path {
            self.schema_store
                .try_get_source_schema(source_url_or_path)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        self.toml_version = source_schema
            .as_ref()
            .and_then(|schema| {
                schema
                    .root_schema
                    .as_ref()
                    .and_then(|root| root.toml_version())
            })
            .unwrap_or(self.toml_version);

        let errors = parsed.errors(self.toml_version).collect_vec();
        if !errors.is_empty() {
            let mut diagnostics = Vec::new();
            for error in errors {
                error.set_diagnostics(&mut diagnostics);
            }
            return Err(diagnostics);
        }

        let root = ast_editor::Editor::new(
            root,
            &schema_store::SchemaContext {
                toml_version: self.toml_version,
                root_schema: source_schema
                    .as_ref()
                    .and_then(|schema| schema.root_schema.as_ref()),
                sub_schema_url_map: source_schema
                    .as_ref()
                    .map(|schema| &schema.sub_schema_url_map),
                store: self.schema_store,
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

    /// Format a TOML document without schema and return the result as a string
    ///
    /// This method is sync because it does not need to fetch the schema from the schema store.
    pub fn format_without_schema(mut self, source: &str) -> Result<String, Vec<Diagnostic>> {
        let (parsed, root) = self.parsed_and_ast(source);

        let errors = parsed.errors(self.toml_version).collect_vec();
        if !errors.is_empty() {
            let mut diagnostics = Vec::new();
            for error in errors {
                error.set_diagnostics(&mut diagnostics);
            }
            return Err(diagnostics);
        }

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

    #[inline]
    pub(crate) fn toml_version(&self) -> TomlVersion {
        self.toml_version
    }

    #[inline]
    pub(crate) fn line_width(&self) -> u8 {
        self.definitions.line_width.unwrap_or_default().value()
    }

    #[inline]
    pub(crate) fn line_ending(&self) -> &'static str {
        match self.definitions.line_ending.unwrap_or_default() {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
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
    pub(crate) fn quote_style(&self) -> config::QuoteStyle {
        self.definitions.quote_style.unwrap_or_default()
    }

    #[inline]
    pub(crate) const fn tailing_comment_space(&self) -> &'static str {
        self.definitions.tailing_comment_space()
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
