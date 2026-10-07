fn main() {
    windows();
}

#[cfg(windows)]
fn windows() {
    println!("cargo:rerun-if-changed=../../package/windows.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../package/windows.ico");
    res.set("FileDescription", "CubeLauncher");
    res.set("ProductName", "CubeLauncher");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn windows() {}
