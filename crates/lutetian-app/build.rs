fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=../../assets/icons/lutetian.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icons/lutetian.ico");
        if let Err(err) = res.compile() {
            println!("cargo:warning=Failed to compile Windows icon resource: {err}");
        }
    }
}
