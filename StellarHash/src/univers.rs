use bevy::prelude::*;
use bevy::utils::HashSet;

// On importe notre fonction de "hachage"
use crate::generation;

// On a besoin de la caméra
use crate::camera::CameraPrincipale;

// Le Plugin qui regroupe la génération procédurale et les astres
pub struct UniversPlugin;

impl Plugin for UniversPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraineGlobale(42))
           .init_resource::<SecteursCharges>()
           .add_systems(Update, (
               generer_univers_dynamique,
               garbage_collector_spatial
           ));
    }
}




#[derive(Resource)]
pub struct GraineGlobale(pub u32);

#[derive(Resource, Default)]
pub struct SecteursCharges(pub HashSet<(i32, i32)>);

#[derive(Component)]
pub struct Etoile;




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

    let taille_secteur = 150.0;
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

            let probabilite = generation::calculer_hachage_spatial(x, y, graine.0);

            if probabilite > 0.95 {
                // Le filtre du maximum locale
                let mut est_le_plus_grand = true;
                
                'voisins: for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        
                        let prob_voisin = generation::calculer_hachage_spatial(x + dx, y + dy, graine.0);
                        if prob_voisin >= probabilite {
                            est_le_plus_grand = false;
                            break 'voisins; 
                        }
                    }
                }


                if est_le_plus_grand {
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
                        SpriteBundle {
                            sprite: Sprite {
                                color: couleur_etoile,
                                custom_size: Some(Vec2::new(taille_visuelle, taille_visuelle)),
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                (x as f32 * taille_secteur) + (probabilite * 50.0),
                                (y as f32 * taille_secteur) - (probabilite * 50.0), 
                                0.0
                            ),
                            ..default()
                        },
                        Etoile,
                        systeme_stellaire,
                    ));
                }
            }
        }
    }
}



fn garbage_collector_spatial(
    mut commands: Commands,
    requete_camera: Query<&Transform, With<CameraPrincipale>>,
    requete_etoiles: Query<(Entity, &Transform), With<Etoile>>,
    mut secteurs_charges: ResMut<SecteursCharges>,
) {
    let camera_transform = requete_camera.single();
    let cam_x = camera_transform.translation.x;
    let cam_y = camera_transform.translation.y;
    let zoom = camera_transform.scale.x;

    let taille_secteur = 150.0;
    
    // On calcule le rayon de vision actu
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);
    
    // On ajoute une "marge de sécurité" (padding).
    // On détruit les étoiles un peu PLUS LOIN que notre rayon de vision.
    let rayon_despawn = rayon_vision + 5;

    let centre_grille_x = (cam_x / taille_secteur).round() as i32;
    let centre_grille_y = (cam_y / taille_secteur).round() as i32;

    // Détruire les entite
    for (entite, transform_etoile) in requete_etoiles.iter() {
        let etoile_x = (transform_etoile.translation.x / taille_secteur).round() as i32;
        let etoile_y = (transform_etoile.translation.y / taille_secteur).round() as i32;

        if (etoile_x - centre_grille_x).abs() > rayon_despawn ||
           (etoile_y - centre_grille_y).abs() > rayon_despawn {
            commands.entity(entite).despawn();
        }
    }




    // VIDER LE CACHE
    // Tout secteur en dehors du rayon de despawn sera définitivement effacé de la mémoire.
    secteurs_charges.0.retain(|&(x, y)| {
        (x - centre_grille_x).abs() <= rayon_despawn &&
        (y - centre_grille_y).abs() <= rayon_despawn
    });
}

