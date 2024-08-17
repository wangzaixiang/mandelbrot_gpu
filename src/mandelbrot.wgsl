@group(0) @binding(0)
var<storage, read_write> pixes: array<u32>;

@group(0) @binding(1)
var<uniform> bounds: vec2<u32>;  // width, height

@group(0) @binding(2)
var<uniform> upper_left: vec2<f32>;  // re, im

@group(0) @binding(3)
var<uniform> lower_right: vec2<f32>; // re, im

struct Complex {
    re: f32,
    im: f32,
}

fn addComplex(a: Complex, b: Complex) -> Complex {
    return Complex(a.re + b.re, a.im + b.im);
}

fn mulComplex(a: Complex, b: Complex) -> Complex {
    return Complex(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re);
}

// limit = 255
fn escapeTime(c: Complex) -> u32 {
    var z: Complex = Complex(0.0, 0.0);
    var n: u32 = 0;
    var norm: f32 = 0.0;
    while (n < 255u) {
        z = addComplex(mulComplex(z, z), c);
        norm = norm_sqr(z);
        if (norm > 4.0) {
            return n;
        }
        n = n + 1u;
    }
    return n;
}

fn norm_sqr(z: Complex) -> f32 {
    return z.re * z.re + z.im * z.im;
}

@compute
@workgroup_size(256)
fn main( @builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;

    let r: u32 = i / bounds.x;  // row
    let c: u32 = i % bounds.x;  // column
    let width = lower_right.x - upper_left.x;
    let height = upper_left.y - lower_right.y;

    let z = Complex(
        upper_left.x + f32(c) * width / f32(bounds.x),
        upper_left.y - f32(r) * height / f32(bounds.y)
    );
    pixes[i] = escapeTime(z);
}

