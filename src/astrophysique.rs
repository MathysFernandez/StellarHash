use bevy::prelude::Component;

const PREFIXES_STELLAIRES: [&str; 6] =
    ["Kepler", "Gliese", "Trappist", "Wolf", "Barnard", "Sirius"];

#[derive(Debug, Clone, Copy, PartialEq)]
// Les classes possibles pour chaque étoile
pub enum ClasseSpectrale {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

// Ce composant sera attaché à chaque étoile générée
#[derive(Component, Debug)]
pub struct SystemeStellaire {
    pub nom: String,
    pub classe: ClasseSpectrale,
    // 1.0 = la masse de notre Soleil
    pub masse_solaire: f32,
    pub rayon_solaire: f32,
    pub nb_planetes: u8,
    pub age_milliards_annees: f32,
}

/// Convertit le hachage brut et les coordonnées en données astrophysiques
pub fn generer_caracteristiques(x: i32, y: i32, probabilite: f32) -> SystemeStellaire {
    // Détermination de la Classe Spectrale (Répartition réaliste)
    let classe = if probabilite > 0.998 {
        // Ultra rare
        ClasseSpectrale::O
    } else if probabilite > 0.99 {
        ClasseSpectrale::B
    } else if probabilite > 0.98 {
        ClasseSpectrale::A
    } else if probabilite > 0.97 {
        ClasseSpectrale::F
    } else if probabilite > 0.96 {
        ClasseSpectrale::G
    } else if probabilite > 0.955 {
        ClasseSpectrale::K
    } else {
        // Très commun (Naines rouges)
        ClasseSpectrale::M
    };

    // On multiplie la probabilité et on ne garde que les décimales (ex: 0.9734 * 1000 = 973.4 -> 0.4)
    // Cela nous donne une nouvelle valeur entre 0.0 et 1.0 pour varier les données
    let variation = (probabilite * 1000.0).fract();

    // Pour la masse et l'âge selon le type d'étoile
    let (masse, age) = match classe {
        // (Masse min + variation, Âge min + variation)

        // Vies très courtes
        ClasseSpectrale::O => (16.0 + variation * 74.0, 0.001 + variation * 0.01),
        ClasseSpectrale::B => (2.1 + variation * 13.9, 0.01 + variation * 0.1),
        ClasseSpectrale::A => (1.4 + variation * 0.7, 0.1 + variation * 0.9),
        ClasseSpectrale::F => (1.04 + variation * 0.36, 1.0 + variation * 2.0),

        // Comme notre Soleil
        ClasseSpectrale::G => (0.8 + variation * 0.24, 4.0 + variation * 6.0),
        ClasseSpectrale::K => (0.45 + variation * 0.35, 10.0 + variation * 15.0),

        // Vies quasi éternelles
        ClasseSpectrale::M => (0.08 + variation * 0.37, 20.0 + variation * 80.0),
    };

    // Approximation simple pour calculer le rayon en fonction de la masse
    let rayon = masse.powf(0.8);

    // Génération du Nom (Déterministe basé sur les coordonnées X et Y)
    let index_nom = ((x.abs() + y.abs()) as usize) % PREFIXES_STELLAIRES.len();
    let suffixe_numerique = (x.abs() * 73 + y.abs() * 37) % 9999;
    let nom = format!("{}-{}", PREFIXES_STELLAIRES[index_nom], suffixe_numerique);

    // Nombre de planètes (Favorisé autour des étoiles stables G et K)
    let multiplicateur_planetes = match classe {
        ClasseSpectrale::G | ClasseSpectrale::K => 1.5,
        ClasseSpectrale::O | ClasseSpectrale::B => 0.1,
        _ => 1.0,
    };

    let nb_planetes = ((variation * 10.0) * multiplicateur_planetes) as u8;
    // Limite entre 0 et 8 planètes
    let nb_planetes = nb_planetes.clamp(0, 8);

    SystemeStellaire {
        nom,
        classe,
        masse_solaire: masse,
        rayon_solaire: rayon,
        nb_planetes,
        age_milliards_annees: age,
    }
}

// --- START UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    // ---Start of threshold verification---
    // Verify that the probability thresholds correctly trigger the appropriate spectral class.
    #[test]
    fn test_classe_spectrale_ultra_rare_o() {
        // Classe O
        let systeme = generer_caracteristiques(0, 0, 0.999);
        assert_eq!(systeme.classe, ClasseSpectrale::O);
    }

    #[test]
    fn test_classe_spectrale_commune_b() {
        // Classe B
        let systeme = generer_caracteristiques(0, 0, 0.997);
        assert_eq!(systeme.classe, ClasseSpectrale::B);
    }

    #[test]
    fn test_classe_spectrale_commune_a() {
        // Classe A
        let systeme = generer_caracteristiques(0, 0, 0.987);
        assert_eq!(systeme.classe, ClasseSpectrale::A);
    }

    #[test]
    fn test_classe_spectrale_commune_f() {
        // Classe F
        let systeme = generer_caracteristiques(0, 0, 0.977);
        assert_eq!(systeme.classe, ClasseSpectrale::F);
    }

    #[test]
    fn test_classe_spectrale_commune_g() {
        // Classe G
        let systeme = generer_caracteristiques(0, 0, 0.967);
        assert_eq!(systeme.classe, ClasseSpectrale::G);
    }

    #[test]
    fn test_classe_spectrale_commune_k() {
        // Classe K
        let systeme = generer_caracteristiques(0, 0, 0.957);
        assert_eq!(systeme.classe, ClasseSpectrale::K);
    }

    #[test]
    fn test_classe_spectrale_commune_m() {
        // Classe M
        let systeme = generer_caracteristiques(0, 0, 0.5);
        assert_eq!(systeme.classe, ClasseSpectrale::M);
    }
    // ---End of threshold verification---

    // ---Start floating-point (f32) imprecision.---
    // Using .fract() on floats can introduce slight inaccuracies (e.g., 0.3999999 instead of 0.4).
    // Therefore, avoid testing for exact values; instead, check if the value falls within a specific range.
    #[test]
    fn test_imprecision_flottants_et_bornes() {
        // On génère une étoile de type G (probabilité > 0.96)
        let probabilite = 0.965;
        let systeme = generer_caracteristiques(0, 0, probabilite);

        assert_eq!(systeme.classe, ClasseSpectrale::G);

        // Classe G : masse entre 0.8 et (0.8 + 0.24 = 1.04)
        assert!(systeme.masse_solaire >= 0.8);
        assert!(systeme.masse_solaire <= 1.04);

        // Pour le rayon, on compare l'écart avec l'epsilon de f32
        let rayon_calcule = systeme.masse_solaire.powf(0.8);
        let difference = (systeme.rayon_solaire - rayon_calcule).abs();

        assert!(
            difference < f32::EPSILON,
            "Le rayon ne correspond pas à la formule mathématique"
        );
    }
    // ---End floating-point (f32) imprecision.---

    // ---Start handling signs---
    // Ensure that negative coordinates do not cause the name generation to fail.
    #[test]
    fn test_coordonnees_negatives_generent_noms_valides() {
        let systeme_positif = generer_caracteristiques(15, 30, 0.96);
        let systeme_negatif = generer_caracteristiques(-15, -30, 0.96);

        // Thanks to x.abs() and y.abs(), negative coordinates should yield the same name
        // as their absolute positive equivalents.
        assert_eq!(systeme_positif.nom, systeme_negatif.nom);
    }
    // ---End handling signs---
}
// --- END UNIT TESTS ---
