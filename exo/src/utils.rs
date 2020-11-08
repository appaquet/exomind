use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub fn shell_prompt(question: &str, default: Option<&str>) -> anyhow::Result<Option<String>> {
    print!("{}", question);

    if let Some(default) = default.as_ref() {
        print!(" (default: {}): ", default);
    } else {
        print!(": ");
    }

    std::io::stdout()
        .flush()
        .map_err(|err| anyhow!("Couldn't flush to stdout: {}", err))?;

    let mut resp = String::new();
    std::io::stdin()
        .read_line(&mut resp)
        .map_err(|err| anyhow!("Couldn't read from stding: {}", err))?;

    let resp_trimmed = resp.trim();

    if resp_trimmed == "" {
        return Ok(default.map(|s| s.to_string()));
    }

    Ok(Some(resp_trimmed.to_string()))
}

pub fn edit_file<P: AsRef<Path>, V>(file: P, validator: V)
where
    V: Fn(&Path) -> bool,
{
    let temp_file = tempfile::NamedTempFile::new().expect("Couldn't create temp file");

    std::fs::copy(&file, temp_file.path()).expect("Couldn't copy edit file to temp file");
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    loop {
        Command::new(&editor)
            .arg(temp_file.path().as_os_str())
            .status()
            .expect("Couldn't launch editor");

        if validator(temp_file.path()) {
            break;
        }
    }

    std::fs::copy(temp_file.path(), &file)
        .expect("Couldn't copy edited temp file to original file");
}

pub fn edit_string<S: AsRef<str>, V>(content: S, validator: V) -> String
where
    V: Fn(&str) -> bool,
{
    let temp_file = tempfile::NamedTempFile::new().expect("Couldn't create temp file");

    {
        let mut file = std::fs::File::create(temp_file.path()).expect("Couldn't create temp file");
        file.write_all(content.as_ref().as_ref())
            .expect("Couldn't write to temp file");
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    loop {
        Command::new(&editor)
            .arg(temp_file.path().as_os_str())
            .status()
            .expect("Couldn't launch editor");

        let content = std::fs::read_to_string(temp_file.path()).expect("Couldn't read temp file");
        if validator(&content) {
            break;
        }
    }

    std::fs::read_to_string(temp_file.path()).expect("Couldn't read temp file")
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
