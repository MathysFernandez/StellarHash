use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::f32::consts::PI;

use StellarHash::astrophysique::generer_caracteristiques;

fn bench_generer_caracteristiques(c: &mut Criterion) {
    // A single call with a fixed probability (Class M, very common)
    c.bench_function("generer_carac_unique_classe_m", |b| {
        b.iter(|| {
            generer_caracteristiques(
                black_box(42),
                black_box(-84),
                // Probability of 0.5 = Class M
                black_box(0.5),
            )
        })
    });

    // A single call with a fixed probability (Class O, ultra-rare)
    // This makes it possible to see if the path taken in the `if/else` affects performance.
    c.bench_function("generer_carac_unique_classe_o", |b| {
        b.iter(|| {
            generer_caracteristiques(
                black_box(42),
                black_box(-84),
                // Probability of 0.999 = Class O
                black_box(0.999),
            )
        })
    });

    // Realistic simulation
    // Generate 100 features with different probabilities
    // to force the CPU to take different branches (branch prediction)
    c.bench_function("generer_carac_lot_100", |b| {
        // Prepare an array of 100 probabilities
        let probabilites: Vec<f32> = (0..100)
            .map(|i| {
                let pseudo_alea = (i as f32 * PI).sin().abs();
                0.95 + (pseudo_alea * 0.05)
            })
            .collect();

        b.iter(|| {
            for (i, &prob) in probabilites.iter().enumerate() {
                black_box(generer_caracteristiques(
                    black_box(i as i32),
                    black_box(i as i32 * 2),
                    black_box(prob),
                ));
            }
        })
    });
}

criterion_group!(benches, bench_generer_caracteristiques);
criterion_main!(benches);
