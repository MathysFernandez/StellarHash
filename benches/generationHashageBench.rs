use criterion::{Criterion, black_box, criterion_group, criterion_main};

use StellarHash::generation::calculate_spatial_hash;

fn bench_spatial_hash(c: &mut Criterion) {
    let graine = 9999;

    // A single static coordinate (Raw algorithm time)
    c.bench_function("unique_spatial_hashing", |b| {
        b.iter(|| calculate_spatial_hash(black_box(42), black_box(-84), black_box(graine)))
    });

    // A 10x10 grid (Realistic simulation with processor cache effect)
    c.bench_function("10x10_grid_spatial_hashing", |b| {
        b.iter(|| {
            // Iterate over 100 cells (10 * 10)
            for x in 0..10 {
                for y in 0..10 {
                    black_box(calculate_spatial_hash(
                        black_box(x),
                        black_box(y),
                        black_box(graine),
                    ));
                }
            }
        })
    });
}

// Saving and launching the benchmark
criterion_group!(benches, bench_hachage_spatial);
criterion_main!(benches);
