use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use spectrogram::transform::fftw::{fourier_transform, planned_fourier_transform};

use fftw::array::AlignedVec;
use fftw::plan::{R2CPlan, R2CPlan64};
use fftw::types::Flag;

//pub fn criterion_benchmark(c: &mut Criterion) {
//    c.bench_function("fft", |b| {
//        b.iter(|| fourier_transform(black_box(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])))
//    });
//}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fft", |b| {
        b.iter(|| {
            let n = 8;
            let mut plan: R2CPlan64 =
                R2CPlan::aligned(&[n], Flag::MEASURE).expect("plan to create");
            let mut a = AlignedVec::new(n);
            let mut b = AlignedVec::new(n / 2 + 1);
            for _ in 0..100 {
                //let mut plan: R2CPlan64 = R2CPlan::aligned(&[n], Flag::MEASURE).expect("plan to create");
                a.copy_from_slice(&vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
                plan.r2c(&mut a, &mut b).expect("fftw dft to execute");
                let m: Vec<f64> = b.iter().map(|x| x.norm()).collect();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
