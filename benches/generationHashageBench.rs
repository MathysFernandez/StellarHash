use criterion::{black_box, criterion_group, criterion_main, Criterion};

use StellarHash::generation::calculer_hachage_spatial;

fn bench_hachage_spatial(c: &mut Criterion) {
    // "hachage_spatial_unique" 

    c.bench_function("hachage_spatial", |b| {
        // b.iter exécute la fonction en boucle des milliers de fois 
        // pour obtenir une moyenne fiable en nanosecondes
        b.iter(|| {
            calculer_hachage_spatial(
                black_box(42),    // Coordonnée X
                black_box(-84),   // Coordonnée Y
                black_box(9999)   // Graine globale
            )
        })
    });
}

// Enregistrement et lancement du benchmark
criterion_group!(benches, bench_hachage_spatial);
criterion_main!(benches);