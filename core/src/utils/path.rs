use std::path::{Path, PathBuf};

pub fn child_to_abs_path<P: AsRef<Path>, C: AsRef<Path>>(parent: P, child: C) -> PathBuf {
    let parent_path = parent.as_ref();
    let child_path = child.as_ref();

    if child_path.is_absolute() {
        return child_path.to_path_buf();
    }

    let parent_path_buf = PathBuf::from(parent_path);
    parent_path_buf.join(child_path)
}
