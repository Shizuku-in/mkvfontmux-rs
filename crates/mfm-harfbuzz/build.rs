use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../third_party/harfbuzz/src");
    let amalgamation = root.join("harfbuzz-subset.cc");
    assert!(
        amalgamation.exists(),
        "HarfBuzz sources are missing; run `git submodule update --init`"
    );

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .warnings(false)
        .file(&amalgamation);
    if build.get_compiler().is_like_msvc() {
        build.flag("/bigobj").flag("/utf-8");
    } else {
        build
            .flag_if_supported("-fno-exceptions")
            .flag_if_supported("-fno-rtti");
    }
    build.compile("harfbuzz-subset");

    println!("cargo:rerun-if-changed={}", root.display());
}
