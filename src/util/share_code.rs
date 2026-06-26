use crate::args::ARGS;
use crate::pasta::Pasta;
use crate::util::animalnumbers::to_u64 as animal_to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;

const RESERVED_CODES: &[&str] = &[
    "admin",
    "archive",
    "auth",
    "auth_admin",
    "auth_edit_private",
    "auth_file",
    "auth_raw",
    "auth_remove_private",
    "auth_url",
    "edit",
    "favicon.ico",
    "file",
    "guide",
    "incorrect",
    "list",
    "p",
    "qr",
    "raw",
    "remove",
    "robots.txt",
    "secure_file",
    "static",
    "success",
    "u",
    "upload",
    "url",
];

pub fn is_valid_custom_share_code(code: &str) -> bool {
    let code = code.trim();
    (3..=64).contains(&code.len())
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        && !is_reserved_share_code(code)
}

pub fn is_reserved_share_code(code: &str) -> bool {
    let normalized = code.trim().to_ascii_lowercase();
    RESERVED_CODES.contains(&normalized.as_str())
}

pub fn generated_code_to_id(code: &str) -> Option<u64> {
    if ARGS.hash_ids {
        hashid_to_u64(code).ok()
    } else {
        animal_to_u64(code).ok()
    }
}

pub fn find_pasta_index_by_code(pastas: &[Pasta], code: &str) -> Option<usize> {
    pastas.iter().position(|pasta| pasta.matches_share_code(code))
}

pub fn share_code_exists(pastas: &[Pasta], code: &str) -> bool {
    pastas.iter().any(|pasta| pasta.matches_share_code(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_custom_share_codes() {
        assert!(is_valid_custom_share_code("work-note_1"));
        assert!(is_valid_custom_share_code("1234"));
        assert!(is_valid_custom_share_code("abc"));
        assert!(!is_valid_custom_share_code("ab"));
        assert!(!is_valid_custom_share_code("upload"));
        assert!(!is_valid_custom_share_code("../secret"));
        assert!(!is_valid_custom_share_code("带中文"));
        assert!(!is_valid_custom_share_code("has space"));
    }
}
