pub mod definitions;

use std::fmt::Write;

use itertools::Either;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
use tombi_config::{IndentStyle, TomlVersion};
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    types::{AlignmentWidth, WithAlignmentHint},
    Format,
};

pub struct Formatter<'a> {
    toml_version: TomlVersion,
    indent_depth: u8,
    skip_indent: bool,
    skip_comment: bool,
    single_line_mode: bool,
    definitions: crate::FormatDefinitions,
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
        options: &'a crate::FormatOptions,
        source_uri_or_path: Option<Either<&'a tombi_uri::Uri, &'a std::path::Path>>,
        schema_store: &'a tombi_schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            indent_depth: 0,
            skip_indent: false,
            skip_comment: false,
            single_line_mode: false,
            definitions: crate::FormatDefinitions::new(options),
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

        Ok(if self.buf.is_empty() {
            self.buf
        } else {
            self.buf + line_ending
        })
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

    pub(crate) fn format_to_string_without_comment<T: Format>(
        &mut self,
        node: &T,
    ) -> Result<String, std::fmt::Error> {
        self.skip_comment = true;
        let result = self.format_to_string(node)?;
        self.skip_comment = false;
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
    pub(crate) const fn toml_version(&self) -> TomlVersion {
        self.toml_version
    }

    #[inline]
    pub(crate) fn skip_comment(&self) -> bool {
        self.skip_comment
    }

    #[inline]
    pub(crate) fn single_line_mode(&self) -> bool {
        self.single_line_mode
    }

    #[inline]
    pub(crate) const fn line_width(&self) -> u8 {
        self.definitions.line_width
    }

    #[inline]
    pub const fn line_ending(&self) -> &'static str {
        self.definitions.line_ending
    }

    #[inline]
    pub(crate) const fn indent_sub_tables(&self) -> bool {
        self.definitions.indent_sub_tables
    }

    #[inline]
    pub(crate) const fn indent_table_key_values(&self) -> bool {
        self.definitions.indent_table_key_values
    }

    #[inline]
    pub(crate) fn key_value_equal_space(&self) -> &'static str {
        // SAFETY: The lifetime of `key_value_equal_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe {
            std::mem::transmute::<&str, &'static str>(&self.definitions.key_value_equal_space)
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn trailing_comment_alignment_width<'b, T: Format + Sized + 'b>(
        &mut self,
        values: impl Iterator<Item = &'b T>,
        equal_alignment_width: Option<AlignmentWidth>,
    ) -> Result<Option<AlignmentWidth>, std::fmt::Error>
    where
        WithAlignmentHint<'b, T>: Format,
    {
        if self.definitions.trailing_comment_alignment {
            let mut widths = vec![];
            for value in values {
                let formatted = self.format_to_string_without_comment(
                    &WithAlignmentHint::new_with_equal_alignment_width(
                        value,
                        equal_alignment_width,
                    ),
                )?;
                widths.push(AlignmentWidth::new(&formatted));
            }
            Ok(widths.into_iter().max())
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub(crate) fn trailing_comment_space(&self) -> &'static str {
        // SAFETY: The lifetime of `trailing_comment_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe {
            std::mem::transmute::<&str, &'static str>(&self.definitions.trailing_comment_space)
        }
    }

    #[inline]
    pub(crate) fn string_quote_style(&self) -> tombi_config::StringQuoteStyle {
        self.definitions.string_quote_style
    }

    #[inline]
    pub(crate) fn date_time_delimiter(&self) -> Option<&str> {
        self.definitions.date_time_delimiter
    }

    #[inline]
    pub(crate) fn array_bracket_space(&self) -> &'static str {
        // SAFETY: The lifetime of `array_bracket_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe { std::mem::transmute::<&str, &'static str>(&self.definitions.array_bracket_space) }
    }

    #[inline]
    pub(crate) fn array_comma_space(&self) -> &'static str {
        // SAFETY: The lifetime of `array_comma_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe { std::mem::transmute::<&str, &'static str>(&self.definitions.array_comma_space) }
    }

    #[inline]
    pub(crate) fn inline_table_brace_space(&self) -> &'static str {
        // SAFETY: The lifetime of `inline_table_brace_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe {
            std::mem::transmute::<&str, &'static str>(&self.definitions.inline_table_brace_space)
        }
    }

    #[inline]
    pub(crate) fn inline_table_comma_space(&self) -> &'static str {
        // SAFETY: The lifetime of `inline_table_comma_space` is `'static`.
        //         It is guaranteed by the `FormatDefinitions` struct.
        unsafe {
            std::mem::transmute::<&str, &'static str>(&self.definitions.inline_table_comma_space)
        }
    }

    #[inline]
    pub(crate) fn key_value_equal_alignment_width(
        &self,
        key_values: impl Iterator<Item = &'a tombi_ast::KeyValue>,
    ) -> Option<AlignmentWidth> {
        if self.definitions.key_value_equal_alignment {
            key_values
                .filter_map(|key_value| key_value.keys())
                .map(|keys| AlignmentWidth::new(&keys.to_string()))
                .max()
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn ident(&self, depth: u8) -> String {
        match self.definitions.indent_style {
            IndentStyle::Space => " ".repeat((self.definitions.indent_width * depth) as usize),
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
