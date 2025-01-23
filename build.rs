fn main() {
    if cfg!(target_os = "windows") {
        if let Ok(asio_sdk_dir) = std::env::var("ASIO_SDK_DIR") {
            println!("cargo:rustc-env=ASIO_SDK_DIR={}", asio_sdk_dir);
            println!("cargo:rustc-link-search=native={}/lib", asio_sdk_dir);
        } else {
            println!("cargo:warning=ASIO_SDK_DIR environment variable not set. ASIO support will be disabled.");
        }
    }
}
