use tombi_diagnostic::Level;
use tombi_linter::test_lint;
use tombi_test_lib::pyproject_schema_path;

test_lint! {
    #[test]
    fn readme_relative_path(
        r#"[project]
name = "example"
version = "1.0"
readme = "docs/README.md"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn readme_absolute_path(
        r#"[project]
name = "example"
version = "1.0"
readme = "/README.md"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn readme_file_with_content_type(
        r#"[project]
name = "example"
version = "1.0"
readme = { file = "README.txt", content-type = "text/plain" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn readme_absolute_file(
        r#"[project]
name = "example"
version = "1.0"
readme = { file = "/README.md", content-type = "text/markdown" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn readme_file_missing_content_type(
        r#"[project]
name = "example"
version = "1.0"
readme = { file = "README.md" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "table-key-required", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn readme_text_with_content_type(
        r#"[project]
name = "example"
version = "1.0"
readme = { text = "Example", content-type = "text/plain" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn readme_text_missing_content_type(
        r#"[project]
name = "example"
version = "1.0"
readme = { text = "Example" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "table-key-required", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn readme_content_type_only(
        r#"[project]
name = "example"
version = "1.0"
readme = { content-type = "text/plain" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn readme_posix_relative_drive_filename(
        r#"[project]
name = "example"
version = "1.0"
readme = 'C:\README.md'
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn authors_and_maintainers_names(
        r#"[project]
name = "example"
version = "1.0"
authors = [{name = "Example Person"}]
maintainers = [{name = "Example Organization"}]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn author_name_with_comma(
        r#"[project]
name = "example"
version = "1.0"
authors = [{name = "Person, Example"}]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn maintainer_name_with_comma(
        r#"[project]
name = "example"
version = "1.0"
maintainers = [{name = "Person, Example"}]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn urls_label_32(
        r#"[project]
name = "example"
version = "1.0"
[project.urls]
"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" = "https://example.com"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn urls_label_33(
        r#"[project]
name = "example"
version = "1.0"
[project.urls]
"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" = "https://example.com"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-max-length", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn urls_supplementary_unicode_32(
        r#"[project]
name = "example"
version = "1.0"
[project.urls]
"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀" = "https://example.com"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn urls_combining_unicode_33(
        r#"[project]
name = "example"
version = "1.0"
[project.urls]
"ááááááááááááááááa" = "https://example.com"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-max-length", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scripts_non_recommended_name(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
"!tool with space" = "example:main [legacy-extra]"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn scripts_name_equals(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
"tool=bad" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scripts_name_bracket(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
"[tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scripts_name_leading_space(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
" tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scripts_name_trailing_space(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
"tool " = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn gui_scripts_non_recommended_name(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
"!tool with space" = "example:main [legacy-extra]"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn gui_scripts_name_equals(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
"tool=bad" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn gui_scripts_name_bracket(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
"[tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn gui_scripts_name_leading_space(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
" tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn gui_scripts_name_trailing_space(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
"tool " = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_non_recommended_name(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
"!tool with space" = "example:main [legacy-extra]"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_name_equals(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
"tool=bad" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_name_bracket(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
"[tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_name_leading_space(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
" tool" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_name_trailing_space(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
"tool " = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_namespaces_nonempty(
        r#"[project]
name = "example"
version = "1.0"
import-namespaces = ["example.namespace"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_namespaces_empty(
        r#"[project]
name = "example"
version = "1.0"
import-namespaces = []
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "array-min-values", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_names_empty(
        r#"[project]
name = "example"
version = "1.0"
import-names = []
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_names_missing_parents(
        r#"[project]
name = "example"
version = "1.0"
import-names = ["example.module"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn scripts_empty_name_has_only_existing_warning(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
"" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "key-empty", level: Level::WARNING }])
}

test_lint! {
    #[test]
    fn gui_scripts_empty_name_has_only_existing_warning(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
"" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "key-empty", level: Level::WARNING }])
}

test_lint! {
    #[test]
    fn entry_points_example_plugins_empty_name_has_only_existing_warning(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points."example.plugins"]
"" = "example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "key-empty", level: Level::WARNING }])
}

test_lint! {
    #[test]
    fn readme_relative_parent_path(
        r#"[project]
name = "example"
version = "1.0"
readme = "../README.md"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn readme_relative_parent_file(
        r#"[project]
name = "example"
version = "1.0"
readme = { file = "../README.md", content-type = "text/markdown" }
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn omitted_project_table(
        r#"[tool.example]
enabled = true
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_authors(
        r#"[project]
name = "example"
version = "1.0"
authors = [{name="Author"}]
dynamic = ["authors"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_classifiers(
        r#"[project]
name = "example"
version = "1.0"
classifiers = ["Topic :: Utilities"]
dynamic = ["classifiers"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_dependencies(
        r#"[project]
name = "example"
version = "1.0"
dependencies = ["requests"]
dynamic = ["dependencies"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_entry_points(
        r#"[project]
name = "example"
version = "1.0"
entry-points = {pytest11={example="example"}}
dynamic = ["entry-points"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_gui_scripts(
        r#"[project]
name = "example"
version = "1.0"
gui-scripts = {example="example:main"}
dynamic = ["gui-scripts"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_import_names(
        r#"[project]
name = "example"
version = "1.0"
import-names = ["example"]
dynamic = ["import-names"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_import_namespaces(
        r#"[project]
name = "example"
version = "1.0"
import-namespaces = ["namespace"]
dynamic = ["import-namespaces"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_keywords(
        r#"[project]
name = "example"
version = "1.0"
keywords = ["toml"]
dynamic = ["keywords"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_license_files(
        r#"[project]
name = "example"
version = "1.0"
license-files = ["LICENSE"]
dynamic = ["license-files"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_maintainers(
        r#"[project]
name = "example"
version = "1.0"
maintainers = [{name="Maintainer"}]
dynamic = ["maintainers"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_optional_dependencies(
        r#"[project]
name = "example"
version = "1.0"
optional-dependencies = {dev=["pytest"]}
dynamic = ["optional-dependencies"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_scripts(
        r#"[project]
name = "example"
version = "1.0"
scripts = {example="example:main"}
dynamic = ["scripts"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn partially_dynamic_urls(
        r#"[project]
name = "example"
version = "1.0"
urls = {Homepage="https://example.com"}
dynamic = ["urls"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn scalar_static_and_dynamic_description(
        r#"[project]
name = "example"
version = "1.0"
description = "Example"
dynamic = ["description"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scalar_static_and_dynamic_readme(
        r#"[project]
name = "example"
version = "1.0"
readme = "README.md"
dynamic = ["readme"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scalar_static_and_dynamic_requires_python(
        r#"[project]
name = "example"
version = "1.0"
requires-python = ">=3.8"
dynamic = ["requires-python"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn scalar_static_and_dynamic_license(
        r#"[project]
name = "example"
version = "1.0"
license = "MIT"
dynamic = ["license"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn version_static_and_dynamic(
        r#"[project]
name = "example"
version = "1.0"
dynamic = ["version"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }, { code: "one-of-multiple-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn version_dynamic_only(
        r#"[project]
name = "example"
dynamic = ["version"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn name_unicode_invalid(
        r#"[project]
name = "aéz"
version = "1.0"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }, { code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn name_unicode_digit_invalid(
        r#"[project]
name = "١"
version = "1.0"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }, { code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn name_ascii_punctuation_valid(
        r#"[project]
name = "A.b_c-1"
version = "1.0"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn version_upper_case(
        r#"[project]
name = "example"
version = "V1.0RC1.POST2.DEV3"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn version_normalization_whitespace(
        r#"[project]
name = "example"
version = " \t1.0\n\r\f\u000b"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn version_invalid(
        r#"[project]
name = "example"
version = "1.invalid"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn include_group_required(
        r#"[dependency-groups]
dev = [{}]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "table-key-required", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn include_group_extra_key(
        r#"[dependency-groups]
dev = [{include-group="test", other=true}]
test=[]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "key-not-allowed", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn include_group_valid(
        r#"[dependency-groups]
dev = [{include-group="test"}]
test=[]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dependency_group_repeated_requirements(
        r#"[dependency-groups]
dev = ["pytest", "pytest"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dependency_group_repeated_includes(
        r#"[dependency-groups]
dev = [{include-group="test"},{include-group="test"}]
test=[]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn build_requirements_repeated(
        r#"[build-system]
requires=["setuptools","setuptools"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dependencies_repeated(
        r#"[project]
name = "example"
version = "1.0"
dependencies = ["pytest","pytest"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn keywords_repeated(
        r#"[project]
name = "example"
version = "1.0"
keywords = ["toml","toml"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn classifiers_repeated(
        r#"[project]
name = "example"
version = "1.0"
classifiers = ["Topic :: Utilities","Topic :: Utilities"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn optional_dependencies_invalid_name(
        r#"[project]
name = "example"
version = "1.0"
[project.optional-dependencies]
"bad!" = ["pytest"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "key-pattern", level: Level::ERROR }, { code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn optional_dependencies_invalid_shape(
        r#"[project]
name = "example"
version = "1.0"
[project.optional-dependencies]
dev = 42
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "type-mismatch", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn optional_dependencies_nonnormalized_extra(
        r#"[project]
name = "example"
version = "1.0"
[project.optional-dependencies]
Dev_Test = ["pytest"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_name_empty_string(
        r#"[project]
name = "example"
version = "1.0"
import-names=[""]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_namespace_empty_string(
        r#"[project]
name = "example"
version = "1.0"
import-namespaces=[""]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_name_unicode(
        r#"[project]
name = "example"
version = "1.0"
import-names=["café.module", "𐐀 ; private"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_namespace_unicode(
        r#"[project]
name = "example"
version = "1.0"
import-namespaces=["日本語.namespace"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn import_name_digit_start(
        r#"[project]
name = "example"
version = "1.0"
import-names=["1abc"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }, { code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_name_empty_component(
        r#"[project]
name = "example"
version = "1.0"
import-names=["pkg..module"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }, { code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_name_ascii_punctuation(
        r#"[project]
name = "example"
version = "1.0"
import-names=["pkg-name"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }, { code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_relative(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn license_glob_dots_in_filename(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE..txt"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn license_glob_ranges(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICEN[CS]E*"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn license_glob_unicode(
        r#"[project]
name = "example"
version = "1.0"
license-files=["licenses/日本語*"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn license_glob_absolute(
        r#"[project]
name = "example"
version = "1.0"
license-files=["/LICENSE"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_parent(
        r#"[project]
name = "example"
version = "1.0"
license-files=["../LICENSE"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_nested_parent(
        r#"[project]
name = "example"
version = "1.0"
license-files=["licenses/../LICENSE"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_backslash(
        r#"[project]
name = "example"
version = "1.0"
license-files=["licenses\\LICENSE"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }, { code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_brace(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICEN{CS}E"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_unclosed_range(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE[CS"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_empty_range(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE[]"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_negated_range(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE[!a]"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_slash_in_range(
        r#"[project]
name = "example"
version = "1.0"
license-files=["LICENSE[a/b]"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn legacy_license_absolute(
        r#"[project]
name = "example"
version = "1.0"
license={file="/LICENSE"}
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn legacy_license_relative_parent(
        r#"[project]
name = "example"
version = "1.0"
license={file="../LICENSE"}
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "deprecated", level: Level::WARNING }])
}

test_lint! {
    #[test]
    fn backend_path_absolute(
        r#"[build-system]
requires=[]
backend-path=["/backend"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "not-schema-match", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn backend_path_relative_parent(
        r#"[build-system]
requires=[]
backend-path=["backend/../actual"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_module_only(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="example.module"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_unicode(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="日本語.module:𐐀"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_extras(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="example:main [Dev_Test, other]"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_empty_extras(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="example:main []"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_spaces(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example=" example.module : main.attr [ dev ] "
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn entry_reference_double_colon(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="example::main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_reference_missing_module(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example=":main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_reference_missing_object(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="example:"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn entry_reference_digit_start(
        r#"[project]
name = "example"
version = "1.0"
[project.scripts]
example="1example:main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn gui_reference_invalid(
        r#"[project]
name = "example"
version = "1.0"
[project.gui-scripts]
example="pkg::main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn other_entry_reference_invalid(
        r#"[project]
name = "example"
version = "1.0"
[project.entry-points.pytest11]
example="pkg::main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn build_backend_module(
        r#"[build-system]
requires=[]
build-backend="example.module"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn build_backend_unicode(
        r#"[build-system]
requires=[]
build-backend="日本語:𐐀"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn build_backend_invalid(
        r#"[build-system]
requires=[]
build-backend="pkg::main"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn build_backend_extras_invalid(
        r#"[build-system]
requires=[]
build-backend="pkg:main [extra]"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn import_name_empty_private(
        r#"[project]
name = "example"
version = "1.0"
import-names = ["; private"]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn version_unicode_whitespace(
        r#"[project]
name = "example"
version = "\u00a01.0\u0085"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn license_glob_empty_string(
        r#"[project]
name = "example"
version = "1.0"
license-files = [""]
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-min-length", level: Level::ERROR }])
}

test_lint! {
    #[test]
    fn license_glob_empty_array(
        r#"[project]
name = "example"
version = "1.0"
license-files = []
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn version_bom_is_not_whitespace(
        r#"[project]
name = "example"
version = "\uFEFF1.0\uFEFF"
"#,
        SchemaPath(pyproject_schema_path()),
    ) -> Diagnostics([{ code: "string-pattern", level: Level::ERROR }])
}
