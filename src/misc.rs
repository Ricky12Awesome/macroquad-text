use std::fs::File;
pub use std::io::Error as IoError;
pub use std::io::ErrorKind as IoErrorKind;
use std::io::Read;
pub use std::io::Result as IoResult;
use std::path::Path;

pub fn read_file(path: impl AsRef<Path>) -> IoResult<Vec<u8>> {
  let mut file = File::open(path)?;
  let total_bytes = file.metadata()?.len() as usize;
  let mut bytes = Vec::with_capacity(total_bytes);

  file.read_to_end(&mut bytes)?;

  Ok(bytes)
}
