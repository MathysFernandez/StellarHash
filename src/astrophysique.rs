use bevy::prelude::Component;

const PREFIXES_STELLAIRES: [&str; 6] =
    ["Kepler", "Gliese", "Trappist", "Wolf", "Barnard", "Sirius"];

#[derive(Debug, Clone, Copy, PartialEq)]
// The possible classes for each star
pub enum SpectralClass {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

// This component will be attached to each generated star
#[derive(Component, Debug)]
pub struct StellarSystem {
    pub nom: String,
    pub classe: SpectralClass,
    // 1.0 = the mass of our Sun
    pub masse_solaire: f32,
    pub rayon_solaire: f32,
    pub nb_planetes: u8,
    pub age_milliards_annees: f32,
}

/// Converts the raw hash and coordinates into astrophysical data
pub fn generate_characteristics(x: i32, y: i32, probabilite: f32) -> StellarSystem {
    // Determination of the Spectral Class (Realistic distribution)
    let classe = if probabilite > 0.998 {
        // Ultra rare
        SpectralClass::O
    } else if probabilite > 0.99 {
        SpectralClass::B
    } else if probabilite > 0.98 {
        SpectralClass::A
    } else if probabilite > 0.97 {
        SpectralClass::F
    } else if probabilite > 0.96 {
        SpectralClass::G
    } else if probabilite > 0.955 {
        SpectralClass::K
    } else {
        // Very common (Red dwarfs)
        SpectralClass::M
    };

    // Multiply the probability and keep only the decimal part (e.g., 0.9734 * 1000 = 973.4 -> 0.4)
    // This gives us a new value between 0.0 and 1.0 to vary the data.
    let variation = (probabilite * 1000.0).fract();

    // For mass and age according to the type of star
    let (masse, age) = match classe {
        // (Min. mass + variation, min. age + variation)

        // Very short lives
        SpectralClass::O => (16.0 + variation * 74.0, 0.001 + variation * 0.01),
        SpectralClass::B => (2.1 + variation * 13.9, 0.01 + variation * 0.1),
        SpectralClass::A => (1.4 + variation * 0.7, 0.1 + variation * 0.9),
        SpectralClass::F => (1.04 + variation * 0.36, 1.0 + variation * 2.0),

        // Like our Sun
        SpectralClass::G => (0.8 + variation * 0.24, 4.0 + variation * 6.0),
        SpectralClass::K => (0.45 + variation * 0.35, 10.0 + variation * 15.0),

        // Quasi-eternal lives
        SpectralClass::M => (0.08 + variation * 0.37, 20.0 + variation * 80.0),
    };

    // Simple approximation for calculating the radius based on mass
    let rayon = masse.powf(0.8);

    // Name Generation (Deterministic, based on X and Y coordinates)
    let index_nom = ((x.abs() + y.abs()) as usize) % PREFIXES_STELLAIRES.len();
    let suffixe_numerique = (x.abs() * 73 + y.abs() * 37) % 9999;
    let nom = format!("{}-{}", PREFIXES_STELLAIRES[index_nom], suffixe_numerique);

    // Number of planets (Favored around stable G- and K-type stars)
    let multiplicateur_planetes = match classe {
        SpectralClass::G | SpectralClass::K => 1.5,
        SpectralClass::O | SpectralClass::B => 0.1,
        _ => 1.0,
    };

    let nb_planetes = ((variation * 10.0) * multiplicateur_planetes) as u8;
    // Limit between 0 and 8 planets
    let nb_planetes = nb_planetes.clamp(0, 8);

    StellarSystem {
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
        let systeme = generate_characteristics(0, 0, 0.999);
        assert_eq!(systeme.classe, SpectralClass::O);
    }

    #[test]
    fn test_classe_spectrale_commune_b() {
        // Classe B
        let systeme = generate_characteristics(0, 0, 0.997);
        assert_eq!(systeme.classe, SpectralClass::B);
    }

    #[test]
    fn test_classe_spectrale_commune_a() {
        // Classe A
        let systeme = generate_characteristics(0, 0, 0.987);
        assert_eq!(systeme.classe, SpectralClass::A);
    }

    #[test]
    fn test_classe_spectrale_commune_f() {
        // Classe F
        let systeme = generate_characteristics(0, 0, 0.977);
        assert_eq!(systeme.classe, SpectralClass::F);
    }

    #[test]
    fn test_classe_spectrale_commune_g() {
        // Classe G
        let systeme = generate_characteristics(0, 0, 0.967);
        assert_eq!(systeme.classe, SpectralClass::G);
    }

    #[test]
    fn test_classe_spectrale_commune_k() {
        // Classe K
        let systeme = generate_characteristics(0, 0, 0.957);
        assert_eq!(systeme.classe, SpectralClass::K);
    }

    #[test]
    fn test_classe_spectrale_commune_m() {
        // Classe M
        let systeme = generate_characteristics(0, 0, 0.5);
        assert_eq!(systeme.classe, SpectralClass::M);
    }
    // ---End of threshold verification---

    // ---Start floating-point (f32) imprecision.---
    // Using .fract() on floats can introduce slight inaccuracies (e.g., 0.3999999 instead of 0.4).
    // Therefore, avoid testing for exact values; instead, check if the value falls within a specific range.
    #[test]
    fn test_imprecision_flottants_et_bornes() {
        // On génère une étoile de type G (probabilité > 0.96)
        let probabilite = 0.965;
        let systeme = generate_characteristics(0, 0, probabilite);

        assert_eq!(systeme.classe, SpectralClass::G);

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
        let systeme_positif = generate_characteristics(15, 30, 0.96);
        let systeme_negatif = generate_characteristics(-15, -30, 0.96);

        // Thanks to x.abs() and y.abs(), negative coordinates should yield the same name
        // as their absolute positive equivalents.
        assert_eq!(systeme_positif.nom, systeme_negatif.nom);
    }
    // ---End handling signs---
}
// --- END UNIT TESTS ---
