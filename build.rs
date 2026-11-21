use std::{env, fs, io, path::Path};

fn main() {
    // Stage the prototype fixtures first. Team-owned JSON takes precedence file by file.
    let out = env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR");
    let staging = Path::new(&out).join("engine-data");
    if staging.exists() {
        fs::remove_dir_all(&staging).expect("clear staged engine data");
    }
    for root in ["tests/fixtures/engine", "data"] {
        println!("cargo:rerun-if-changed={root}");
        let source = Path::new(root);
        if source.is_dir() {
            copy_json(source, &staging).unwrap_or_else(|err| panic!("stage {root}: {err}"));
        }
    }

    tauri_build::build();
}

fn copy_json(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let output = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_json(&entry.path(), &output)?;
        } else if entry.path().extension().is_some_and(|ext| ext == "json") {
            fs::copy(entry.path(), output)?;
        }
    }
    Ok(())
}
