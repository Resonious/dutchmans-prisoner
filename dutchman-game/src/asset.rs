use std::os;
// use std::io::fs::PathExtensions;

pub fn path(asset_path: &str) -> Path {
    let mut exe_path = match os::self_exe_path() {
        Some(p) => p,
        None => {
            panic!("Couldn't get executable path :(");
        }
    };
    exe_path.push("../assets");
    exe_path.push(asset_path);

    exe_path
}
