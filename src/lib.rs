//! A generative Rocky Mountain birthday keepsake for Ryleigh.
//!
//! Every mountain is rasterised pixel-by-pixel here in Rust and shipped to the
//! browser as WebAssembly. Give it a `seed` and you get a one-of-a-kind alpine
//! scene — layered ridgelines, snow caps, a drifting sky, and a mirrored lake.
//! Turn `kick` on and a certain birthday tradition makes a cameo. 🥋

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use core::f32::consts::PI;

/// Tiny deterministic PRNG (xorshift32): the same seed always paints the same
/// mountain, so a keepsake is reproducible and no OS entropy is needed in wasm.
struct Rng(u32);
impl Rng {
    fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 0x9E37_79B9 } else { seed })
    }
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// f32 in [0, 1)
    fn f(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// f32 in [lo, hi)
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f()
    }
}

type Col = (f32, f32, f32);

fn mix(a: Col, b: Col, t: f32) -> Col {
    let t = t.clamp(0.0, 1.0);
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
    )
}
fn scale(a: Col, s: f32) -> Col {
    (a.0 * s, a.1 * s, a.2 * s)
}
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct Canvas<'a> {
    buf: &'a mut [u8],
    w: usize,
    h: usize,
}
impl<'a> Canvas<'a> {
    fn set(&mut self, x: usize, y: usize, c: Col) {
        if x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) * 4;
        self.buf[i] = (c.0.clamp(0.0, 1.0) * 255.0) as u8;
        self.buf[i + 1] = (c.1.clamp(0.0, 1.0) * 255.0) as u8;
        self.buf[i + 2] = (c.2.clamp(0.0, 1.0) * 255.0) as u8;
        self.buf[i + 3] = 255;
    }
    fn get(&self, x: usize, y: usize) -> Col {
        let x = x.min(self.w - 1);
        let y = y.min(self.h - 1);
        let i = (y * self.w + x) * 4;
        (
            self.buf[i] as f32 / 255.0,
            self.buf[i + 1] as f32 / 255.0,
            self.buf[i + 2] as f32 / 255.0,
        )
    }
    fn blend(&mut self, x: usize, y: usize, c: Col, a: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let bg = self.get(x, y);
        self.set(x, y, mix(bg, c, a));
    }
    fn fill_disc(&mut self, cx: f32, cy: f32, r: f32, c: Col) {
        if r <= 0.0 {
            return;
        }
        let x0 = (cx - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil().max(0.0) as usize).min(self.w);
        let y0 = (cy - r).floor().max(0.0) as usize;
        let y1 = ((cy + r).ceil().max(0.0) as usize).min(self.h);
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx * dx + dy * dy <= r * r {
                    self.set(x, y, c);
                }
            }
        }
    }
    fn glow(&mut self, cx: f32, cy: f32, r: f32, c: Col) {
        let x0 = (cx - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil().max(0.0) as usize).min(self.w);
        let y0 = (cy - r).floor().max(0.0) as usize;
        let y1 = ((cy + r).ceil().max(0.0) as usize).min(self.h);
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d = (dx * dx + dy * dy).sqrt() / r;
                if d < 1.0 {
                    self.blend(x, y, c, (1.0 - d) * (1.0 - d) * 0.7);
                }
            }
        }
    }
    fn thick_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, c: Col) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let steps = len.ceil() as usize;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            self.fill_disc(x0 + dx * t, y0 + dy * t, width * 0.5, c);
        }
    }
}

struct Sky {
    top: Col,
    horizon: Col,
    night: bool,
    warm: bool,
}
fn pick_sky(rng: &mut Rng) -> Sky {
    match rng.next_u32() % 4 {
        // dawn
        0 => Sky { top: (0.16, 0.22, 0.38), horizon: (0.97, 0.68, 0.50), night: false, warm: true },
        // clear day
        1 => Sky { top: (0.22, 0.46, 0.72), horizon: (0.76, 0.89, 0.96), night: false, warm: false },
        // dusk
        2 => Sky { top: (0.20, 0.15, 0.32), horizon: (0.93, 0.52, 0.38), night: false, warm: true },
        // starry night
        _ => Sky { top: (0.03, 0.05, 0.13), horizon: (0.10, 0.16, 0.30), night: true, warm: false },
    }
}

// ---------------------------------------------------------------------------
// Oil-painting stylisation. Reused for both the generated scene and (later) a
// real Banff photo. Classic intensity-histogram "oil" filter: each pixel takes
// the average colour of the most common brightness band in its neighbourhood,
// smearing the image into painterly blobs. A faint canvas weave adds impasto.
// ---------------------------------------------------------------------------

const OIL_LEVELS: usize = 20;

fn oil_filter(src: &[u8], w: usize, h: usize, radius: i32) -> Vec<u8> {
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let mut count = [0u32; OIL_LEVELS];
            let mut rs = [0f32; OIL_LEVELS];
            let mut gs = [0f32; OIL_LEVELS];
            let mut bs = [0f32; OIL_LEVELS];
            for dy in -radius..=radius {
                let ny = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                let row = ny * w;
                for dx in -radius..=radius {
                    let nx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                    let i = (row + nx) * 4;
                    let r = src[i] as f32;
                    let g = src[i + 1] as f32;
                    let b = src[i + 2] as f32;
                    let mut lvl = ((r + g + b) / 3.0 / 255.0 * (OIL_LEVELS as f32 - 1.0)) as usize;
                    if lvl >= OIL_LEVELS {
                        lvl = OIL_LEVELS - 1;
                    }
                    count[lvl] += 1;
                    rs[lvl] += r;
                    gs[lvl] += g;
                    bs[lvl] += b;
                }
            }
            let mut best = 0usize;
            for k in 1..OIL_LEVELS {
                if count[k] > count[best] {
                    best = k;
                }
            }
            let cnt = count[best].max(1) as f32;
            let o = (y * w + x) * 4;
            out[o] = (rs[best] / cnt) as u8;
            out[o + 1] = (gs[best] / cnt) as u8;
            out[o + 2] = (bs[best] / cnt) as u8;
            out[o + 3] = 255;
        }
    }
    out
}

fn tex_noise(x: usize, y: usize) -> f32 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n >> 8) & 0xffff) as f32 / 65535.0
}

fn apply_canvas_texture(buf: &mut [u8], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let weave = 0.02 * (((x + y) as f32) * 0.7).sin();
            let f = (0.93 + 0.09 * tex_noise(x, y) + weave).clamp(0.85, 1.05);
            for k in 0..3 {
                buf[i + k] = (buf[i + k] as f32 * f).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn oil_radius(w: usize, h: usize) -> i32 {
    ((w.min(h) / 220).max(2) as i32).min(6)
}

/// Render one mountain scene into a fresh RGBA8 buffer (`w*h*4` bytes).
/// Pure Rust, no wasm types, so it is unit-testable on the host too.
pub fn render_rgba(width: usize, height: usize, seed: u32, kick: bool) -> Vec<u8> {
    let w = width.max(2);
    let h = height.max(2);
    let mut buf = vec![0u8; w * h * 4];
    let mut c = Canvas { buf: &mut buf, w, h };
    let mut rng = Rng::new(seed);

    let sky = pick_sky(&mut rng);
    let hy = h as f32 * rng.range(0.60, 0.68); // horizon / lake top
    let hyi = hy as usize;

    // --- sky gradient ---
    for y in 0..hyi {
        let col = mix(sky.top, sky.horizon, smoothstep(y as f32 / hy));
        for x in 0..w {
            c.set(x, y, col);
        }
    }

    // --- stars ---
    if sky.night {
        let n = (w * h / 1100).max(30);
        for _ in 0..n {
            let sx = rng.range(0.0, w as f32) as usize;
            let sy = rng.range(0.0, hy * 0.92) as usize;
            let b = rng.range(0.35, 1.0);
            c.blend(sx, sy, (1.0, 1.0, 0.96), b);
            if rng.f() < 0.12 {
                c.blend(sx + 1, sy, (1.0, 1.0, 0.96), b * 0.5);
            }
        }
    }

    // --- sun / moon ---
    let bx = rng.range(w as f32 * 0.15, w as f32 * 0.85);
    if sky.night {
        let by = rng.range(hy * 0.14, hy * 0.42);
        let r = rng.range(w as f32 * 0.028, w as f32 * 0.045);
        c.glow(bx, by, r * 3.2, (0.85, 0.88, 0.98));
        c.fill_disc(bx, by, r, (0.94, 0.96, 1.0));
    } else {
        let by = if sky.warm {
            rng.range(hy * 0.52, hy * 0.78)
        } else {
            rng.range(hy * 0.18, hy * 0.46)
        };
        let r = rng.range(w as f32 * 0.032, w as f32 * 0.052);
        let sun = if sky.warm { (1.0, 0.85, 0.60) } else { (1.0, 0.97, 0.86) };
        c.glow(bx, by, r * 4.0, sun);
        c.fill_disc(bx, by, r, mix(sun, (1.0, 1.0, 1.0), 0.45));
    }

    // --- mountain layers (far -> near) ---
    let n_layers = 5usize;
    struct Layer {
        ty: Vec<f32>,
        color: Col,
        snow: bool,
        amp: f32,
    }
    let mut layers: Vec<Layer> = Vec::with_capacity(n_layers);
    for i in 0..n_layers {
        let depth = i as f32 / (n_layers as f32 - 1.0);
        let baseline = hy * (0.42 + 0.50 * depth);
        let amp = hy * (0.38 - 0.22 * depth);
        let f1 = rng.range(0.8, 1.8);
        let p1 = rng.range(0.0, PI * 2.0);
        let f2 = rng.range(2.5, 4.5);
        let p2 = rng.range(0.0, PI * 2.0);
        let f3 = rng.range(5.0, 9.0);
        let p3 = rng.range(0.0, PI * 2.0);
        let fj = rng.range(9.0, 15.0);
        let pj = rng.range(0.0, PI * 2.0);
        let mut ty = vec![0.0f32; w];
        for x in 0..w {
            let xn = x as f32 / w as f32;
            let mut v = 0.55 * (xn * PI * 2.0 * f1 + p1).sin()
                + 0.28 * (xn * PI * 2.0 * f2 + p2).sin()
                + 0.14 * (xn * PI * 2.0 * f3 + p3).sin();
            v += 0.06 * (xn * PI * 2.0 * fj + pj).sin() * depth; // craggier up front
            let prof = ((v / 0.97) * 0.5 + 0.5).clamp(0.0, 1.0);
            ty[x] = baseline - prof * amp;
        }
        let far = (0.62, 0.68, 0.78);
        let near = (0.09, 0.16, 0.12);
        let base = mix(far, near, depth);
        let hazed = mix(base, sky.horizon, (1.0 - depth) * 0.55); // atmospheric haze
        layers.push(Layer { ty, color: hazed, snow: depth < 0.55, amp });
    }

    for y in 0..hyi {
        for x in 0..w {
            for i in (0..n_layers).rev() {
                let l = &layers[i];
                if (y as f32) >= l.ty[x] {
                    let d = (y as f32 - l.ty[x]).max(0.0);
                    // subtle vertical shading, lighter near the ridge
                    let shade = 1.0 - (d / l.amp.max(1.0)).min(1.0) * 0.28;
                    let mut col = scale(l.color, shade);
                    if l.snow {
                        let band = l.amp * 0.17;
                        if d < band {
                            col = mix(col, (0.95, 0.96, 0.99), (1.0 - d / band) * 0.85);
                        }
                    }
                    c.set(x, y, col);
                    break;
                }
            }
        }
    }

    // --- lake: a darkened, shimmering mirror of everything above ---
    let shimmer_phase = rng.range(0.0, PI * 2.0);
    let lake_tint = mix(sky.horizon, (0.14, 0.22, 0.30), 0.5);
    for y in hyi..h {
        let dy = y as f32 - hy;
        let depth_t = (dy / (h as f32 - hy)).clamp(0.0, 1.0);
        let src = hy - dy * 0.97;
        let sy = if src < 0.0 { 0.0 } else { src } as usize;
        for x in 0..w {
            let sh = ((y as f32 * 0.18) + shimmer_phase).sin() * (1.4 + depth_t * 3.6);
            let sx = ((x as f32 + sh).round() as isize).clamp(0, (w - 1) as isize) as usize;
            let mut col = c.get(sx, sy);
            col = (col.0 * 0.80, col.1 * 0.82, col.2 * 0.88);
            col = mix(col, lake_tint, 0.12 + depth_t * 0.18);
            c.set(x, y, col);
        }
    }
    for x in 0..w {
        c.blend(x, hyi, (1.0, 1.0, 1.0), 0.16);
        if hyi + 1 < h {
            c.blend(x, hyi + 1, (1.0, 1.0, 1.0), 0.06);
        }
    }

    // --- oil-painting stylisation over the whole landscape ---
    {
        let radius = oil_radius(w, h);
        let painted = oil_filter(&c.buf, w, h, radius);
        c.buf.copy_from_slice(&painted);
        apply_canvas_texture(c.buf, w, h);
    }

    // --- the inside joke, drawn crisp on top of the painting, on request ---
    if kick {
        draw_kick(&mut c, w, h);
    }

    buf
}

/// Foreground silhouette cameo: Ryleigh delivering a roundhouse kick to a
/// hapless, cane-wielding old fellow. Purely for birthday purposes.
fn draw_kick(c: &mut Canvas, w: usize, h: usize) {
    let dark: Col = (0.04, 0.05, 0.08);
    let pow: Col = (1.0, 0.86, 0.32);
    let wf = w as f32;
    let hf = h as f32;

    // foreground shore
    let shore_y = hf * 0.90;
    for y in (shore_y as usize)..h {
        for x in 0..w {
            let bump = ((x as f32 / wf) * PI * 3.0).sin() * 6.0;
            if (y as f32) > shore_y + bump {
                c.set(x, y, dark);
            }
        }
    }

    let feet_y = shore_y + 2.0;
    let s = hf * 0.16; // figure scale
    let base_x = wf * 0.40;

    // --- Ryleigh (the kicker), facing right ---
    let hip = (base_x, feet_y - s * 0.50);
    let shoulder = (base_x - s * 0.03, feet_y - s * 0.95);
    let head = (shoulder.0, shoulder.1 - s * 0.16);
    let lw = s * 0.07;
    c.thick_line(hip.0, hip.1, base_x, feet_y, lw, dark); // standing leg
    c.thick_line(hip.0, hip.1, shoulder.0, shoulder.1, lw, dark); // torso
    c.fill_disc(head.0, head.1, s * 0.11, dark); // head
    c.thick_line(head.0, head.1, head.0 - s * 0.16, head.1 - s * 0.04, lw * 0.7, dark); // ponytail
    c.thick_line(shoulder.0, shoulder.1, shoulder.0 - s * 0.28, shoulder.1 + s * 0.06, lw * 0.8, dark); // back arm
    c.thick_line(shoulder.0, shoulder.1, shoulder.0 + s * 0.22, shoulder.1 + s * 0.11, lw * 0.8, dark); // guard arm
    let knee = (hip.0 + s * 0.30, hip.1 - s * 0.06);
    let foot = (hip.0 + s * 0.80, hip.1 - s * 0.30);
    c.thick_line(hip.0, hip.1, knee.0, knee.1, lw * 1.1, dark); // kicking thigh
    c.thick_line(knee.0, knee.1, foot.0, foot.1, lw * 1.1, dark); // kicking shin

    // --- the old guy (reeling back with hat + cane) ---
    let o = s * 0.82;
    let ofeet_x = base_x + s * 1.08;
    let ofeet_y = feet_y;
    let ohip = (ofeet_x + o * 0.14, ofeet_y - o * 0.45);
    let oshoulder = (ofeet_x + o * 0.24, ofeet_y - o * 0.80);
    let ohead = (oshoulder.0 + o * 0.06, oshoulder.1 - o * 0.14);
    let olw = o * 0.07;
    c.thick_line(ohip.0, ohip.1, ofeet_x, ofeet_y, olw, dark); // back leg
    c.thick_line(ohip.0, ohip.1, ofeet_x + o * 0.24, ofeet_y, olw, dark); // front leg
    c.thick_line(ohip.0, ohip.1, oshoulder.0, oshoulder.1, olw, dark); // hunched torso
    c.fill_disc(ohead.0, ohead.1, o * 0.10, dark); // head
    c.thick_line(ohead.0 - o * 0.15, ohead.1 - o * 0.09, ohead.0 + o * 0.15, ohead.1 - o * 0.09, olw * 0.8, dark); // hat brim
    c.fill_disc(ohead.0, ohead.1 - o * 0.15, o * 0.07, dark); // hat crown
    c.thick_line(oshoulder.0, oshoulder.1, oshoulder.0 + o * 0.26, oshoulder.1 - o * 0.18, olw * 0.8, dark); // flailing arm
    c.thick_line(ofeet_x + o * 0.32, ofeet_y, ofeet_x + o * 0.42, ofeet_y - o * 0.55, olw * 0.7, dark); // cane

    // --- comic impact starburst at the point of contact ---
    let ix = foot.0 + s * 0.06;
    let iy = foot.1;
    for k in 0..8 {
        let ang = k as f32 / 8.0 * PI * 2.0;
        c.thick_line(ix, iy, ix + ang.cos() * s * 0.17, iy + ang.sin() * s * 0.17, lw * 0.55, pow);
    }
    c.fill_disc(ix, iy, s * 0.05, pow);
}

/// WebAssembly entry point. Returns an RGBA8 buffer the page wraps in an
/// `ImageData` and blits to a `<canvas>`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn render(width: u32, height: u32, seed: u32, kick: bool) -> Vec<u8> {
    render_rgba(width as usize, height as usize, seed, kick)
}

/// Oil-paint an arbitrary RGBA8 image (e.g. a real Banff photo pulled from a
/// `<canvas>` via `getImageData`). Returns a new stylised buffer of the same
/// size; if the length doesn't match `width*height*4` the input is returned
/// unchanged so the page can fail gracefully.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn oil_paint(bytes: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    if bytes.len() != w * h * 4 {
        return bytes;
    }
    let mut out = oil_filter(&bytes, w, h, oil_radius(w, h));
    apply_canvas_texture(&mut out, w, h);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_is_rgba_sized_and_opaque() {
        let b = render_rgba(80, 60, 42, false);
        assert_eq!(b.len(), 80 * 60 * 4);
        assert!(b.iter().skip(3).step_by(4).all(|&a| a == 255));
    }

    #[test]
    fn same_seed_same_mountain() {
        assert_eq!(render_rgba(64, 48, 7, false), render_rgba(64, 48, 7, false));
    }

    #[test]
    fn different_seed_different_mountain() {
        assert_ne!(render_rgba(64, 48, 1, false), render_rgba(64, 48, 2, false));
    }

    #[test]
    fn kick_changes_the_scene() {
        assert_ne!(render_rgba(160, 120, 5, false), render_rgba(160, 120, 5, true));
    }

    #[test]
    fn scene_is_not_a_flat_fill() {
        let b = render_rgba(100, 100, 3, false);
        let first = [b[0], b[1], b[2]];
        assert!(b.chunks(4).any(|p| [p[0], p[1], p[2]] != first));
    }

    #[test]
    fn oil_filter_preserves_size_and_opacity() {
        let src = render_rgba(64, 64, 9, false);
        let out = oil_filter(&src, 64, 64, 3);
        assert_eq!(out.len(), src.len());
        assert!(out.iter().skip(3).step_by(4).all(|&a| a == 255));
    }
}
