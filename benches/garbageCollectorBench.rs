use bevy::prelude::*;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

// On importe les éléments nécessaires de ton jeu
use StellarHash::camera::CameraPrincipale;
use StellarHash::univers::{Etoile, SecteursCharges, garbage_collector_spatial};

/// Fonction utilitaire qui recrée un univers de test
fn preparer_monde_surcharge() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<SecteursCharges>();

    app.world_mut()
        .spawn((Transform::from_xyz(5000.0, 0.0, 0.0), CameraPrincipale));

    for x in -50..50 {
        for y in -50..50 {
            app.world_mut().spawn(Etoile {
                grille_x: x,
                grille_y: y,
            });
            app.world_mut()
                .resource_mut::<SecteursCharges>()
                .0
                .insert((x, y));
        }
    }

    app.add_systems(Update, garbage_collector_spatial);

    app
}

fn bench_garbage_collector(c: &mut Criterion) {
    c.bench_function("garbage_collector_10000_etoiles", |b| {
        b.iter_batched_ref(
            || preparer_monde_surcharge(),
            |app| {
                // Étape mesurée : On lance 1 frame.
                // Le moteur va détecter les 10 000 entités lointaines, les détruire, et filtrer le HashSet.
                app.update();
            },
            // Indique à Criterion que la préparation prend beaucoup de RAM
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, bench_garbage_collector);
criterion_main!(benches);
