use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;

use crate::camera::CameraPrincipale;
use crate::generation;

pub struct UniversPlugin;

impl Plugin for UniversPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraineGlobale(42))
            .init_resource::<SecteursCharges>()
            .add_systems(
                Update,
                (
                    generer_univers_dynamique,
                    garbage_collector_spatial,
                    gerer_clic_etoile,
                    animer_orbites,
                    gerer_lod_planetes,
                ),
            );
    }
}

#[derive(Resource)]
pub struct GraineGlobale(pub u32);

#[derive(Resource, Default)]
pub struct SecteursCharges(pub HashSet<(i32, i32)>);

#[derive(Component)]
pub struct Etoile {
    pub grille_x: i32,
    pub grille_y: i32,
}

#[derive(Component)]
pub struct SystemeDeveloppe;

#[derive(Component)]
pub struct Planete {
    pub rayon_orbite: f32,
    pub angle_actuel: f32,
    pub vitesse_orbite: f32,
}

fn generer_univers_dynamique(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<CameraPrincipale>>,
    graine: Res<GraineGlobale>,
    mut secteurs_charges: ResMut<SecteursCharges>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut derniere_pos_maj: Local<Vec2>,
    mut dernier_zoom_maj: Local<f32>,
) {
    let camera_transform = requete_camera.single();
    let pos_actuelle = camera_transform.translation.truncate();
    let zoom = camera_transform.scale.x;

    // Optimisation : On ne régénère que si la caméra bouge
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

            let probabilite = generation::calculer_hachage_spatial(x, y, graine.0);

            // Seuil de génération augmenté à 0.95 pour la performance et la clarté
            if probabilite > 0.95 {
                let systeme_stellaire =
                    crate::astrophysique::generer_caracteristiques(x, y, probabilite);

                let couleur_etoile = match systeme_stellaire.classe {
                    crate::astrophysique::ClasseSpectrale::O => Color::srgb(0.3, 0.5, 1.0),
                    crate::astrophysique::ClasseSpectrale::B => Color::srgb(0.6, 0.8, 1.0),
                    crate::astrophysique::ClasseSpectrale::A => Color::srgb(1.0, 1.0, 1.0),
                    crate::astrophysique::ClasseSpectrale::F => Color::srgb(1.0, 1.0, 0.8),
                    crate::astrophysique::ClasseSpectrale::G => Color::srgb(1.0, 0.9, 0.2),
                    crate::astrophysique::ClasseSpectrale::K => Color::srgb(1.0, 0.5, 0.1),
                    crate::astrophysique::ClasseSpectrale::M => Color::srgb(0.9, 0.2, 0.2),
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
                    Etoile {
                        grille_x: x,
                        grille_y: y,
                    },
                    systeme_stellaire,
                ));
            }
        }
    }
}

fn garbage_collector_spatial(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<CameraPrincipale>>,
    requete_etoiles: Query<(Entity, &Etoile)>,
    mut secteurs_charges: ResMut<SecteursCharges>,
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

fn gerer_lod_planetes(
    requete_camera: Query<&Transform, (With<CameraPrincipale>, Changed<Transform>)>,
    mut requete_planetes: Query<&mut Visibility, With<Planete>>,
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

fn gerer_clic_etoile(
    mut commands: Commands,
    touches_souris: Res<ButtonInput<MouseButton>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<CameraPrincipale>>,
    requete_etoiles: Query<(
        Entity,
        &Transform,
        &crate::astrophysique::SystemeStellaire,
        Option<&SystemeDeveloppe>,
        &Etoile,
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
                            .insert(SystemeDeveloppe)
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
                                        Planete {
                                            rayon_orbite,
                                            angle_actuel: angle_depart,
                                            vitesse_orbite: vitesse,
                                        },
                                    ));
                                }
                            });
                    } else {
                        commands.entity(entite).remove::<SystemeDeveloppe>();
                        commands.entity(entite).despawn_descendants();
                    }
                    break;
                }
            }
        }
    }
}

fn animer_orbites(temps: Res<Time>, mut requete_planetes: Query<(&mut Transform, &mut Planete)>) {
    for (mut transform, mut planete) in requete_planetes.iter_mut() {
        planete.angle_actuel += planete.vitesse_orbite * temps.delta_seconds();
        transform.translation.x = planete.rayon_orbite * planete.angle_actuel.cos();
        transform.translation.y = planete.rayon_orbite * planete.angle_actuel.sin();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrophysique::{ClasseSpectrale, SystemeStellaire};
    use bevy::render::camera::CameraProjection;
    use bevy::window::PrimaryWindow;
    use std::time::Duration;

    // --- Fonction utilitaire pour préparer l'environnement ---
    fn preparer_app() -> App {
        let mut app = App::new();

        // Les assets sont requis pour les MaterialMesh2dBundle générés
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));

        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();

        // Initialisation des ressources requises par le système
        app.insert_resource(GraineGlobale(42));

        // OBLIGATOIRE : Initialisation du HashSet pour éviter un crash "Resource does not exist"
        app.init_resource::<SecteursCharges>();

        // Création de notre caméra principale au centre (0, 0)
        app.world_mut()
            .spawn((Camera2dBundle::default(), CameraPrincipale));

        app
    }

    // --- Fonction utilitaire pour préparer l'environnement ---
    fn preparer_app_gc() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // La ressource SecteursCharges doit exister
        app.init_resource::<SecteursCharges>();

        // Création de notre caméra (par défaut en 0, 0, avec un scale/zoom de 1.0)
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), CameraPrincipale));

        app
    }

    // --- Fonction utilitaire pour préparer l'environnement ---
    fn preparer_app_clic() -> App {
        let mut app = App::new();

        // Les assets sont requis pour faire apparaître les planètes (MaterialMesh2dBundle)
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();

        // On initialise l'input de la souris
        app.init_resource::<ButtonInput<MouseButton>>();

        // 1. Simuler une fenêtre avec la souris au centre (400, 300)
        let mut fenetre = Window::default();
        fenetre.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
        app.world_mut().spawn((fenetre, PrimaryWindow));

        // 2. Simuler la caméra centrée en 0,0
        let mut camera_bundle = Camera2dBundle::default();
        camera_bundle.projection.update(800.0, 600.0);
        app.world_mut().spawn((camera_bundle, CameraPrincipale));

        app
    }

    // --- Fonction utilitaire pour préparer l'environnement ---
    fn preparer_app_lod() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    // --- Initialisation et chargement du premier secteur ---
    #[test]
    fn test_generer_univers_charge_les_premiers_secteurs() {
        let mut app = preparer_app();
        app.add_systems(Update, generer_univers_dynamique);

        // Au départ, le HashSet des secteurs chargés doit être vide
        assert!(app.world().resource::<SecteursCharges>().0.is_empty());

        // On exécute le système une fois (la caméra est en 0,0)
        app.update();

        // Le système a dû marquer de nombreux secteurs (autour de 0,0) comme "chargés"
        let secteurs = app.world().resource::<SecteursCharges>();
        assert!(
            !secteurs.0.is_empty(),
            "Les secteurs autour de la caméra auraient dû être générés"
        );

        // On vérifie si des entités Étoile ont été créées dans l'univers
        // (Avec la graine 42 et un rayon > 10, il y a mathématiquement des étoiles générées)
        let mut requete_etoiles = app.world_mut().query::<&Etoile>();
        let nombre_etoiles = requete_etoiles.iter(app.world()).count();

        assert!(
            nombre_etoiles > 0,
            "Au moins une étoile aurait dû apparaître avec cette graine"
        );
    }

    // --- Optimisation (Ignorer les petits mouvements) ---
    #[test]
    fn test_generer_univers_ignore_les_micro_mouvements() {
        let mut app = preparer_app();
        app.add_systems(Update, generer_univers_dynamique);

        // 1ère frame : Initialisation (la position (0,0) est enregistrée dans le `Local`)
        app.update();

        // On mémorise le nombre exact de secteurs actuellement chargés
        let secteurs_avant = app.world().resource::<SecteursCharges>().0.len();

        // On simule un tout petit mouvement de la caméra (distance < 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<CameraPrincipale>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 15.0;

        // 2ème frame
        app.update();

        // L'optimisation (le `return` de la ligne 61) doit avoir fonctionné.
        // Aucun nouveau secteur ne doit avoir été calculé.
        let secteurs_apres = app.world().resource::<SecteursCharges>().0.len();
        assert_eq!(
            secteurs_avant, secteurs_apres,
            "La fonction aurait dû retourner prématurément sans charger de nouveaux secteurs"
        );
    }

    // --- Génération après un grand déplacement ---
    #[test]
    fn test_generer_univers_reprend_apres_grand_deplacement() {
        let mut app = preparer_app();
        app.add_systems(Update, generer_univers_dynamique);

        // 1ère frame (initialisation)
        app.update();
        let secteurs_avant = app.world().resource::<SecteursCharges>().0.len();

        // On téléporte la caméra très loin (distance > 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<CameraPrincipale>>();
        let mut transform = requete_camera.single_mut(app.world_mut());
        transform.translation.x += 1000.0;
        transform.translation.y += 1000.0;

        // 2ème frame
        app.update();

        // Le seuil de 40.0 étant dépassé, la fonction a dû calculer la nouvelle zone spatiale
        let secteurs_apres = app.world().resource::<SecteursCharges>().0.len();
        assert!(
            secteurs_apres > secteurs_avant,
            "La caméra s'est téléportée de 1000 unités, de nouveaux secteurs auraient dû être chargés"
        );
    }

    // --- Nettoyage effectif ---
    #[test]
    fn test_garbage_collector_nettoie_hors_limites() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, garbage_collector_spatial);

        // On peuple l'univers avec deux étoiles et deux secteurs correspondants
        let entite_proche = app
            .world_mut()
            .spawn(Etoile {
                grille_x: 2,
                grille_y: 2,
            })
            .id();
        // Une étoile très lointaine (grille 50, soit 4000 pixels de distance)
        let entite_lointaine = app
            .world_mut()
            .spawn(Etoile {
                grille_x: 50,
                grille_y: 50,
            })
            .id();

        let mut secteurs = app.world_mut().resource_mut::<SecteursCharges>();
        secteurs.0.insert((2, 2));
        secteurs.0.insert((50, 50));

        // Pour forcer l'exécution du Garbage Collector, on déplace la caméra d'au moins 40 unités.
        // Sinon, la variable Local<Vec2> initialisée à (0,0) provoquera un "return" prématuré.
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<CameraPrincipale>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 100.0; // Centre grille recalculé à environ (1, 0)

        app.update();

        // --- VÉRIFICATIONS ---

        // L'étoile proche doit avoir survécu
        assert!(app.world().get_entity(entite_proche).is_some());

        // L'étoile lointaine a dû être désintégrée (despawn_recursive) par le système
        assert!(app.world().get_entity(entite_lointaine).is_none());

        // Le HashSet des secteurs chargés a dû être purgé de la coordonnée lointaine
        let secteurs_apres = app.world().resource::<SecteursCharges>();
        assert!(secteurs_apres.0.contains(&(2, 2)));
        assert!(
            !secteurs_apres.0.contains(&(50, 50)),
            "Le secteur lointain aurait dû être retiré de la mémoire"
        );
    }

    // --- L'optimisation locale ---
    #[test]
    fn test_garbage_collector_ignore_les_micro_mouvements() {
        let mut app = preparer_app_gc();
        app.add_systems(Update, garbage_collector_spatial);

        // Frame 1 : On place la caméra très loin pour initialiser les variables locales `derniere_pos_maj`
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<CameraPrincipale>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x = 1000.0;
        app.update();

        // On crée une étoile complètement hors de portée
        let entite_lointaine = app
            .world_mut()
            .spawn(Etoile {
                grille_x: 9999,
                grille_y: 9999,
            })
            .id();

        // Frame 2 : On simule un mouvement de 10 unités (< 40.0)
        let mut requete_camera = app
            .world_mut()
            .query_filtered::<&mut Transform, With<CameraPrincipale>>();
        let mut camera_transform = requete_camera.single_mut(app.world_mut());
        camera_transform.translation.x += 10.0;

        app.update();

        // Puisque le mouvement est trop faible, le système a dû faire un `return` à la ligne 61.
        // L'entité lointaine n'a donc pas été nettoyée, prouvant que l'optimisation fonctionne.
        assert!(
            app.world().get_entity(entite_lointaine).is_some(),
            "Le GC n'aurait pas dû s'exécuter"
        );
    }

    // --- Masquage lors d'un dézoom ---
    #[test]
    fn test_gerer_lod_cache_les_planetes_si_zoom_superieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, gerer_lod_planetes);

        // On crée une caméra avec un zoom (scale.x) supérieur au seuil de 3.5
        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(4.0)), CameraPrincipale));

        // On crée une planète qui est actuellement visible (Inherited)
        let entite_planete = app
            .world_mut()
            .spawn((
                Planete {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Inherited,
            ))
            .id();

        app.update();

        // Le système doit avoir détecté le zoom > 3.5 et caché la planète
        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Hidden);
    }

    // --- Affichage lors d'un zoom ---
    #[test]
    fn test_gerer_lod_affiche_les_planetes_si_zoom_inferieur_au_seuil() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, gerer_lod_planetes);

        // On crée une caméra avec un zoom très proche (inférieur au seuil de 3.5)
        app.world_mut()
            .spawn((Transform::from_scale(Vec3::splat(1.0)), CameraPrincipale));

        // On crée une planète actuellement cachée
        let entite_planete = app
            .world_mut()
            .spawn((
                Planete {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                Visibility::Hidden,
            ))
            .id();

        app.update();

        // Le système doit avoir restauré la visibilité normale (Inherited) de la planète
        let visibilite = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(*visibilite, Visibility::Inherited);
    }

    // --- L'optimisation Changed<Transform> ---
    #[test]
    fn test_gerer_lod_ne_fait_rien_si_camera_immobile() {
        let mut app = preparer_app_lod();
        app.add_systems(Update, gerer_lod_planetes);

        // On crée la caméra
        let entite_camera = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(2.0)), CameraPrincipale))
            .id();

        let entite_planete = app
            .world_mut()
            .spawn((
                Planete {
                    rayon_orbite: 10.0,
                    angle_actuel: 0.0,
                    vitesse_orbite: 1.0,
                },
                // On la force à un état incorrect par rapport au zoom pour tester le filtre
                Visibility::Hidden,
            ))
            .id();

        // Première frame : Le composant vient d'être créé, il compte comme "Changed".
        // Le système va donc corriger la visibilité à Inherited.
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );

        // On la force MANUELLEMENT à Hidden à nouveau
        *app.world_mut()
            .get_mut::<Visibility>(entite_planete)
            .unwrap() = Visibility::Hidden;

        // Deuxième frame : La caméra n'a pas été modifiée !
        app.update();

        // Comme la caméra n'a pas bougé, la requête Changed<Transform> renvoie une erreur
        // Le système ne s'est pas exécuté, la planète est donc restée dans son état forcé (Hidden).
        let visibilite_finale = app.world().get::<Visibility>(entite_planete).unwrap();
        assert_eq!(
            *visibilite_finale,
            Visibility::Hidden,
            "Le système a ignoré le filtre Changed<Transform>"
        );

        // Troisième frame : On bouge la caméra
        app.world_mut()
            .get_mut::<Transform>(entite_camera)
            .unwrap()
            .scale
            .x = 2.1;
        app.update();

        // Le système s'est réveillé et a corrigé la visibilité
        assert_eq!(
            *app.world().get::<Visibility>(entite_planete).unwrap(),
            Visibility::Inherited
        );
    }

    // --- Ignorer si aucun clic n'est détecté ---
    #[test]
    fn test_gerer_clic_ignore_si_pas_de_clic() {
        let mut app = preparer_app_clic();
        app.add_systems(Update, gerer_clic_etoile);

        // On fait apparaître une étoile au centre
        let entite_etoile = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                SystemeStellaire {
                    nom: "Test-Beta".to_string(),
                    classe: ClasseSpectrale::G,
                    masse_solaire: 1.0,
                    rayon_solaire: 1.0,
                    nb_planetes: 2,
                    age_milliards_annees: 4.0,
                },
                Etoile {
                    grille_x: 0,
                    grille_y: 0,
                },
            ))
            .id();

        // On lance le système SANS simuler de clic de souris
        app.update();

        // L'étoile ne doit pas avoir reçu le composant SystemeDeveloppe
        assert!(app.world().get::<SystemeDeveloppe>(entite_etoile).is_none());
    }

    #[test]
    fn test_animer_orbites_calcule_position_et_angle_correctement() {
        let mut app = App::new();

        // Initialisation du temps simulé
        // On déclare explicitement le type `: Time` pour aider le compilateur
        let mut time: Time = Time::default();

        // On avance le temps d'exactement 0.5 seconde
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);

        // Création de notre planète de test
        let rayon = 100.0;
        // On définit une vitesse d'orbite de PI radians par seconde (un demi-tour par seconde)
        let vitesse = std::f32::consts::PI;

        let entite = app
            .world_mut()
            .spawn((
                // La position de départ n'a aucune importance,
                // car votre système écrase la valeur X et Y quoi qu'il arrive.
                Transform::default(),
                Planete {
                    rayon_orbite: rayon,
                    angle_actuel: 0.0, // Départ à 0 radian (tout à droite du cercle)
                    vitesse_orbite: vitesse,
                },
            ))
            .id();

        // Exécution du système
        app.add_systems(Update, animer_orbites);
        app.update();

        // Vérifications mathématiques
        let planete = app.world().get::<Planete>(entite).unwrap();
        let transform = app.world().get::<Transform>(entite).unwrap();

        // L'angle actuel a dû augmenter selon la formule : vitesse * delta_seconds[cite: 2]
        // PI rad/s * 0.5s = PI/2 rad (soit 90 degrés, le sommet du cercle)
        let angle_attendu = vitesse * 0.5;
        assert_eq!(planete.angle_actuel, angle_attendu);

        // À PI/2 (90 degrés) sur un cercle de rayon 100 :
        // cos(PI/2) est environ 0.0 -> La position X doit être 0.0[cite: 2]
        // sin(PI/2) est 1.0 -> La position Y doit être 100.0[cite: 2]
        let difference_x = (transform.translation.x - 0.0).abs();
        let difference_y = (transform.translation.y - 100.0).abs();

        // On utilise toujours une tolérance (Delta) pour les flottants
        assert!(
            difference_x < 0.0001,
            "La position X calculée avec le cosinus est incorrecte"
        );
        assert!(
            difference_y < 0.0001,
            "La position Y calculée avec le sinus est incorrecte"
        );
    }
}
