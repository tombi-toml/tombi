#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum SchemaTooltip {
    Content(SchemaTooltipContent),
    Markdown(String),
    Composite(Box<CompositeSchemaTooltip>),
}

impl SchemaTooltip {
    pub(crate) fn composite(contents: Vec<SchemaTooltip>) -> Option<Self> {
        let mut flattened = Vec::with_capacity(contents.len());
        for content in contents {
            match content {
                Self::Composite(composite) => flattened.extend(composite.contents),
                content => flattened.push(content),
            }
        }

        let mut unique_contents = Vec::with_capacity(flattened.len());
        for content in flattened {
            if !unique_contents.contains(&content) {
                unique_contents.push(content);
            }
        }

        match unique_contents.len() {
            0 => None,
            1 => unique_contents.pop(),
            _ => Some(Self::Composite(Box::new(CompositeSchemaTooltip {
                contents: unique_contents,
            }))),
        }
    }

    pub(crate) fn render(&self, keys: Option<&str>) -> String {
        let mut output = String::new();
        self.write_markdown(&mut output, keys)
            .expect("writing Markdown to a String cannot fail");
        output
    }

    pub(crate) fn with_common_content(self, common: SchemaTooltipContent) -> Self {
        Self::Content(common).combine(self)
    }

    pub(crate) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Content(common), Self::Content(specific)) => {
                Self::Content(common.merge(specific))
            }
            (Self::Composite(composite), other) => {
                Self::Composite(Box::new(CompositeSchemaTooltip {
                    contents: composite
                        .contents
                        .into_iter()
                        .map(|content| content.combine(other.clone()))
                        .collect(),
                }))
            }
            (content, Self::Composite(composite)) => {
                Self::Composite(Box::new(CompositeSchemaTooltip {
                    contents: composite
                        .contents
                        .into_iter()
                        .map(|branch| content.clone().combine(branch))
                        .collect(),
                }))
            }
            (Self::Markdown(mut left), Self::Markdown(right)) => {
                if !left.is_empty() && !left.ends_with("\n\n") {
                    left.push_str("\n\n");
                }
                left.push_str(&right);
                Self::Markdown(left)
            }
            (Self::Content(content), Self::Markdown(markdown)) => {
                Self::Content(content).with_markdown(markdown)
            }
            (Self::Markdown(markdown), Self::Content(content)) => {
                Self::Content(content).with_markdown(markdown)
            }
        }
    }

    fn with_markdown(self, markdown: String) -> Self {
        match self {
            Self::Content(content) => {
                let mut common_markdown = String::new();
                content
                    .write_markdown(&mut common_markdown, None)
                    .expect("writing Markdown to a String cannot fail");
                common_markdown.push_str(&markdown);
                Self::Markdown(common_markdown)
            }
            content => content,
        }
    }

    fn write_markdown(&self, output: &mut String, keys: Option<&str>) -> std::fmt::Result {
        use std::fmt::Write;

        match self {
            Self::Content(content) => content.write_markdown(output, keys),
            Self::Markdown(markdown) => {
                if let Some(keys) = keys {
                    writeln!(output, "Keys: `{keys}`\n")?;
                }
                output.push_str(markdown);
                if !markdown.ends_with('\n') {
                    output.push('\n');
                }
                Ok(())
            }
            Self::Composite(composite) => {
                for (index, content) in composite.contents.iter().enumerate() {
                    if index > 0 {
                        writeln!(output, "---\n")?;
                    }
                    content.write_markdown(output, keys)?;
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for SchemaTooltip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(None))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CompositeSchemaTooltip {
    contents: Vec<SchemaTooltip>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SchemaTooltipContent {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) value_type: String,
    pub(crate) constraints: Option<String>,
    pub(crate) schema: Option<String>,
}

impl SchemaTooltipContent {
    fn merge(self, specific: Self) -> Self {
        Self {
            title: specific.title.or(self.title),
            description: merge_text(specific.description, self.description),
            value_type: specific.value_type,
            constraints: merge_text(self.constraints, specific.constraints),
            schema: merge_text(self.schema, specific.schema),
        }
    }

    fn write_markdown(&self, output: &mut String, keys: Option<&str>) -> std::fmt::Result {
        use std::fmt::Write;

        if let Some(title) = &self.title {
            writeln!(output, "#### {title}\n")?;
        }
        if let Some(description) = &self.description {
            writeln!(output, "{description}\n")?;
        }
        if let Some(keys) = keys {
            writeln!(output, "Keys: `{keys}`\n")?;
        }
        writeln!(output, "Value: `{}`\n", self.value_type)?;
        if let Some(constraints) = &self.constraints {
            output.push_str(constraints);
            if !constraints.ends_with('\n') {
                output.push('\n');
            }
        }
        if let Some(schema) = &self.schema {
            writeln!(output, "{schema}\n")?;
        }
        Ok(())
    }
}

fn merge_text(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    match (primary, secondary) {
        (Some(primary), Some(secondary)) if primary != secondary => Some(format!(
            "{}\n\n{}",
            primary.trim_end(),
            secondary.trim_start()
        )),
        (Some(primary), _) => Some(primary),
        (None, secondary) => secondary,
    }
}
