use criterion::{criterion_group, criterion_main, Criterion};
use opencv::prelude::Mat;
use opencv::core::Vector;

fn benchmark(c: &mut Criterion) {
    let mut vector = Vector::<i32>::with_capacity(1000);
    for x in 0..1000 {
        vector.push(rand::random());  
    }
    let mat = unsafe { Mat::new_nd_vec(&vector, 1) };

        
}

fn cpu_ascii_calculation(frame: &Mat, chars: &String, step: u32, grayscale: bool) {
    
}

fn gpu_ascii_calculation() {

}

criterion_group!(benches, benchmark);
criterion_main!(benches);
