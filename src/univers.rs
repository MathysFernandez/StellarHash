use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::generation;

pub struct UniversPlugin;

impl Plugin for UniversPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalSeed(42))
            .init_resource::<LoadedSectors>()
            .add_systems(
                Update,
                (
                    generate_dynamic_universe,
                    spatial_garbage_collector,
                    handle_star_click,
                    animate_orbits,
                    handle_planet_lod,
                    animate_star_twinkle,
                ),
            );
    }
}

#[derive(Resource)]
pub struct GlobalSeed(pub u32);

#[derive(Resource, Default)]
pub struct LoadedSectors(pub HashSet<(i32, i32)>);

#[derive(Component)]
pub struct Star {
    pub grille_x: i32,
    pub grille_y: i32,
}

#[derive(Component)]
pub struct ExpandedSystem;

#[derive(Component)]
pub struct Planet {
    pub rayon_orbite: f32,
    pub angle_actuel: f32,
    pub vitesse_orbite: f32,
}

fn generate_dynamic_universe(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<MainCamera>>,
    graine: Res<GlobalSeed>,
    mut secteurs_charges: ResMut<LoadedSectors>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut derniere_pos_maj: Local<Vec2>,
    mut dernier_zoom_maj: Local<f32>,
) {
    let camera_transform = requete_camera.single();
    let pos_actuelle = camera_transform.translation.truncate();
    let zoom = camera_transform.scale.x;

    // Optimization: Only regenerate if the camera moves
    if pos_actuelle.distance(*derniere_pos_maj) < 40.0 && (zoom - *dernier_zoom_maj).abs() < 0.1 {
        return;
    }

    *derniere_pos_maj = pos_actuelle;
    *dernier_zoom_maj = zoom;

    let taille_secteur = 80.0;
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);

    let centre_grille_x = (pos_actuelle.x / taille_secteur).round() as i32;
    let centre_grille_y = (pos_actuelle.y / taille_secteur).round() as i32;

    for x in (centre_grille_x - rayon_vision)..=(centre_grille_x + rayon_vision) {
        for y in (centre_grille_y - rayon_vision)..=(centre_grille_y + rayon_vision) {
            if secteurs_charges.0.contains(&(x, y)) {
                continue;
            }

            secteurs_charges.0.insert((x, y));

            let probabilite = generation::calculate_spatial_hash(x, y, graine.0);

            // Generation threshold increased to 0.95 for performance and clarity
            if probabilite > 0.95 {
                let systeme_stellaire =
                    crate::astrophysique::generate_characteristics(x, y, probabilite);

                let couleur_etoile = match systeme_stellaire.classe {
                    crate::astrophysique::SpectralClass::O => Color::srgb(0.3, 0.5, 1.0),
                    crate::astrophysique::SpectralClass::B => Color::srgb(0.6, 0.8, 1.0),
                    crate::astrophysique::SpectralClass::A => Color::srgb(1.0, 1.0, 1.0),
                    crate::astrophysique::SpectralClass::F => Color::srgb(1.0, 1.0, 0.8),
                    crate::astrophysique::SpectralClass::G => Color::srgb(1.0, 0.9, 0.2),
                    crate::astrophysique::SpectralClass::K => Color::srgb(1.0, 0.5, 0.1),
                    crate::astrophysique::SpectralClass::M => Color::srgb(0.9, 0.2, 0.2),
                };

                let taille_visuelle = 8.0 + (systeme_stellaire.rayon_solaire * 4.0);

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Circle::new(taille_visuelle / 2.0))),
                        material: materials.add(ColorMaterial::from(couleur_etoile)),
                        transform: Transform::from_xyz(
                            x as f32 * taille_secteur,
                            y as f32 * taille_secteur,
                            0.0,
                        ),
                        ..default()
                    },
                    Star {
                        grille_x: x,
                        grille_y: y,
                    },
                    systeme_stellaire,
                ));
            }
        }
    }
}

pub fn animate_star_twinkle(
    temps: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut requete_etoiles: Query<(
        &Star,
        &crate::astrophysique::StellarSystem,
        &Handle<ColorMaterial>,
        &mut Transform,
    )>,
) {
    let temps_ecoule = temps.elapsed_seconds();

    for (etoile, systeme, handle_materiau, mut transform) in requete_etoiles.iter_mut() {
        let (r, g, b) = match systeme.classe {
            crate::astrophysique::SpectralClass::O => (0.3, 0.5, 1.0),
            crate::astrophysique::SpectralClass::B => (0.6, 0.8, 1.0),
            crate::astrophysique::SpectralClass::A => (1.0, 1.0, 1.0),
            crate::astrophysique::SpectralClass::F => (1.0, 1.0, 0.8),
            crate::astrophysique::SpectralClass::G => (1.0, 0.9, 0.2),
            crate::astrophysique::SpectralClass::K => (1.0, 0.5, 0.1),
            crate::astrophysique::SpectralClass::M => (0.9, 0.2, 0.2),
        };

        let dephasage = (etoile.grille_x as f32 * 0.7) + (etoile.grille_y as f32 * 0.3);
        let vitesse = 2.0;

        let onde_sinus = (temps_ecoule * vitesse + dephasage).sin();

        // min: 10.0
        // max: 20.0
        let intensite = 15.0 + (onde_sinus * 5.0);

        let echelle = 1.0 + (onde_sinus * 0.01);

        if let Some(materiau) = materials.get_mut(handle_materiau) {
            materiau.color = Color::srgb(r * intensite, g * intensite, b * intensite);
        }

        transform.scale = Vec3::splat(echelle);
    }
}

pub fn spatial_garbage_collector(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<MainCamera>>,
    requete_etoiles: Query<(Entity, &Star)>,
    mut secteurs_charges: ResMut<LoadedSectors>,
    mut derniere_pos_maj: Local<Vec2>,
    mut dernier_zoom_maj: Local<f32>,
) {
    let camera_transform = requete_camera.single();
    let pos_actuelle = camera_transform.translation.truncate();
    let zoom = camera_transform.scale.x;

    if pos_actuelle.distance(*derniere_pos_maj) < 40.0 && (zoom - *dernier_zoom_maj).abs() < 0.1 {
        return;
    }

    *derniere_pos_maj = pos_actuelle;
    *dernier_zoom_maj = zoom;

    let taille_secteur = 80.0;
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);
    let rayon_despawn = rayon_vision + 5;

    let centre_grille_x = (pos_actuelle.x / taille_secteur).round() as i32;
    let centre_grille_y = (pos_actuelle.y / taille_secteur).round() as i32;

    for (entite, etoile) in requete_etoiles.iter() {
        if (etoile.grille_x - centre_grille_x).abs() > rayon_despawn
            || (etoile.grille_y - centre_grille_y).abs() > rayon_despawn
        {
            commands.entity(entite).despawn_recursive();
        }
    }

    secteurs_charges.0.retain(|&(x, y)| {
        (x - centre_grille_x).abs() <= rayon_despawn && (y - centre_grille_y).abs() <= rayon_despawn
    });
}

fn handle_planet_lod(
    requete_camera: Query<&Transform, (With<MainCamera>, Changed<Transform>)>,
    mut requete_planetes: Query<&mut Visibility, With<Planet>>,
) {
    if let Ok(camera_transform) = requete_camera.get_single() {
        let zoom = camera_transform.scale.x;
        let seuil_lod = 3.5;
        let visibilite_voulue = if zoom > seuil_lod {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };

        for mut visibilite in requete_planetes.iter_mut() {
            if *visibilite != visibilite_voulue {
                *visibilite = visibilite_voulue;
            }
        }
    }
}

fn handle_star_click(
    mut commands: Commands,
    touches_souris: Res<ButtonInput<MouseButton>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    requete_etoiles: Query<(
        Entity,
        &Transform,
        &crate::astrophysique::StellarSystem,
        Option<&ExpandedSystem>,
        &Star,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !touches_souris.just_pressed(MouseButton::Left) {
        return;
    }

    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    if let Some(position_curseur) = fenetre.cursor_position() {
        if let Some(position_monde) =
            camera.viewport_to_world_2d(camera_transform, position_curseur)
        {
            let taille_secteur = 80.0;
            let clic_grille_x = (position_monde.x / taille_secteur).round() as i32;
            let clic_grille_y = (position_monde.y / taille_secteur).round() as i32;

            for (entite, transform_etoile, systeme, developpe, etoile) in requete_etoiles.iter() {
                if (etoile.grille_x - clic_grille_x).abs() > 1
                    || (etoile.grille_y - clic_grille_y).abs() > 1
                {
                    continue;
                }

                if position_monde.distance(transform_etoile.translation.truncate()) < 25.0 {
                    if developpe.is_none() {
                        commands
                            .entity(entite)
                            .insert(ExpandedSystem)
                            .with_children(|parent| {
                                for i in 0..systeme.nb_planetes {
                                    let rayon_orbite = 15.0 + (i as f32 * 10.0);
                                    let angle_depart = (i as f32) * 1.2;
                                    let vitesse = 1.5 / (i as f32 + 1.0);

                                    parent.spawn((
                                        MaterialMesh2dBundle {
                                            mesh: Mesh2dHandle(meshes.add(Circle::new(2.0))),
                                            material: materials.add(ColorMaterial::from(
                                                Color::srgb(0.6, 0.8, 0.9),
                                            )),
                                            transform: Transform::from_xyz(
                                                rayon_orbite * angle_depart.cos(),
                                                rayon_orbite * angle_depart.sin(),
                                                1.0,
                                            ),
                                            ..default()
                                        },
                                        Planet {
                                            rayon_orbite,
                                            angle_actuel: angle_depart,
                                            vitesse_orbite: vitesse,
                                        },
                                    ));
                                }
                            });
                    } else {
                        commands.entity(entite).remove::<ExpandedSystem>();
                        commands.entity(entite).despawn_descendants();
                    }
                    break;
                }
            }
        }
    }
}

pub fn animate_orbits(
    temps: Res<Time>,
    mut requete_planetes: Query<(&mut Transform, &mut Planet)>,
) {
    for (mut transform, mut planete) in requete_planetes.iter_mut() {
        planete.angle_actuel += planete.vitesse_orbite * temps.delta_seconds();
        transform.translation.x = planete.rayon_orbite * planete.angle_actuel.cos();
        transform.translation.y = planete.rayon_orbite * planete.angle_actuel.sin();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrophysique::{SpectralClass, StellarSystem};
    use bevy::render::camera::CameraProjection;
    use bevy::window::PrimaryWindow;
    use std::time::Duration;

    // --- Utility function to prepare the environment ---
    fn preparer_app() -> App {
        let mut app = App::new();

        // Assets are required for the generated MaterialMesh2dBundles
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));

        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();

        // Initialization of resources required by the system
        app.insert_resource(GlobalSeed(42));

        // MANDATORY: Initialize the HashSet to avoid a "Resource does not exist" crash.
        app.init_resource::<LoadedSectors>();

        // Creating our main camera at the center (0, 0)
        app.world_mut()
            .spawn((Camera2dBundle::default(), MainCamera));

        app
    }

    // --- Utility function to prepare the environment ---
    fn preparer_app_gc() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // The LoadedSectors resource must exist
        app.init_resource::<LoadedSectors>();

        // Creating our camera (defaulting to 0, 0, with a scale/zoom of 1.0)
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), MainCamera));

        app
    }

    // --- Utility function to prepare the environment ---
    fn preparer_app_clic() -> App {
        let mut app = App::new();

        // Assets are required to display the planets (MaterialMesh2dBundle)
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();

        // Initialize mouse input
        app.init_resource::<ButtonInput<MouseButton>>();

        // Simulate a window with the mouse at the center (400, 300)
        let mut fenetre = Window::default();
        fenetre.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
        app.world_mut().spawn((fenetre, PrimaryWindow));

        // Simulate the camera centered at (0, 0)
        let mut camera_bundle = Camera2dBundle::default();
        camera_bundle.projection.update(800.0, 600.0);
        app.world_mut().spawn((camera_bundle, MainCamera));

        app
    }

    // --- Utility function to prepare the environment ---
    fn preparer_app_lod() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    // --- Initialization and loading of the first sector ---
    #[test]
    fn test_generer_univers_charge_les_premiers_secteurs() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);

        // Initially, the HashSet of loaded sectors must be empty
        assert!(app.world().resource::<LoadedSectors>().0.is_empty());

        // Execute the system once (the camera is at 0,0)
        app.update();

        // The system must have marked numerous sectors (around 0.0) as "loaded"
        let secteurs = app.world().resource::<LoadedSectors>();
        assert!(
            !secteurs.0.is_empty(),
            "The sectors around the camera should have been generated."
        );

        // Check if Star entities have been created in the universe
        let mut requete_etoiles = app.world_mut().query::<&Star>();
        let nombre_etoiles = requete_etoiles.iter(app.world()).count();

        assert!(
            nombre_etoiles > 0,
            "At least one star should have appeared with this seed."
        );
    }

    // --- Optimization (Ignore small movements) ---
    #[test]
    fn test_generer_univers_ignore_les_micro_mouvements() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);

        // 1st frame: Initialization (position (0,0) is stored in `Local`)
        app.update();

        // Store the exact number of currently loaded sectors
        let secteurs_avant = app.world().resource::<LoadedSectors>().0.len();

        // On simule un tout petit mouvement de la caméra (distance < 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 15.0;

        // 2nd frame
        app.update();

        // The optimization (the `return` on line 61) must have worked.
        // No new sector must have been calculated.
        let secteurs_apres = app.world().resource::<LoadedSectors>().0.len();
        assert_eq!(
            secteurs_avant, secteurs_apres,
            "The function should have returned early without loading new sectors."
        );
    }

    // --- Generation after a large displacement ---
    #[test]
    fn test_generer_univers_reprend_apres_grand_deplacement() {
        let mut app = preparer_app();
        app.add_systems(Update, generate_dynamic_universe);

        // 1st frame (initialization)
        app.update();
        let secteurs_avant = app.world().resource::<LoadedSectors>().0.len();

        // Teleport the camera very far away (distance > 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 1000.0;
        transform.translation.y += 1000.0;

        // 2nd frame
        app.update();

        // The 40.0 threshold having been exceeded, the function had to calculate the new spatial zone.
        let secteurs_apres = app.world().resource::<LoadedSectors>().0.len();
        assert!(
            secteurs_apres > secteurs_avant,
            "The camera teleported 1,000 units; new sectors should have been loaded."
        );
    }

    // --- Effective cleaning ---
    #[test]
    fn test_garbage_collector_nettoie_hors_limites() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, spatial_garbage_collector);

        // Populate the universe with two stars and two corresponding sectors
        let entite_proche = app
            .world_mut()
            .spawn(Star {
                grille_x: 2,
                grille_y: 2,
            })
            .id();

        // A very distant star (grid 50, i.e., a distance of 4000 pixels)
        let entite_lointaine = app
            .world_mut()
            .spawn(Star {
                grille_x: 50,
                grille_y: 50,
            })
            .id();

        let mut secteurs = app.world_mut().resource_mut::<LoadedSectors>();
        secteurs.0.insert((2, 2));
        secteurs.0.insert((50, 50));

        // To force the Garbage Collector to run, move the camera by at least 40 units.
        // Otherwise, the Local<Vec2> variable initialized to (0,0) will cause a premature return.
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 100.0; // Grid center recalculated to approximately (1, 0)

        app.update();

        // --- VÉRIFICATIONS ---

        // The nearby star must have survived
        assert!(app.world().get_entity(entite_proche).is_some());

        // The distant star must have been disintegrated (despawn_recursive) by the system.
        assert!(app.world().get_entity(entite_lointaine).is_none());

        // The HashSet of loaded sectors had to be purged of the distant coordinate.
        let secteurs_apres = app.world().resource::<LoadedSectors>();
        assert!(secteurs_apres.0.contains(&(2, 2)));
        assert!(
            !secteurs_apres.0.contains(&(50, 50)),
            "The distant sector should have been removed from memory."
        );
    }

    // --- Local optimization ---
    #[test]
    fn test_garbage_collector_ignore_les_micro_mouvements() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, spatial_garbage_collector);

        // Frame 1: The camera is placed very far away to initialize the local variables `derniere_pos_maj`
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 1000.0;
        app.update();

        // Create a star completely out of reach
        let entite_lointaine = app
            .world_mut()
            .spawn(Star {
                grille_x: 9999,
                grille_y: 9999,
            })
            .id();

        // Frame 2: Simulate a movement of 10 units (< 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<MainCamera>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x += 10.0;

        app.update();

        // Since the movement is too weak, the system had to execute a `return` at line 61.
        // The distant entity was therefore not cleaned up, proving that the optimization works.
        assert!(
            app.world().get_entity(entite_lointaine).is_some(),
            "Le GC n'aurait pas dû s'exécuter"
        );
    }

    // --- Masking during zooming out ---
    #[test]
    fn test_gerer_lod_cache_les_planetes_si_zoom_superieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        // Create a camera with a zoom (scale.x) greater than the threshold of 3.5
        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(4.0)), MainCamera));

        // Create a planet that is currently visible (Inherited)
        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Inherited,
            ))
            .id();

        app.update();

        // The system must have detected a zoom level > 3.5 and hidden the planet.
        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Hidden);
    }

    // --- Display when zooming ---
    #[test]
    fn test_gerer_lod_affiche_les_planetes_si_zoom_inferieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        // Create a camera with a very close zoom (below the 3.5 threshold)
        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(1.0)), MainCamera));

        // We create a planet that is currently hidden
        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Hidden,
            ))
            .id();

        app.update();

        // The system must have restored the planet's normal visibility (Inherited).
        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Inherited);
    }

    // --- Changed<Transform> Optimization  ---
    #[test]
    fn test_gerer_lod_ne_fait_rien_si_camera_immobile() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, handle_planet_lod);

        // Create the camera
        let entite_camera = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(2.0)), MainCamera))
            .id();

        let entite_planete = app
            .world_mut()
            .spawn((
                Planet {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                // Force it into an incorrect state relative to the zoom to test the filter
                Visibility::Hidden,
            ))
            .id();

        // First frame: The component has just been created; it counts as "Changed".
        // The system will therefore correct the visibility to Inherited.
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );

        // We manually force it to Hidden again
        *app.world_mut()
            .get_mut::<Visibility>(entite_planete)
            .unwrap() = Visibility::Hidden;

        // Second frame: The camera has not been modified!
        app.update();

        // Since the camera has not moved, the Changed<Transform> query returns an error.
        // The system did not execute, so the planet remained in its forced state (Hidden).
        let visibilite_finale = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(
            *visibilite_finale,
            Visibility::Hidden,
            "Le système a ignoré le filtre Changed<Transform>"
        );

        // Third frame: Move the camera
        app.world_mut()
            .get_mut::<Transform>(entite_camera)
            .unwrap()
            .scale
            .x = 2.1;
        app.update();

        // The system woke up and corrected the visibility.
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );
    }

    // --- Ignore if no click is detected ---
    #[test]
    fn test_gerer_clic_ignore_si_pas_de_clic() {
        let mut app = preparer_app_clic();
        app.add_systems(Update, handle_star_click);

        // A star appears in the center.
        let entite_etoile = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                StellarSystem {
                    nom: "Test-Beta".to_string(),
                    classe: SpectralClass::G,
                    masse_solaire: 1.0,
                    rayon_solaire: 1.0,
                    nb_planetes: 2,
                    age_milliards_annees: 4.0,
                },
                Star {
                    grille_x: 0,
                    grille_y: 0,
                },
            ))
            .id();

        // We launch the system without simulating a mouse click.
        app.update();

        // The star must not have received the ExpandedSystem component.
        assert!(app.world().get::<ExpandedSystem>(entite_etoile).is_none());
    }

    #[test]
    fn test_animer_orbites_calcule_position_et_angle_correctement() {
        let mut app = App::new();

        // Initialization of simulated time
        // We explicitly declare the type `: Time` to help the compiler
        let mut time: Time = Time::default();

        // Advance time by exactly 0.5 seconds
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);

        // Creating our test planet
        let rayon = 100.0;
        // We define an orbital speed of PI radians per second (half a revolution per second)
        let vitesse = std::f32::consts::PI;

        let entite = app
            .world_mut()
            .spawn((
                // The starting position is irrelevant.
                // because your system overwrites the X and Y values ​​no matter what.
                Transform::default(),
                Planet {
                    rayon_orbite: rayon,
                    angle_actuel: 0.0,
                    vitesse_orbite: vitesse,
                },
            ))
            .id();

        // System execution
        app.add_systems(Update, animate_orbits);
        app.update();

        // Mathematical checks
        let planete = app.world().get::<Planet>(entite).unwrap();
        let transform = app.world().get::<Transform>(entite).unwrap();

        // The current angle had to increase according to the formula: speed * delta_seconds
        // PI rad/s * 0.5s = PI/2 rad (i.e., 90 degrees, the top of the circle)
        let angle_attendu = vitesse * 0.5;
        assert_eq!(planete.angle_actuel, angle_attendu);

        // At π/2 (90 degrees) on a circle with a radius of 100:
        // cos(PI/2) is approximately 0.0 -> The X position must be 0.0
        // sin(PI/2) is 1.0 -> The Y position must be 100.0
        let difference_x = (transform.translation.x - 0.0).abs();
        let difference_y = (transform.translation.y - 100.0).abs();

        // We always use a tolerance (Delta) for floating-point numbers
        assert!(
            difference_x < 0.0001,
            "The X position calculated using the cosine is incorrect."
        );
        assert!(
            difference_y < 0.0001,
            "The Y position calculated using the sine is incorrect."
        );
    }
}
