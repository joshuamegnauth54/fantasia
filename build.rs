use std::{env, fs, path::Path};

fn main() {
    // Recompile if migrations change
    println!("cargo:rerun-if-changed=migrations");

    let manifest =
        env::var("CARGO_MANIFEST_DIR").expect("`build.rs` expects to be invoked with Cargo");
    let manifest = Path::new(&manifest);

    let profile = env::var("PROFILE").expect("Cargo should provide a build profile");
    let target = manifest.join("target").join(profile);

    // Copy static files to output
    for file in ["dev.env", "fantasia_small.toml", "fantasia_full.toml"] {
        let src = manifest.join(file);
        let dst = target.join(file);

        fs::copy(&*src, &*dst).unwrap_or_else(|e| {
            panic!("Copying `{}` to `{}`\n\t{e}", src.display(), dst.display())
        });
    }
}
