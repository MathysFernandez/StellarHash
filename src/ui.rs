use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
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
            .insert_resource(ChronoAnecdote(Timer::from_seconds(
                60.0,
                TimerMode::Repeating,
            )))
            .add_systems(
                Startup,
                (
                    initialiser_fps,
                    initialiser_panneau_info,
                    initialiser_panneau_anecdotes,
                ),
            )
            .add_systems(
                Update,
                (
                    mettre_a_jour_fps,
                    gerer_survol_souris,
                    mettre_a_jour_anecdotes,
                ),
            );
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

fn initialiser_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");
    // On crée une une boîte d'interface collée en haut à gauche
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // À l'intérieur de cette boîte, on ajoute notre texte
            parent.spawn((
                TextBundle::from_section(
                    "FPS: calcul...",
                    TextStyle {
                        font: police,
                        font_size: 24.0,
                        color: Color::srgb(0.0, 1.0, 0.0),
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

fn initialiser_panneau_info(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");

    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            // Le texte à l'intérieur
            parent.spawn((
                TextBundle::from_section(
                    "Données Stellaire",
                    TextStyle {
                        font: police,
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ),
                TexteInfo,
            ));
        });
}

fn gerer_survol_souris(
    requete_fenetre: Query<&Window, With<PrimaryWindow>>,
    requete_camera: Query<(&Camera, &GlobalTransform), With<CameraPrincipale>>,
    requete_etoiles: Query<(&Transform, &SystemeStellaire, &Etoile)>,
    mut requete_panneau: Query<&mut Style, With<PanneauInfo>>,
    mut requete_texte: Query<&mut Text, With<TexteInfo>>,
) {
    let fenetre = requete_fenetre.single();
    let (camera, camera_transform) = requete_camera.single();

    // On vérifie si la souris est bien dans la fenêtre
    if let Some(position_curseur_ecran) = fenetre.cursor_position() {
        // On convertit les pixels de l'écran en coordonnées du Monde 2D (Raycasting mathématique)
        if let Some(position_monde) =
            camera.viewport_to_world_2d(camera_transform, position_curseur_ecran)
        {
            let mut etoile_survolee = None;

            // On calcule la case sur laquelle se trouve la souris
            let taille_secteur = 80.0;
            let souris_grille_x = (position_monde.x / taille_secteur).round() as i32;
            let souris_grille_y = (position_monde.y / taille_secteur).round() as i32;

            // On teste toutes les étoiles actuellement affichées pour voir si la souris est dessus
            for (transform_etoile, systeme, etoile) in requete_etoiles.iter() {
                // On ignore instantanément toutes les étoiles qui ne sont pas dans la case de la souris ou dans les cases voisines.
                if (etoile.grille_x - souris_grille_x).abs() <= 1
                    && (etoile.grille_y - souris_grille_y).abs() <= 1
                {
                    let distance = position_monde.distance(transform_etoile.translation.truncate());

                    if distance < 25.0 {
                        etoile_survolee = Some(systeme);
                        break;
                    }
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

fn initialiser_panneau_anecdotes(mut commands: Commands, asset_server: Res<AssetServer>) {
    let police = asset_server.load("../fonts/GeistPixel.ttf");

    let lignes: Vec<&str> = FICHIER_ANECDOTES
        .lines()
        .filter(|l| !l.is_empty())
        .collect();
    let texte_initial = if !lignes.is_empty() {
        let temps_actuel = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;

        // On utilise ce nombre pour choisir une ligne au "hasard" mdr
        let index_aleatoire = temps_actuel % lignes.len();

        lignes[index_aleatoire]
    } else {
        "Base de données vide."
    };

    commands
        .spawn(NodeBundle {
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
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    texte_initial,
                    TextStyle {
                        font: police,
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.8, 1.0),
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
        let lignes: Vec<&str> = FICHIER_ANECDOTES
            .lines()
            .filter(|l| !l.is_empty())
            .collect();

        if lignes.is_empty() {
            return;
        }

        let temps_actuel = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        let index_aleatoire = temps_actuel % lignes.len();

        // On met à jour le texte
        let mut texte = requete_texte.single_mut();
        texte.sections[0].value = format!("{}", lignes[index_aleatoire]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::diagnostic::Diagnostic;

    #[test]
    fn test_initialiser_fps_cree_composants() {
        // Create a new empty Bevy App
        let mut app = App::new();

        // Add the AssetServer plugin (required for loading fonts)
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        // Add our system to the Startup schedule
        app.add_systems(Startup, initialiser_fps);

        // Run the app for one frame to execute the Startup schedule
        app.update();

        // Query to check if a node with TexteFps component was created
        let mut requete_texte = app.world_mut().query_filtered::<&Text, With<TexteFps>>();

        // If the query fails to find exactly one entity, the test will panic
        let texte = requete_texte.single(app.world());

        // Verify that the initial text is correct
        assert_eq!(texte.sections[0].value, "FPS: calcul...");
    }

    #[test]
    fn test_initialiser_fps_position_correcte() {
        let mut app = App::new();

        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        app.add_systems(Startup, initialiser_fps);
        app.update();

        // The TexteFps is a child of the main NodeBundle.
        // We need to find the parent node and check its style.
        // First, let's find the entity with TexteFps
        let mut query = app.world_mut().query_filtered::<Entity, With<TexteFps>>();
        let entity_texte = query.single(app.world());

        // Now, get its Parent component
        let parent_component = app.world().get::<Parent>(entity_texte).unwrap();
        let entity_parent = parent_component.get();

        // Finally, get the Style component of the parent node
        let style = app.world().get::<Style>(entity_parent).unwrap();

        // Assert the positioning values we set in the function
        assert_eq!(style.position_type, PositionType::Absolute);
        assert_eq!(style.top, Val::Px(10.0));
        assert_eq!(style.left, Val::Px(10.0));
    }

    #[test]
    fn test_mettre_a_jour_fps_affiche_valeur_arrondie() {
        let mut app = App::new();

        // Instancier l'entité texte avec les composants requis
        let entite = app
            .world_mut()
            .spawn((
                TextBundle::from_section("FPS: calcul...", TextStyle::default()),
                TexteFps,
            ))
            .id();

        // Simuler les diagnostics du moteur
        let mut diagnostics = DiagnosticsStore::default();
        let mut diagnostic_fps = Diagnostic::new(FrameTimeDiagnosticsPlugin::FPS);

        // On injecte manuellement une mesure factice de FPS (ex: 60.48)
        diagnostic_fps.add_measurement(bevy::diagnostic::DiagnosticMeasurement {
            time: bevy::utils::Instant::now(),
            value: 60.48,
        });
        diagnostics.add(diagnostic_fps);

        // On insère cette fausse base de données dans l'application de test
        app.insert_resource(diagnostics);

        // Ajouter et exécuter notre système
        app.add_systems(Update, mettre_a_jour_fps);
        app.update();

        // Vérifier le résultat
        let texte = app.world().get::<Text>(entite).unwrap();

        // Le format {:.1} de votre système doit arrondir 60.48 à 60.5
        assert_eq!(texte.sections[0].value, "FPS: 60.5");
    }

    #[test]
    fn test_mettre_a_jour_fps_ignore_si_pas_de_donnees() {
        let mut app = App::new();

        // On démarre avec le texte par défaut
        let entite = app
            .world_mut()
            .spawn((
                TextBundle::from_section("FPS: calcul...", TextStyle::default()),
                TexteFps,
            ))
            .id();

        // On prépare le conteneur de FPS, mais cette fois on n'ajoute AUCUNE mesure
        let mut diagnostics = DiagnosticsStore::default();
        let diagnostic_fps = Diagnostic::new(FrameTimeDiagnosticsPlugin::FPS);
        diagnostics.add(diagnostic_fps);
        app.insert_resource(diagnostics);

        app.add_systems(Update, mettre_a_jour_fps);
        app.update();

        // On récupère le texte après l'exécution du système
        let texte = app.world().get::<Text>(entite).unwrap();

        // Puisque `fps.smoothed()` a renvoyé None, le texte n'a pas dû changer
        assert_eq!(texte.sections[0].value, "FPS: calcul...");
    }

    #[test]
    fn test_initialiser_panneau_info_est_cache_par_defaut() {
        let mut app = App::new();

        // Configuration indispensable pour charger la police sans erreur
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        // Ajout et exécution du système
        app.add_systems(Startup, initialiser_panneau_info);
        app.update();

        // On récupère le composant Style de notre panneau d'information
        let mut requete_panneau = app
            .world_mut()
            .query_filtered::<&Style, With<PanneauInfo>>();

        // S'il n'y a pas exactement un PanneauInfo, le test va échouer
        let style_panneau = requete_panneau.single(app.world());

        // VÉRIFICATIONS CRITIQUES :
        // Le panneau DOIT être invisible (Display::None) au démarrage[cite: 1]
        assert_eq!(style_panneau.display, Display::None);

        // Le panneau doit avoir un positionnement absolu pour flotter sur l'écran[cite: 1]
        assert_eq!(style_panneau.position_type, PositionType::Absolute);

        // Les éléments internes doivent être empilés en colonne[cite: 1]
        assert_eq!(style_panneau.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_initialiser_panneau_info_cree_le_texte_enfant() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();

        app.add_systems(Startup, initialiser_panneau_info);
        app.update();

        // On cherche l'entité qui possède le marqueur TexteInfo
        let mut requete_texte = app.world_mut().query_filtered::<&Text, With<TexteInfo>>();
        let texte = requete_texte.single(app.world());

        // On vérifie le contenu par défaut[cite: 1]
        assert_eq!(texte.sections[0].value, "Données Stellaire");

        // On vérifie le style du texte (taille et couleur)[cite: 1]
        assert_eq!(texte.sections[0].style.font_size, 18.0);
        assert_eq!(texte.sections[0].style.color, Color::WHITE);
    }
}
