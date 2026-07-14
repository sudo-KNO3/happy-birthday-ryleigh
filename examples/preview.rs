// Dev-only: render a scene natively and dump raw RGBA for visual inspection.
// Usage: cargo run --release --example preview -- <out.raw> <w> <h> <seed> [kick]
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let w: usize = a[2].parse().unwrap();
    let h: usize = a[3].parse().unwrap();
    let seed: u32 = a[4].parse().unwrap();
    let kick: bool = a.get(5).map(|s| s == "1").unwrap_or(false);
    let buf = ryleigh_banff::render_rgba(w, h, seed, kick);
    std::fs::File::create(&a[1]).unwrap().write_all(&buf).unwrap();
}
