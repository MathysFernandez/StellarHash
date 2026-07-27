use bevy::prelude::*;

use StellarHash::camera;
use StellarHash::ui;
use StellarHash::univers;

fn main() {
    App::new()
        // Les Plugins de base du moteur
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "StellarHash".to_string(),
                mode: bevy::window::WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        // Plugins personnalisés
        .add_plugins(camera::CameraPlugin)
        .add_plugins(univers::UniversPlugin)
        .add_plugins(ui::UiPlugin)
        .add_systems(Update, quitter_jeu)
        .run();
}

// Permet de quitter le jeu avec la touche Echap
fn quitter_jeu(
    touches: Res<ButtonInput<KeyCode>>,
    mut evenements_sortie: EventWriter<bevy::app::AppExit>,
) {
    if touches.just_pressed(KeyCode::Escape) {
        evenements_sortie.send(bevy::app::AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::AppExit;

    // --- Fonction utilitaire pour préparer l'environnement ---
    fn preparer_app_quitter() -> App {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);

        // On initialise la ressource qui gère les entrées clavier
        app.init_resource::<ButtonInput<KeyCode>>();

        app
    }

    // --- Aucun événement si la touche n'est pas pressée ---
    #[test]
    fn test_quitter_jeu_ignore_sans_echap() {
        let mut app = preparer_app_quitter();
        app.add_systems(Update, quitter_jeu);

        // On exécute le système sans toucher au clavier
        app.update();

        // On récupère la file d'événements AppExit
        let evenements = app.world().resource::<Events<AppExit>>();
        let lecteur = evenements.get_reader();

        // La file doit être strictement vide
        assert!(
            lecteur.is_empty(evenements),
            "Aucun événement de sortie ne devrait être envoyé"
        );
    }

    // --- Envoi de l'événement AppExit::Success si Echap est pressé ---
    #[test]
    fn test_quitter_jeu_envoie_event_avec_echap() {
        let mut app = preparer_app_quitter();
        app.add_systems(Update, quitter_jeu);

        // On récupère la ressource clavier pour simuler un appui sur Échap
        let mut touches = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        touches.press(KeyCode::Escape);

        // On exécute le système pour qu'il lise l'entrée et génère l'événement
        app.update();

        // On récupère la file d'événements
        let evenements = app.world().resource::<Events<AppExit>>();
        let mut lecteur = evenements.get_reader();

        // On lit les événements qui ont été envoyés durant cette frame
        let mut nombre_evenements = 0;
        for event in lecteur.read(evenements) {
            assert_eq!(
                *event,
                AppExit::Success,
                "L'événement doit être AppExit::Success"
            );
            nombre_evenements += 1;
        }

        assert_eq!(
            nombre_evenements, 1,
            "Un seul événement de sortie aurait dû être généré"
        );
    }
}
