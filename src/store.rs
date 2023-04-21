use crate::data::{Addlist, AddlistConfig};
use std::{fs, io::Write};

/// Writes addlist to (multiple) file(s).
///
/// Based on [lib::config::Config].size attribute the addlist is split into multiple files or written all at one file.
///
/// # Errors
/// - If file could not be created or manipulated.
pub fn write_to_file(config: AddlistConfig, addlist: Addlist) -> std::io::Result<()> {
    match config.config.size {
        Some(size) => addlist
            .list
            .chunks(size.get())
            .map(|data| data.join("\r\n"))
            .enumerate()
            .try_for_each(|(num, data)| {
                let mut file = fs::File::create(format!(
                    "{}/{}-{}.addlist",
                    config.config.path, num, addlist.name
                ))?;
                file.write_all(data.as_bytes()).map(|_| ())
            })?,
        None => {
            let mut file =
                fs::File::create(format!("{}/{}.addlist", config.config.path, addlist.name))?;
            file.write_all(addlist.list.join("\r\n").as_bytes())?
        }
    }
    Ok(())
}