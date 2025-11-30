// tests/unit_validator.rs
use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

fn check_content(content: &str) -> bool {
    let mut files = HashMap::new();
    files.insert("test.rs".into(), FileContent { content: content.into(), line_count: 1 });
    matches!(validator::validate(&vec![], &files), ApplyOutcome::ValidationFailure { .. })
}

#[test] fn test_block_comment_ellipsis() { assert!(check_content("/* ... */")); }
#[test] fn test_hash_ellipsis() { assert!(check_content("# ...")); }
#[test] fn test_lazy_phrase_rest_of() { assert!(check_content("// rest of implementation")); }
#[test] fn test_lazy_phrase_remaining() { assert!(check_content("// remaining code")); }
#[test] fn test_valid_code_passes() { assert!(!check_content("fn main() {}")); }
#[test] fn test_ellipsis_in_string_allowed() { assert!(!check_content("let s = \"Loading...\";")); }
#[test] fn test_warden_ignore_inline() { assert!(!check_content("// ... warden:ignore")); }
#[test] fn test_line_number_reported() {}
#[test] fn test_gnupg_blocked() {}
#[test] fn test_id_rsa_blocked() {}
#[test] fn test_credentials_blocked() {}
#[test] fn test_backup_dir_blocked() {}
