use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct Packager {
    target_dir: PathBuf,
    writer: ZipWriter<File>,
    options: FileOptions,
    buffer: Vec<u8>,
}

impl Packager {
    pub fn new<P: AsRef<Path>>(target_dir: P, package_name: &str) -> Result<Self, PackagerError> {
        let target_path = target_dir.as_ref().join(package_name);
        let target_file = File::create(target_path)?;

        Ok(Self {
            target_dir: target_dir.as_ref().into(),
            writer: ZipWriter::new(target_file),
            options: FileOptions::default()
                .unix_permissions(0o755)
                .compression_method(CompressionMethod::Deflated),
            buffer: Vec::new(),
        })
    }

    pub fn collect_dir<P: AsRef<Path>>(&mut self, directory: P) -> Result<(), PackagerError> {
        let mut to_process = vec![directory.as_ref().to_owned()];

        while !to_process.is_empty() {
            for path in std::mem::take(&mut to_process) {
                let prefix = path.strip_prefix(&self.target_dir)?;

                if prefix != Path::new("") {
                    self.writer
                        .add_directory(prefix.to_string_lossy(), self.options)?;
                }

                let content = path.read_dir()?;
                for entry in content {
                    let entry = entry?.path();

                    if entry.is_dir() {
                        to_process.push(entry);
                        continue;
                    }

                    if entry.is_file() || entry.is_symlink() {
                        self.collect_file(entry)?;
                    } else {
                        eprintln!(
                            "-- Warning, not packaging file node {} of unknown type",
                            entry.display()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub fn collect_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PackagerError> {
        let prefix = path.as_ref().strip_prefix(&self.target_dir)?;

        self.writer
            .start_file(prefix.to_string_lossy(), self.options)?;

        let mut entry_file = File::open(path)?;
        entry_file.read_to_end(&mut self.buffer)?;
        self.writer.write_all(&self.buffer)?;

        Ok(())
    }

    pub fn finish(self) {
        drop(self);
    }
}

#[derive(Debug, Error)]
pub enum PackagerError {
    #[error("an I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("an error occurred while writing the ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("failed to strip the prefix of a directory")]
    Prefix(#[from] std::path::StripPrefixError),
}
