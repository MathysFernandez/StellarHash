use bevy::prelude::*;

#[derive(Resource)]
struct GraineGlobale(u32);



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

        // Systèmes exécutés UNE fois au démarrage 
        // (Startup)
        .add_systems(Startup, initialiser_camera)

        // Systèmes exécutés en boucle à chaque image
        // (Update)
        .add_systems(Update, afficher_infos_debug)
        .run();
}


// Système d'initialisation: Fait apparaître la caméra 2D
fn initialiser_camera(mut commands: Commands) {
    // Commands permet d'instancier (spawn) des entités dans notre monde 
    commands.spawn(Camera2dBundle::default());
    println!("Caméra 2D initialisée");
}

// Système de test: 
// Lit la ressource globale et l'affiche dans le terminal
fn afficher_infos_debug(graine: Res<GraineGlobale>, temps: Res<Time>) {
    // On n'affiche le message qu'une fois par seconde pour ne pas spammer le terminal
    if temps.elapsed_seconds() % 1.0 < temps.delta_seconds() {
        println!("Graine actuelle : {}", graine.0);
    }
}