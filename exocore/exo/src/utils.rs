use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub fn edit_string<S: AsRef<str>, V, R>(content: S, validator: V) -> R
where
    V: Fn(&str) -> anyhow::Result<R>,
{
    let temp_file = tempfile::NamedTempFile::new().expect("Couldn't create temp file");

    {
        let mut file = std::fs::File::create(temp_file.path()).expect("Couldn't create temp file");
        file.write_all(content.as_ref().as_ref())
            .expect("Couldn't write to temp file");
    }

    let result: R;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    loop {
        #[allow(clippy::needless_borrow)] // https://github.com/rust-lang/rust-clippy/issues/9778
        Command::new(&editor)
            .arg(temp_file.path().as_os_str())
            .status()
            .expect("Couldn't launch editor");

        let content = std::fs::read_to_string(temp_file.path()).expect("Couldn't read temp file");
        match validator(&content) {
            Ok(ret) => {
                result = ret;
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    std::fs::read_to_string(temp_file.path()).expect("Couldn't read temp file");

    result
}

pub fn expand_tild<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();

    if path.starts_with("~/") {
        let rest = path.strip_prefix("~/")?;
        let home_dir =
            dirs_next::home_dir().ok_or_else(|| anyhow!("Couldn't find home directory"))?;
        Ok(home_dir.join(rest))
    } else {
        Ok(path.to_owned())
    }
}
