use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use criterion::{Criterion, criterion_group, criterion_main};

use StellarHash::astrophysique::{ClasseSpectrale, SystemeStellaire};
use StellarHash::camera::CameraPrincipale;
use StellarHash::ui::{PanneauInfo, TexteInfo, gerer_survol_souris};
use StellarHash::univers::Etoile;

/// Fonction pour préparer un faux jeu contenant N étoiles
fn preparer_app_survol(nb_etoiles: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Fausse Fenêtre
    let mut fenetre = Window::default();
    fenetre.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
    app.world_mut().spawn((fenetre, PrimaryWindow));

    // Fausse Caméra
    app.world_mut()
        .spawn((Camera2dBundle::default(), CameraPrincipale));

    // Fausse UI
    app.world_mut().spawn((NodeBundle::default(), PanneauInfo));
    app.world_mut().spawn((
        TextBundle::from_section("Vide", TextStyle::default()),
        TexteInfo,
    ));

    // Génération des étoiles
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
    // Test avec 1 000 étoiles à l'écran
    c.bench_function("survol_souris_1000_etoiles", |b| {
        let mut app = preparer_app_survol(1000);
        b.iter(|| app.update());
    });

    // Test extreme avec 50 000 étoiles
    c.bench_function("survol_souris_50000_etoiles", |b| {
        let mut app = preparer_app_survol(50000);
        b.iter(|| app.update());
    });
}

criterion_group!(benches, bench_survol_souris);
criterion_main!(benches);
