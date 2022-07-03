use std::{fs, io};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=proto");

    std::env::set_var("PROTOC_NO_VENDOR", "");

    let protos = find_protos(Path::new("./proto"))?;
    println!("Done: {protos:?}");

    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.compile_protos(&protos, &["./proto"])?;

    Ok(())
}

fn find_protos(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        println!("F: {:?}", entry.path());

        if entry.file_type()?.is_dir() {
            result.append(&mut find_protos(&entry.path())?);
        } else if entry.file_type()?.is_file() && entry.file_name().to_str().unwrap().ends_with(".proto") {
            println!("Found proto file!");
            result.push(entry.path())
        }
    }

    Ok(result)
}
