use std::{
    io::Write,
    path::{Path, PathBuf},
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
