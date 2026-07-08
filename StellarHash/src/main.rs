use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::utils::HashSet;

mod generation;


#[derive(Resource)]
struct GraineGlobale(u32);

#[derive(Resource, Default)]
struct SecteursCharges(HashSet<(i32, i32)>);

#[derive(Component)]
struct CameraPrincipale;

#[derive(Component)]
struct Etoile;


fn main() {
    App::new()
        // Ajoute les fonctionnalités de base de Bevy (Fenêtre, Rendu, Input, etc.)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "StellarHash".to_string(),
                
                // mode plein écran
                mode: bevy::window::WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))

        // On injecte notre graine globale au démarrage
        // (pour test: 42)
        .insert_resource(GraineGlobale(42))
        .init_resource::<SecteursCharges>()
        .add_systems(Startup, initialiser_camera)
        .add_systems(Update, (
            quitter_jeu,
            deplacer_camera,
            zoomer_camera,
            generer_univers_dynamique,
        ))
        .run();
}


// Système d'initialisation: Fait apparaître la caméra 2D
fn initialiser_camera(mut commands: Commands) {
    // Commands permet d'instancier (spawn) des entités dans notre monde 
    commands.spawn((
        Camera2dBundle::default(),
        CameraPrincipale,
    ));
    println!("Caméra 2D initialisée");
}

fn quitter_jeu(touches: Res<ButtonInput<KeyCode>>, mut evenements_sortie: EventWriter<bevy::app::AppExit>) {
    if touches.just_pressed(KeyCode::Escape) {
        evenements_sortie.send(bevy::app::AppExit::Success);
    }
}

fn deplacer_camera(
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut requete_camera: Query<&mut Transform, With<CameraPrincipale>>,
) {
    let mut transform = requete_camera.single_mut();
    let vitesse = 500.0 * transform.scale.x; 

    let mut direction = Vec3::ZERO;
    if touches.pressed(KeyCode::ArrowLeft) || touches.pressed(KeyCode::KeyQ) { direction.x -= 1.0; }
    if touches.pressed(KeyCode::ArrowRight) || touches.pressed(KeyCode::KeyD) { direction.x += 1.0; }
    if touches.pressed(KeyCode::ArrowUp) || touches.pressed(KeyCode::KeyZ) { direction.y += 1.0; }
    if touches.pressed(KeyCode::ArrowDown) || touches.pressed(KeyCode::KeyS) { direction.y -= 1.0; }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * vitesse * temps.delta_seconds();
    }
}

fn zoomer_camera(
    mut evenements_molette: EventReader<MouseWheel>,
    mut requete_camera: Query<&mut Transform, With<CameraPrincipale>>,
) {
    let mut transform = requete_camera.single_mut();
    for evenement in evenements_molette.read() {
        let facteur_zoom = 1.1;
        let mut nouvelle_echelle = transform.scale.x;

        if evenement.y > 0.0 {
            nouvelle_echelle /= facteur_zoom;
        } else if evenement.y < 0.0 {
            nouvelle_echelle *= facteur_zoom;
        }

        nouvelle_echelle = nouvelle_echelle.clamp(0.1, 50.0);
        transform.scale = Vec3::splat(nouvelle_echelle);
    }
}

fn generer_univers_dynamique(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<CameraPrincipale>>,
    graine: Res<GraineGlobale>,
    mut secteurs_charges: ResMut<SecteursCharges>,
) {
    let camera_transform = requete_camera.single();
    let cam_x = camera_transform.translation.x;
    let cam_y = camera_transform.translation.y;
    let zoom = camera_transform.scale.x;

    let taille_secteur = 50.0;
    
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100); 

    let centre_grille_x = (cam_x / taille_secteur).round() as i32;
    let centre_grille_y = (cam_y / taille_secteur).round() as i32;

    for x in (centre_grille_x - rayon_vision)..=(centre_grille_x + rayon_vision) {
        for y in (centre_grille_y - rayon_vision)..=(centre_grille_y + rayon_vision) {
            
            if secteurs_charges.0.contains(&(x, y)) {
                continue;
            }

            secteurs_charges.0.insert((x, y));

            // Appel au fichier generation.rs
            let probabilite = generation::calculer_hachage_spatial(x, y, graine.0);

            if probabilite > 0.95 {
                let couleur_etoile = if probabilite > 0.99 {
                    Color::srgb(0.5, 0.7, 1.0)
                } else if probabilite > 0.97 {
                    Color::srgb(1.0, 1.0, 0.6)
                } else {
                    Color::srgb(1.0, 0.4, 0.3)
                };

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: couleur_etoile,
                            custom_size: Some(Vec2::new(15.0, 15.0)), 
                            ..default()
                        },
                        transform: Transform::from_xyz(x as f32 * taille_secteur, y as f32 * taille_secteur, 0.0),
                        ..default()
                    },
                    Etoile,
                ));
            }
        }
    }
}
