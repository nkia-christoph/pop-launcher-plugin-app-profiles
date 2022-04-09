use std::{
  fs::{
    create_dir_all,
    File,
    OpenOptions
  },
  io::Result,
  path::PathBuf,
};




pub fn log_file(path: &PathBuf, file_name: &str) -> Result<File> {
  create_dir_all(path)?;
  OpenOptions::new()
    .create(true)
    .append(true)
    .open(
      path.join(file_name).as_path()
    )
}