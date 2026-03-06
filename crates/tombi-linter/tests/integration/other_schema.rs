use tombi_linter::test_lint;

test_lint! {
    // Ref: https://github.com/tombi-toml/tombi/issues/517
    #[test]
    fn test_mise_toml(
        r#"
        #:schema https://mise.jdx.dev/schema/mise.json

        [env]
        PROJECT_SLUG = '{{ config_root | basename | slugify }}'

        _.python.venv.path = '{% if env.UV_PROJECT_ENVIRONMENT %}{{ env.UV_PROJECT_ENVIRONMENT }}{% else %}.venv{% endif %}'
        _.python.venv.create = true

        # Flask/Poster dev ONLY settings
        FLASK_DEBUG=1
        "#,
    ) -> Ok(_)
}
