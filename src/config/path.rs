use std::path::PathBuf;

pub(crate) const BIN_POSTFIX: &str = "usr/bin/zps";
pub(crate) const CACHE: &str = "var/cache/zps";
pub(crate) const DATA: &str = "var/lib/zps";
pub(crate) const ETC: &str = "etc/zps";
pub(crate) const TMP: &str = "var/tmp/zps";
pub(crate) const TREE: &str = "zps";

pub fn resolve_tree() -> PathBuf {
    return match std::env::current_exe() {
        Err(_) => PathBuf::from("/"),
        Ok(mut p) => {
            if !p.ends_with(BIN_POSTFIX) {
                return PathBuf::from("/");
            }

            for _ in 1..3 {
                p.pop();
            }
            p
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tree() {
        assert_eq!(resolve_tree().to_str().unwrap(), "/".to_string());
    }
}