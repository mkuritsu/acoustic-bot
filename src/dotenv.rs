use std::fs;

const DOTENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");

pub fn load_dotenv_vars() -> anyhow::Result<()> {
    fs::read_to_string(DOTENV_PATH)?
        .lines()
        .filter(|line| !line.trim().is_empty()) // filter blank lines
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .for_each(|(key, value)| {
            unsafe { std::env::set_var(key, value) };
        });
    Ok(())
}
