/// Format bytes into human-readable format (B, KB, MB, GB, TB)
pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k: f64 = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log2() / k.log2();
    let i = i.floor() as usize;
    let i = i.min(sizes.len() - 1);
    let size = sizes[i];
    format!("{:.2} {}", bytes as f64 / k.powi(i as i32), size)
}

