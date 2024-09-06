use std::path::{Component, Path, PathBuf};

pub fn child_to_abs_path<P: AsRef<Path>, C: AsRef<Path>>(parent: P, child: C) -> PathBuf {
    let parent_path = parent.as_ref();
    let child_path = child.as_ref();

    if child_path.is_absolute() {
        return child_path.to_path_buf();
    }

    let parent_path_buf = PathBuf::from(parent_path);
    clean_path(parent_path_buf.join(child_path))
}

pub fn child_to_relative_path<P: AsRef<Path>, C: AsRef<Path>>(parent: P, child: C) -> PathBuf {
    child
        .as_ref()
        .strip_prefix(parent.as_ref())
        .unwrap_or_else(|_| child.as_ref())
        .to_owned()
}

pub fn clean_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut out = PathBuf::new();

    for (i, component) in path.components().enumerate() {
        match component {
            Component::CurDir if i > 0 => {}
            other => out.push(other),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_path() {
        assert_eq!(clean_path("./test/./test"), PathBuf::from("./test/test"));
        assert_eq!(clean_path("./test/././test"), PathBuf::from("./test/test"));
        assert_eq!(clean_path("/test/test"), PathBuf::from("/test/test"));
    }

    #[test]
    fn to_absolute_path() {
        assert_eq!(
            child_to_abs_path("/parent", "child"),
            PathBuf::from("/parent/child")
        );
        assert_eq!(
            child_to_abs_path("/parent", "./child"),
            PathBuf::from("/parent/child")
        );
        assert_eq!(
            child_to_abs_path("/parent", "././child"),
            PathBuf::from("/parent/child")
        );
        assert_eq!(child_to_abs_path("/", "././child"), PathBuf::from("/child"));
    }

    #[test]
    fn to_relative_path() {
        assert_eq!(
            child_to_relative_path("/parent", "/parent/child"),
            PathBuf::from("child")
        );
        assert_eq!(
            child_to_relative_path("/bleh", "/parent/child"),
            PathBuf::from("/parent/child")
        );
    }
}
