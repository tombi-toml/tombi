#[macro_export]
macro_rules! matches_accessors {
    ($accessors:expr, []) => {
        false
    };
    ($accessors:expr, [$($pattern:tt),+ $(,)?]) => {{
        let patterns_len = 0 $(+ { let _ = stringify!($pattern); 1 })*;
        if $accessors.len() != patterns_len {
            false
        } else {
            let mut iter = $accessors.iter();
            $crate::matches_accessors_inner!(iter, $($pattern),*)
        }
    }};
}

#[macro_export]
macro_rules! matches_accessors_inner {
    ($iter:expr) => {
        $iter.next().is_none()
    };
    ($iter:expr, $key:expr) => {
        match $iter.next() {
            Some($crate::Accessor::Key(k)) if k == $key => $iter.next().is_none(),
            _ => false,
        }
    };
    ($iter:expr, _) => {
        match $iter.next() {
            Some(_) => $iter.next().is_none(),
            None => false,
        }
    };
    ($iter:expr, $key:expr, $($rest:tt),* $(,)?) => {
        match $iter.next() {
            Some($crate::Accessor::Key(k)) if k == $key => $crate::matches_accessors_inner!($iter, $($rest),*),
            _ => false,
        }
    };
    ($iter:expr, _, $($rest:tt),* $(,)?) => {
        match $iter.next() {
            Some(_) => $crate::matches_accessors_inner!($iter, $($rest),*),
            None => false,
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::accessor::Accessor;

    #[test]
    fn test_matches_accessors() {
        let accessors = vec![
            Accessor::Key("tool".to_string()),
            Accessor::Key("uv".to_string()),
            Accessor::Key("sources".to_string()),
            Accessor::Key("local".to_string()),
            Accessor::Key("workspace".to_string()),
        ];

        // Exact match
        assert!(matches_accessors!(
            &accessors,
            ["tool", "uv", "sources", "local", "workspace"]
        ));

        // Wildcard
        assert!(matches_accessors!(
            &accessors,
            ["tool", "uv", "sources", _, "workspace"]
        ));

        // Partial match should fail
        assert!(!matches_accessors!(&accessors, ["tool", "uv", "sources"]));

        // Mismatch
        assert!(!matches_accessors!(&accessors, ["tool", "uv", "invalid"]));
        assert!(!matches_accessors!(
            &accessors,
            ["tool", "uv", "sources", "local", "invalid"]
        ));

        // Different array length
        assert!(!matches_accessors!(
            &accessors,
            ["tool", "uv", "sources", "local", "workspace", "extra"]
        ));

        // Empty accessors
        let empty_accessors: Vec<Accessor> = vec![];
        assert!(!matches_accessors!(&empty_accessors, ["tool"]));
    }

    #[test]
    fn test_matches_accessors_with_index() {
        let accessors = vec![
            Accessor::Key("tool".to_string()),
            Accessor::Key("uv".to_string()),
            Accessor::Index(0),
            Accessor::Key("workspace".to_string()),
        ];

        // Pattern with index
        assert!(matches_accessors!(
            &accessors,
            ["tool", "uv", _, "workspace"]
        ));

        // Pattern with specified index (should not match)
        assert!(!matches_accessors!(
            &accessors,
            ["tool", "uv", "sources", "workspace"]
        ));
    }
}
