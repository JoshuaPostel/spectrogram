use fftw::array::AlignedVec;
use fftw::plan::{R2CPlan, R2CPlan64};
use fftw::types::Flag;
use num::Complex;

pub fn fourier_transform(samples: &Vec<f64>) -> Vec<Complex<f64>> {
    let n = samples.len();
    let mut plan: R2CPlan64 = R2CPlan::aligned(&[n], Flag::MEASURE).expect("plan to create");
    let mut a = AlignedVec::new(n);
    let mut b = AlignedVec::new(n / 2 + 1);
    a.copy_from_slice(samples);
    plan.r2c(&mut a, &mut b).expect("fftw dft to execute");
    b.to_vec()
}

pub fn planned_fourier_transform(mut plan: R2CPlan64, samples: &Vec<f64>) -> Vec<Complex<f64>> {
    let n = samples.len();
    let mut a = AlignedVec::new(n);
    let mut b = AlignedVec::new(n / 2 + 1);
    a.copy_from_slice(samples);
    plan.r2c(&mut a, &mut b).expect("fftw dft to execute");
    b.to_vec()
}
