use bevy::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

// Import the function and the component from your game
use StellarHash::univers::{Planete, animer_orbites};

fn preparer_app_orbites(nb_planetes: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut time: Time = Time::default();
    time.advance_by(Duration::from_secs_f32(0.016));
    app.insert_resource(time);

    for i in 0..nb_planetes {
        app.world_mut().spawn((
            Transform::default(),
            Planete {
                rayon_orbite: 50.0 + (i as f32 % 10.0),
                angle_actuel: 0.0,
                vitesse_orbite: 1.5,
            },
        ));
    }

    app.add_systems(Update, animer_orbites);

    app.update();

    app
}

fn bench_animer_orbites(c: &mut Criterion) {
    // 1 000 planets displayed
    c.bench_function("animate_orbits_1000_planets", |b| {
        let mut app = preparer_app_orbites(1_000);
        b.iter(|| app.update());
    });

    // 100 000 planets
    c.bench_function("animate_orbits_100000_planets", |b| {
        let mut app = preparer_app_orbites(100_000);
        b.iter(|| app.update());
    });
}

criterion_group!(benches, bench_animer_orbites);
criterion_main!(benches);
