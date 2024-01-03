use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct OptionalFile(Option<File>);
impl OptionalFile {
    pub fn new(name: &str) -> Self {
        Self(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(name)
                .ok(),
        )
    }
}
impl Write for OptionalFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(file) = self.0.as_mut() {
            write!(file, "{}", std::str::from_utf8(buf).unwrap())?;
            return Ok(buf.len());
        }

        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
