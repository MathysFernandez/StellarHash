use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::f32::consts::PI;

use StellarHash::astrophysique::generer_caracteristiques;

fn bench_generer_caracteristiques(c: &mut Criterion) {
    // Un seul appel avec une probabilité fixe (Classe M, très commune)
    c.bench_function("generer_carac_unique_classe_m", |b| {
        b.iter(|| {
            generer_caracteristiques(
                black_box(42),
                black_box(-84),
                // Probabilité de 0.5 = Classe M
                black_box(0.5),
            )
        })
    });

    // Un seul appel avec une probabilité fixe (Classe O, ultra rare)
    // Cela permet de voir si le chemin pris dans le `if/else` affecte les performances
    c.bench_function("generer_carac_unique_classe_o", |b| {
        b.iter(|| {
            generer_caracteristiques(
                black_box(42),
                black_box(-84),
                // Probabilité de 0.999 = Classe O
                black_box(0.999),
            )
        })
    });

    // Simulation réaliste
    // On génère 100 caractéristiques avec des probabilités différentes
    // pour forcer le CPU à prendre des branches différentes (branch prediction)
    c.bench_function("generer_carac_lot_100", |b| {
        // On prépare un tableau de 100 probabilités
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
