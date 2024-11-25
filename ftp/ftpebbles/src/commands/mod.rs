pub mod handler;
pub mod pasv;
pub mod list;
pub mod cwd;
pub mod retr;
pub mod dele;
pub mod stor;
pub mod port;

// Convert line endings (e.g., `\n` -> `\r\n`)
fn convert_to_ascii(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for &byte in data {
        if byte == b'\n' {
            result.push(b'\r');
        }
        result.push(byte);
    }
    result
}