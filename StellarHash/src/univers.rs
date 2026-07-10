use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;


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
) {
    let camera_transform = requete_camera.single();
    let cam_x = camera_transform.translation.x;
    let cam_y = camera_transform.translation.y;
    let zoom = camera_transform.scale.x;

    let taille_secteur = 80.0;
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

            if probabilite > 0.70 {
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
                        Etoile { grille_x: x, grille_y: y },
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
    requete_etoiles: Query<(Entity, &Etoile)>,
    mut secteurs_charges: ResMut<SecteursCharges>,
) {
    let camera_transform = requete_camera.single();
    let cam_x = camera_transform.translation.x;
    let cam_y = camera_transform.translation.y;
    let zoom = camera_transform.scale.x;

    let taille_secteur = 80.0;
    
    // On calcule le rayon de vision actu
    let rayon_vision = (1000.0 * zoom) as i32 / taille_secteur as i32;
    let rayon_vision = rayon_vision.clamp(10, 100);
    
    // On ajoute une "marge de sécurité" (padding).
    // On détruit les étoiles un peu PLUS LOIN que notre rayon de vision.
    let rayon_despawn = rayon_vision + 5;

    let centre_grille_x = (cam_x / taille_secteur).round() as i32;
    let centre_grille_y = (cam_y / taille_secteur).round() as i32;

    // Détruire les entite
    for (entite, etoile) in requete_etoiles.iter() {
        if (etoile.grille_x - centre_grille_x).abs() > rayon_despawn ||
           (etoile.grille_y - centre_grille_y).abs() > rayon_despawn {
            commands.entity(entite).despawn_recursive(); 
        }
    }




    // VIDER LE CACHE
    // Tout secteur en dehors du rayon de despawn sera définitivement effacé de la mémoire.
    secteurs_charges.0.retain(|&(x, y)| {
        (x - centre_grille_x).abs() <= rayon_despawn &&
        (y - centre_grille_y).abs() <= rayon_despawn
    });
}


fn gerer_lod_planetes(
    requete_camera: Query<&Transform, With<CameraPrincipale>>,
    mut requete_planetes: Query<&mut Visibility, With<Planete>>,
) {
    let zoom = requete_camera.single().scale.x;
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




// Écoute le clic gauche,
// trouve l'étoile cliquée,
// et génère ses planètes
fn gerer_clic_etoile(
    mut commands: Commands,
    touches_souris: Res<ButtonInput<MouseButton>>,
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<CameraPrincipale>>,
    // Option<&SystemeDeveloppe> nous permet de savoir la présence de ce composant 
    requete_etoiles: Query<(Entity, &Transform, &crate::astrophysique::SystemeStellaire, Option<&SystemeDeveloppe>), With<Etoile>>,
) {
    if !touches_souris.just_pressed(MouseButton::Left) { return; }

    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    if let Some(position_curseur) = fenetre.cursor_position() {
        if let Some(position_monde) = camera.viewport_to_world_2d(camera_transform, position_curseur) {
            
            for (entite, transform_etoile, systeme, developpe) in requete_etoiles.iter() {
                let distance = position_monde.distance(transform_etoile.translation.truncate());
                
                // Si on clique sur une étoile (tolérance de 25 pixels)
                if distance < 25.0 {
                    if developpe.is_none() {
                        
                        commands.entity(entite).insert(SystemeDeveloppe)
                        
                        .with_children(|parent| {
                            for i in 0..systeme.nb_planetes {
                                // Espacement des orbites
                                let rayon_orbite = 15.0 + (i as f32 * 10.0); 
                                // Départ décalé
                                let angle_depart = (i as f32) * 1.2;
                                // Les planètes lointaines sont plus lentes (Loi de Kepler) 
                                // (voir les 3 lois dans le livre à la maison)
                                let vitesse = 1.5 / (i as f32 + 1.0); 

                                parent.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.6, 0.8, 0.9),
                                            custom_size: Some(Vec2::new(3.0, 3.0)),
                                            ..default()
                                        },
                                        // La position est calculée par rapport à l'étoile
                                        transform: Transform::from_xyz(
                                            rayon_orbite * angle_depart.cos(),
                                            rayon_orbite * angle_depart.sin(),
                                            1.0
                                            // Z = 1.0 pour passer par-dessus l'étoile (Z = 0)
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

// Applique la trigonométrie à chaque frame pour faire tourner les planètes
fn animer_orbites(
    temps: Res<Time>,
    mut requete_planetes: Query<(&mut Transform, &mut Planete)>,
) {
    for (mut transform, mut planete) in requete_planetes.iter_mut() {
        // On avance l'angle
        planete.angle_actuel += planete.vitesse_orbite * temps.delta_seconds();

        transform.translation.x = planete.rayon_orbite * planete.angle_actuel.cos();
        transform.translation.y = planete.rayon_orbite * planete.angle_actuel.sin();
    }
}
