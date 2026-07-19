use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use crate::generation;
use crate::camera::CameraPrincipale;

pub struct UniversPlugin;

impl Plugin for UniversPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraineGlobale(42))
           .init_resource::<SecteursCharges>()
           .add_systems(Update, (
               generer_univers_dynamique,
               garbage_collector_spatial,
               gerer_clic_etoile,
               animer_orbites,
               gerer_lod_planetes
           ));
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
                let systeme_stellaire = crate::astrophysique::generer_caracteristiques(x, y, probabilite);

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
                            0.0
                        ),
                        ..default()
                    },
                    Etoile { grille_x: x, grille_y: y },
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
        if (etoile.grille_x - centre_grille_x).abs() > rayon_despawn ||
           (etoile.grille_y - centre_grille_y).abs() > rayon_despawn {
            commands.entity(entite).despawn_recursive(); 
        }
    }

    secteurs_charges.0.retain(|&(x, y)| {
        (x - centre_grille_x).abs() <= rayon_despawn &&
        (y - centre_grille_y).abs() <= rayon_despawn
    });
}

fn gerer_lod_planetes(
    requete_camera: Query<&Transform, (With<CameraPrincipale>, Changed<Transform>)>,
    mut requete_planetes: Query<&mut Visibility, With<Planete>>,
) {
    if let Ok(camera_transform) = requete_camera.get_single() {
        let zoom = camera_transform.scale.x;
        let seuil_lod = 3.5; 
        let visibilite_voulue = if zoom > seuil_lod { Visibility::Hidden } else { Visibility::Inherited };

        for mut visibilite in requete_planetes.iter_mut() {
            if *visibilite != visibilite_voulue { *visibilite = visibilite_voulue; }
        }
    }
}

fn gerer_clic_etoile(
    mut commands: Commands,
    touches_souris: Res<ButtonInput<MouseButton>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<CameraPrincipale>>,
    requete_etoiles: Query<(Entity, &Transform, &crate::astrophysique::SystemeStellaire, Option<&SystemeDeveloppe>, &Etoile)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !touches_souris.just_pressed(MouseButton::Left) { return; }

    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    if let Some(position_curseur) = fenetre.cursor_position() {
        if let Some(position_monde) = camera.viewport_to_world_2d(camera_transform, position_curseur) {
            
            let taille_secteur = 80.0;
            let clic_grille_x = (position_monde.x / taille_secteur).round() as i32;
            let clic_grille_y = (position_monde.y / taille_secteur).round() as i32;

            for (entite, transform_etoile, systeme, developpe, etoile) in requete_etoiles.iter() {
                if (etoile.grille_x - clic_grille_x).abs() > 1 || (etoile.grille_y - clic_grille_y).abs() > 1 {
                    continue; 
                }
                
                if position_monde.distance(transform_etoile.translation.truncate()) < 25.0 {
                    if developpe.is_none() {
                        commands.entity(entite).insert(SystemeDeveloppe).with_children(|parent| {
                            for i in 0..systeme.nb_planetes {
                                let rayon_orbite = 15.0 + (i as f32 * 10.0); 
                                let angle_depart = (i as f32) * 1.2;
                                let vitesse = 1.5 / (i as f32 + 1.0);

                                parent.spawn((
                                    MaterialMesh2dBundle {
                                        mesh: Mesh2dHandle(meshes.add(Circle::new(2.0))),
                                        material: materials.add(ColorMaterial::from(Color::srgb(0.6, 0.8, 0.9))),
                                        transform: Transform::from_xyz(rayon_orbite * angle_depart.cos(), rayon_orbite * angle_depart.sin(), 1.0),
                                        ..default()
                                    },
                                    Planete { rayon_orbite, angle_actuel: angle_depart, vitesse_orbite: vitesse },
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

fn animer_orbites(
    temps: Res<Time>,
    mut requete_planetes: Query<(&mut Transform, &mut Planete)>,
) {
    for (mut transform, mut planete) in requete_planetes.iter_mut() {
        planete.angle_actuel += planete.vitesse_orbite * temps.delta_seconds();
        transform.translation.x = planete.rayon_orbite * planete.angle_actuel.cos();
        transform.translation.y = planete.rayon_orbite * planete.angle_actuel.sin();
    }
}