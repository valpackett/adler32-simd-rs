#[macro_use]
extern crate bencher;
extern crate adler32;
extern crate rand;

use adler32::RollingAdler32;
use bencher::Bencher;
use rand::{thread_rng, RngCore};

fn bench(b: &mut Bencher, size: usize, adler: &mut RollingAdler32) {
    let mut in_bytes = vec![0u8; size];
    thread_rng().fill_bytes(&mut in_bytes);

    b.iter(|| {
        adler.update_buffer(&in_bytes);
        bencher::black_box(adler.hash())
    });
    b.bytes = size as u64;
}

fn bench_baseline(b: &mut Bencher, size: usize) {
    let mut adl = RollingAdler32::new();
    adl.force_no_acceleration();
    bench(b, size, &mut adl)
}

fn bench_accel(b: &mut Bencher, size: usize) {
    bench(b, size, &mut RollingAdler32::new())
}

fn bench_512b_baseline(b: &mut Bencher) {
    bench_baseline(b, 512)
}
fn bench_512b_accel(b: &mut Bencher) {
    bench_accel(b, 512)
}
fn bench_100kb_baseline(b: &mut Bencher) {
    bench_baseline(b, 1024 * 100)
}
fn bench_100kb_accel(b: &mut Bencher) {
    bench_accel(b, 1024 * 100)
}

benchmark_group!(
    bench_default,
    bench_512b_baseline,
    bench_512b_accel,
    bench_100kb_baseline,
    bench_100kb_accel
);

benchmark_main!(bench_default);
