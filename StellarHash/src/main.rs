use bevy::prelude::*;

mod generation;
mod camera;
mod univers;
mod ui;

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
fn quitter_jeu(touches: Res<ButtonInput<KeyCode>>, mut evenements_sortie: EventWriter<bevy::app::AppExit>) {
    if touches.just_pressed(KeyCode::Escape) {
        evenements_sortie.send(bevy::app::AppExit::Success);
    }
}