use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

#[allow(dead_code)]
pub fn read_wasm<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
  let mut file = fs::File::open(path)?;
  let mut tmp = [0; 8];
  let mut buffer = vec![];
  let _ = file.read_exact(&mut tmp)?;
  file.read_to_end(&mut buffer)?;
  Ok(buffer)
}
