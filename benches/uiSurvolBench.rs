use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use criterion::{Criterion, criterion_group, criterion_main};

use StellarHash::astrophysique::{ClasseSpectrale, SystemeStellaire};
use StellarHash::camera::CameraPrincipale;
use StellarHash::ui::{PanneauInfo, TexteInfo, gerer_survol_souris};
use StellarHash::univers::Etoile;

/// Function to prepare a dummy game containing N stars
fn preparer_app_survol(nb_etoiles: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Faux Window
    let mut fenetre = Window::default();
    fenetre.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
    app.world_mut().spawn((fenetre, PrimaryWindow));

    // Dummy Camera
    app.world_mut()
        .spawn((Camera2dBundle::default(), CameraPrincipale));

    // Mock UI
    app.world_mut().spawn((NodeBundle::default(), PanneauInfo));
    app.world_mut().spawn((
        TextBundle::from_section("Empty", TextStyle::default()),
        TexteInfo,
    ));

    // Star generation
    for i in 0..nb_etoiles {
        let x = (i as f32) * 80.0;
        app.world_mut().spawn((
            Transform::from_xyz(x, 0.0, 0.0),
            Etoile {
                grille_x: i,
                grille_y: 0,
            },
            SystemeStellaire {
                nom: "Test".to_string(),
                classe: ClasseSpectrale::G,
                masse_solaire: 1.0,
                rayon_solaire: 1.0,
                nb_planetes: 1,
                age_milliards_annees: 4.5,
            },
        ));
    }

    app.add_systems(Update, gerer_survol_souris);

    app.update();

    app
}

fn bench_survol_souris(c: &mut Criterion) {
    // Test with 1,000 stars on screen
    c.bench_function("mouse_hover_1000_stars", |b| {
        let mut app = preparer_app_survol(1000);
        b.iter(|| app.update());
    });

    // Extreme test with 50,000 stars
    c.bench_function("mouse_hover_50000_stars", |b| {
        let mut app = preparer_app_survol(50000);
        b.iter(|| app.update());
    });
}

criterion_group!(benches, bench_survol_souris);
criterion_main!(benches);
