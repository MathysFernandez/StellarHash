use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

// Le Plugin qui regroupe toute la logique de la caméra


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialiser_camera)
           .add_systems(Update, (deplacer_camera, zoomer_camera));
    }
}



#[derive(Component)]
pub struct CameraPrincipale;




fn initialiser_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraPrincipale,
    ));
}

fn deplacer_camera(
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut requete_camera: Query<&mut Transform, With<CameraPrincipale>>,
) {
    let mut transform = requete_camera.single_mut();
    let vitesse = 500.0 * transform.scale.x; 

    let mut direction = Vec3::ZERO;
    if touches.pressed(KeyCode::ArrowLeft) || touches.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if touches.pressed(KeyCode::ArrowRight) || touches.pressed(KeyCode::KeyD) { direction.x += 1.0; }
    if touches.pressed(KeyCode::ArrowUp) || touches.pressed(KeyCode::KeyW) { direction.y += 1.0; }
    if touches.pressed(KeyCode::ArrowDown) || touches.pressed(KeyCode::KeyS) { direction.y -= 1.0; }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * vitesse * temps.delta_seconds();
    }
}

fn zoomer_camera(
    mut evenements_molette: EventReader<MouseWheel>,
    mut requete_camera: Query<&mut Transform, With<CameraPrincipale>>,
) {
    let mut transform = requete_camera.single_mut();
    for evenement in evenements_molette.read() {
        let facteur_zoom = 1.1;
        let mut nouvelle_echelle = transform.scale.x;

        if evenement.y > 0.0 {
            nouvelle_echelle /= facteur_zoom;
        } else if evenement.y < 0.0 {
            nouvelle_echelle *= facteur_zoom;
        }

        nouvelle_echelle = nouvelle_echelle.clamp(0.1, 50.0);
        transform.scale = Vec3::splat(nouvelle_echelle);
    }
}