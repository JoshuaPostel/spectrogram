use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fftw::array::AlignedVec;
use fftw::plan::{R2CPlan, R2CPlan64};
use fftw::types::Flag;

#[allow(unused_imports)]
use spectrogram::transform::{naive, naive_simd};

pub fn criterion_benchmark(c: &mut Criterion) {
    // TODO
    // * figure out proper plan createion outside of benchmark time
    // * figure out a fair compairison
    c.bench_function("fft", |b| {
        b.iter(|| {
            let n = 8;
            let mut plan: R2CPlan64 =
                R2CPlan::aligned(&[n], Flag::MEASURE).expect("plan to create");
            let mut a = AlignedVec::new(n);
            let mut b = AlignedVec::new(n / 2 + 1);
            for _ in 0..100 {
                a.copy_from_slice(&vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
                plan.r2c(black_box(&mut a), &mut b)
                    .expect("fftw dft to execute");
                let _m: Vec<f64> = b.iter().map(|x| x.norm()).collect();
            }
        })
    });

    // the following were used to learn about SIMD
    // benchmarks of naive vs naive_simd implementations
    // simd results in ~17% speedup
    c.bench_function("vanilla_dft", |b| {
        b.iter(|| naive_simd::fourier_transform(black_box(vec![1, 0, 0, 0, 0, 0, 0, 0])))
    });

    // simd results in ~17% speedup
    c.bench_function("non_trivial_dft", |b| {
        b.iter(|| {
            naive_simd::fourier_transform(black_box(vec![100, 211, 62, 116, 34, 98, 178, 12]))
        })
    });

    // simd results in ~7% speedup
    c.bench_function("indivisible_by_eight_dft", |b| {
        b.iter(|| {
            naive_simd::fourier_transform(black_box(vec![
                100, 211, 62, 116, 34, 98, 178, 12, 201, 145, 178,
            ]))
        })
    });

    // simd results in ~23% speedup
    c.bench_function("large_dft", |b| {
        b.iter(|| naive_simd::fourier_transform(black_box((0..=7999).collect())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
