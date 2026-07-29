use bevy::prelude::*;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

// Import the necessary elements from your game
use StellarHash::camera::CameraPrincipale;
use StellarHash::univers::{Etoile, SecteursCharges, garbage_collector_spatial};

/// Utility function that recreates a test environment
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
                // Measured step: Launch 1 frame.
                // The engine will detect the 10,000 distant entities, destroy them, and filter the HashSet.
                app.update();
            },
            // Tells Criterion that the preparation requires a lot of RAM.
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, bench_garbage_collector);
criterion_main!(benches);
