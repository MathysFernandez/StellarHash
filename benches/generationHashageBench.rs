use criterion::{Criterion, black_box, criterion_group, criterion_main};

use StellarHash::generation::calculer_hachage_spatial;

fn bench_hachage_spatial(c: &mut Criterion) {
    let graine = 9999;

    // Une seule coordonnée statique (Temps brut de l'algorithme)
    c.bench_function("hachage_spatial_unique", |b| {
        b.iter(|| calculer_hachage_spatial(black_box(42), black_box(-84), black_box(graine)))
    });

    // Une grille de 10x10 (Simulation réaliste avec effet de cache processeur)
    c.bench_function("hachage_spatial_grille_10x10", |b| {
        b.iter(|| {
            // On itère sur 100 cases (10 * 10)
            for x in 0..10 {
                for y in 0..10 {
                    black_box(calculer_hachage_spatial(
                        black_box(x),
                        black_box(y),
                        black_box(graine),
                    ));
                }
            }
        })
    });
}

// Enregistrement et lancement du benchmark
criterion_group!(benches, bench_hachage_spatial);
criterion_main!(benches);
