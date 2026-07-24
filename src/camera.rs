use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

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
    commands.spawn((Camera2dBundle::default(), CameraPrincipale));
}

fn deplacer_camera(
    touches: Res<ButtonInput<KeyCode>>,
    temps: Res<Time>,
    mut requete_camera: Query<&mut Transform, With<CameraPrincipale>>,
) {
    let mut transform = requete_camera.single_mut();
    let mut vitesse = 500.0 * transform.scale.x;

    if touches.pressed(KeyCode::ShiftLeft) {
        vitesse *= 2.0;
    }

    let mut direction = Vec3::ZERO;
    if touches.pressed(KeyCode::ArrowLeft) || touches.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if touches.pressed(KeyCode::ArrowRight) || touches.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if touches.pressed(KeyCode::ArrowUp) || touches.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if touches.pressed(KeyCode::ArrowDown) || touches.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

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

// --- START UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use std::time::Duration;

    // ---Start camera initialization test---
    #[test]
    fn test_initialiser_camera_cree_entite() {
        // Creating an empty Bevy application
        let mut app = App::new();

        // We add our system
        app.add_systems(Startup, initialiser_camera);

        // Run the application for one frame to execute the Startup
        app.update();

        // Verify that the camera has been successfully instantiated with its components
        let mut requete = app
            .world_mut()
            .query_filtered::<&Transform, With<CameraPrincipale>>();

        // if there isn't exactly one camera, this will panic (which is what we want in a test)
        let _transform = requete.single(app.world());
    }
    // ---End camera initialization test---

    // ---Start camera movement test---
    #[test]
    fn test_deplacer_camera_vers_la_droite() {
        let mut app = App::new();

        // Initialize the resources required by the system
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<Time>();
        // We manually instantiate our camera
        let camera_entite = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), CameraPrincipale))
            .id();

        app.add_systems(Update, deplacer_camera);

        // Simulate pressing the D key (right)
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyD);

        // Simulate the passage of time (e.g., 0.1 seconds)
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(Duration::from_secs_f32(0.1));

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();

        // The speed is 500.0 * 1.0 (scale); over 0.1 sec, we should be at X = 50.0
        assert_eq!(transform.translation.x, 50.0);
        assert_eq!(transform.translation.y, 0.0);
    }
    // ---End camera movement test---

    // ---Start camera Shift movement test---
    #[test]
    fn test_deplacer_camera_avec_multiplicateur_shift() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>();

        // On ajoute l'annotation ": Time" pour guider le compilateur
        let mut time: Time = Time::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);

        let camera_entite = app
            .world_mut()
            .spawn((Transform::default(), CameraPrincipale))
            .id();
        app.add_systems(Update, deplacer_camera);

        // On simule l'appui sur D (droite) ET sur Shift Gauche
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyD);
        input.press(KeyCode::ShiftLeft);

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();
        // Vitesse normale = 50.0, avec Shift (* 2.0) = 100.0
        assert_eq!(transform.translation.x, 100.0);
    }
    // ---End camera Shift movement test---

    // ---Start top and bottom zoom test ---
    #[test]
    fn test_zoomer_camera_molette_haut_et_bas() {
        let mut app = App::new();

        // Le système a besoin de lire des évènements MouseWheel
        app.add_event::<MouseWheel>();

        let camera_entite = app
            .world_mut()
            .spawn((Transform::default(), CameraPrincipale))
            .id();
        app.add_systems(Update, zoomer_camera);

        // --- ZOOM IN (y > 0.0) ---
        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window: Entity::PLACEHOLDER, // Entité factice pour Bevy 0.13+
        });

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();
        // Zoom IN = on divise par 1.1. L'échelle par défaut est 1.0. (1.0 / 1.1 = ~0.909)
        assert!(transform.scale.x < 1.0);

        // --- ZOOM OUT (y < 0.0) ---
        // On renvoie un évènement pour dézoomer
        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window: Entity::PLACEHOLDER,
        });

        app.update();

        let transform_final = app.world().get::<Transform>(camera_entite).unwrap();
        // On devrait être revenus à environ 1.0 (0.909 * 1.1)
        let difference = (transform_final.scale.x - 1.0).abs();
        assert!(difference < 0.0001, "L'échelle devrait être revenue à 1.0");
    }
    // ---End top and bottom zoom test ---

    // ---Start of zoom limit test---
    #[test]
    fn test_zoomer_camera_respecte_les_limites() {
        let mut app = App::new();
        app.add_event::<MouseWheel>();

        // On instancie une caméra déjà à la limite maximale autorisée (50.0)
        let camera_entite = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(50.0)), CameraPrincipale))
            .id();

        app.add_systems(Update, zoomer_camera);

        // On essaye de zoomer OUT (agrandir la caméra, ce qui fait y < 0.0)
        let mut evenements = app.world_mut().resource_mut::<Events<MouseWheel>>();
        evenements.send(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window: Entity::PLACEHOLDER,
        });

        app.update();

        let transform = app.world().get::<Transform>(camera_entite).unwrap();

        // Le scale ne doit pas avoir dépassé 50.0 grâce à votre clamp()
        assert_eq!(transform.scale.x, 50.0);
    }
    // ---End of zoom limit test---
}
// --- END UNIT TESTS ---
