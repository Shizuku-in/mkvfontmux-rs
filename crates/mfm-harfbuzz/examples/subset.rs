//! `cargo run -p mfm-harfbuzz --example subset -- <font> <face-index> <text-file> <out>`
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [font, index, text, out] = args.as_slice() else {
        eprintln!("usage: subset <font> <face-index> <text-file> <out>");
        std::process::exit(2);
    };
    let data = std::fs::read(font).expect("read font");
    let text = std::fs::read_to_string(text).expect("read text");
    let start = std::time::Instant::now();
    let bytes = mfm_harfbuzz::subset(
        &data,
        index.parse().expect("face index"),
        text.chars(),
        &Default::default(),
    )
    .expect("subset");
    eprintln!(
        "{} -> {} bytes in {:?}",
        data.len(),
        bytes.len(),
        start.elapsed()
    );
    std::fs::write(out, bytes).expect("write");
}
