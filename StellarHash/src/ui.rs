use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // On ajoute le plugin interne de Bevy qui calcule les FPS
        app.add_plugins(FrameTimeDiagnosticsPlugin)
           .add_systems(Startup, initialiser_fps)
           .add_systems(Update, mettre_a_jour_fps);
    }
}



#[derive(Component)]
struct TexteFps;






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