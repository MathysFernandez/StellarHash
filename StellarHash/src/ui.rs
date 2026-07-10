use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::window::PrimaryWindow;
use std::time::SystemTime;

use crate::astrophysique::SystemeStellaire;
use crate::camera::CameraPrincipale;
use crate::univers::Etoile;

const FICHIER_ANECDOTES: &str = include_str!("../assets/anecdotes.txt");






pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
           .insert_resource(ChronoAnecdote(Timer::from_seconds(60.0, TimerMode::Repeating)))
           .add_systems(Startup, (initialiser_fps, initialiser_panneau_info, initialiser_panneau_anecdotes))
           .add_systems(Update, (mettre_a_jour_fps, gerer_survol_souris, mettre_a_jour_anecdotes));
    }
}



#[derive(Component)]
struct TexteFps;

#[derive(Component)]
struct PanneauInfo;

#[derive(Component)]
struct TexteInfo;

#[derive(Resource)]
struct ChronoAnecdote(Timer);

#[derive(Component)]
struct TexteAnecdote;






fn initialiser_fps(mut commands: Commands) {
    // On crée une une boîte d'interface collée en haut à gauche
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // À l'intérieur de cette boîte, on ajoute notre texte
        parent.spawn((
            TextBundle::from_section(
                "FPS: calcul...",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.0, 1.0, 0.0),
                    ..default()
                },
            ),
            TexteFps,
        ));
    });
}

fn mettre_a_jour_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut requete_texte: Query<&mut Text, With<TexteFps>>,
) {
    for mut texte in &mut requete_texte {
        // On récupère la donnée FPS depuis le moteur 
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            // smoothed() donne une moyenne lissée
            if let Some(valeur) = fps.smoothed() {
                texte.sections[0].value = format!("FPS: {:.1}", valeur);
            }
        }
    }
}


fn initialiser_panneau_info(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::None, // Caché par défaut
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // Fond noir semi-transparent
            ..default()
        },
        PanneauInfo,
    )).with_children(|parent| {
        // Le texte à l'intérieur
        parent.spawn((
            TextBundle::from_section(
                "Données Stellaire",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TexteInfo,
        ));
    });
}

fn gerer_survol_souris(
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<CameraPrincipale>>,
    requete_etoiles: Query<(&Transform, &SystemeStellaire), With<Etoile>>,
    mut requete_panneau: Query<&mut Style, With<PanneauInfo>>, 
    mut requete_texte: Query<&mut Text, With<TexteInfo>>,
) {
    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    // On vérifie si la souris est bien dans la fenêtre
    if let Some(position_curseur_ecran) = fenetre.cursor_position() {
        
        // On convertit les pixels de l'écran en coordonnées du Monde 2D (Raycasting mathématique)
        if let Some(position_monde) = camera.viewport_to_world_2d(camera_transform, position_curseur_ecran) {
            
            let mut etoile_survolee = None;

            // On teste toutes les étoiles actuellement affichées pour voir si la souris est dessus
            for (transform_etoile, systeme) in requete_etoiles.iter() {
                // On calcule la distance entre le curseur et le centre de l'étoile
                let distance = position_monde.distance(transform_etoile.translation.truncate());
                
                // Si la souris est à moins de 25 pixels de l'étoile (zone de tolérance pour faciliter le clic)
                if distance < 25.0 {
                    etoile_survolee = Some(systeme);
                    break; // Dès qu'on trouve une étoile, on arrête de chercher
                }
            }

            let mut style_panneau = requete_panneau.single_mut();
            let mut texte = requete_texte.single_mut();

            // Si on survole une étoile, on met à jour le texte et on affiche le panneau sous la souris
            if let Some(systeme) = etoile_survolee {
                style_panneau.display = Display::Flex;
                
                // On décalle un peu le panneau pour qu'il ne soit pas caché par le curseur de la souris
                style_panneau.left = Val::Px(position_curseur_ecran.x + 15.0);
                style_panneau.top = Val::Px(position_curseur_ecran.y + 15.0);

                texte.sections[0].value = format!(
                    "Systeme : {}\nClasse : {:?}\nMasse Solaire : {:.2} MS\nPlanetes : {}\nAge : {:.1} Ga",
                    systeme.nom,
                    systeme.classe,
                    systeme.masse_solaire,
                    systeme.nb_planetes,
                    systeme.age_milliards_annees
                );
            } else {
                // Si on est dans le vide spatial, on cache le panneau
                style_panneau.display = Display::None;
            }
        }
    }
}



fn initialiser_panneau_anecdotes(mut commands: Commands) {
    let lignes: Vec<&str> = FICHIER_ANECDOTES.lines().filter(|l| !l.is_empty()).collect();
    let texte_initial = if !lignes.is_empty() {
        let temps_actuel = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as usize;
        
        // On utilise ce nombre pour choisir une ligne au "hasard" mdr
        let index_aleatoire = temps_actuel % lignes.len();
        
        lignes[index_aleatoire]
    } else {
        "Base de données vide."
    };

    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Percent(40.0),
            max_width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.7)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                texte_initial,
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.8, 0.8, 1.0),
                    ..default()
                },
            ),
            TexteAnecdote,
        ));
    });
}

fn mettre_a_jour_anecdotes(
    temps: Res<Time>,
    mut chrono: ResMut<ChronoAnecdote>,
    mut requete_texte: Query<&mut Text, With<TexteAnecdote>>,
) {
    chrono.0.tick(temps.delta());

    if chrono.0.just_finished() {
        
        let lignes: Vec<&str> = FICHIER_ANECDOTES.lines().filter(|l| !l.is_empty()).collect();
        
        if lignes.is_empty() { return; }

        let temps_actuel = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as usize;
        let index_aleatoire = temps_actuel % lignes.len();

        // On met à jour le texte
        let mut texte = requete_texte.single_mut();
        texte.sections[0].value = format!("{}", lignes[index_aleatoire]);
    }
}