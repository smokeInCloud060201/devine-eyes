// build.rs - Help linker find Npcap on Windows
fn main() {
    #[cfg(target_os = "windows")]
    {
        // Try to find Npcap installation
        let npcap_paths = vec![
            r"C:\Program Files\Npcap",
            r"C:\Program Files (x86)\Npcap",
            r"C:\WINDOWS\System32", // wpcap.dll might be here
        ];

        for path in npcap_paths {
            let lib_path = format!(r"{}\Lib\x64", path);
            if std::path::Path::new(&lib_path).exists() {
                println!("cargo:rustc-link-search=native={}", lib_path);
                println!("cargo:warning=Found Npcap at: {}", path);
                return;
            }
        }

        // Check if wpcap.dll exists in System32
        if std::path::Path::new(r"C:\WINDOWS\System32\wpcap.dll").exists() {
            println!("cargo:rustc-link-search=native=C:\\WINDOWS\\System32");
            println!("cargo:warning=Found wpcap.dll in System32");
            return;
        }

        // If not found, print helpful message
        println!("cargo:warning=Npcap not found. Please install Npcap from https://nmap.org/npcap/");
        println!("cargo:warning=Make sure to enable 'WinPcap API-compatible Mode' during installation");
        println!("cargo:warning=After installation, restart your computer and rebuild");
    }
}

