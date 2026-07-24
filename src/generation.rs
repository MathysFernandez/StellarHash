// Fonction de hachage spatial déterministe
// Prend les coordonnées (X, Y) et la graine globale, et retourne un "pourcentage" entre 0.0 et 1.0.
pub fn calculer_hachage_spatial(x: i32, y: i32, graine: u32) -> f32 {
    // Les Constantes Magiques: De grands nombres premiers.
    let prime_x: u32 = 374761393;
    let prime_y: u32 = 668265263;

    // Conversion en bits (pour gérer les coordonnées négatives)
    let x_bits = x as u32;
    let y_bits = y as u32;

    // Le Mixeur
    let mut hash = graine;
    hash ^= x_bits.wrapping_mul(prime_x);

    // CORRECTION : On décale les bits circulairement pour casser la symétrie
    // avant d'appliquer le XOR sur Y. Cela détruit l'effet miroir des signes.
    hash = hash.rotate_left(17);

    hash ^= y_bits.wrapping_mul(prime_y);

    // L'Avalanche: On décale les bits et on remélange
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(1274126177);
    hash ^= hash >> 16;

    // Normalisation: Retourne un flottant entre 0.0 et 1.0
    (hash as f32) / (std::u32::MAX as f32)
}

// --- START UNIT TESTS ---
mod tests {
    use super::*;

    // ---Start hashing test with limits---
    #[test]
    fn test_hachage_dans_les_limites() {
        // Verify that the generated hash is always between 0.0 and 1.0 inclusive
        let points_a_tester = [
            (0, 0, 12345),
            (10, 20, 9876),
            (-150, -300, 42),
            (std::i32::MAX, std::i32::MIN, std::u32::MAX),
        ];

        for (x, y, graine) in points_a_tester {
            let resultat = calculer_hachage_spatial(x, y, graine);

            assert!(
                resultat >= 0.0,
                "Le hachage doit être supérieur ou égal à 0.0"
            );
            assert!(
                resultat <= 1.0,
                "Le hachage doit être inférieur ou égal à 1.0"
            );
        }
    }
    // ---End hashing test with limits---

    // ---Start of deterministic hashing tests---
    #[test]
    fn test_hachage_deterministe() {
        // Ensure that calling the function multiple times with the exact same parameters yields the exact same float
        let x = 42;
        let y = -84;
        let graine = 9999;

        let premier_appel = calculer_hachage_spatial(x, y, graine);
        let deuxieme_appel = calculer_hachage_spatial(x, y, graine);

        // We can safely use assert_eq! here because we expect the exact same bit-for-bit float output
        assert_eq!(
            premier_appel, deuxieme_appel,
            "Le hachage doit être strictement déterministe pour les mêmes entrées"
        );
    }
    // ---Start of deterministic hashing tests---

    // ---Start the test of hashing differences with different seeds---
    #[test]
    fn test_graine_modifie_resultat() {
        // Check that altering the seed changes the output hash for the same coordinates
        let x = 100;
        let y = 100;

        let hachage_graine_1 = calculer_hachage_spatial(x, y, 1000);
        let hachage_graine_2 = calculer_hachage_spatial(x, y, 1001);

        assert_ne!(
            hachage_graine_1, hachage_graine_2,
            "Deux graines différentes doivent produire des hachages distincts"
        );
    }
    // ---End the test of hashing differences with different seeds---

    // ---Start of the test comparing hashing differences with their negative values---
    #[test]
    fn test_coordonnees_differentes_et_negatives() {
        // Validate that negative coordinates don't crash and that symmetric coordinates do not produce collisions
        let graine = 12345;

        let hachage_positif = calculer_hachage_spatial(10, 10, graine);
        let hachage_negatif = calculer_hachage_spatial(-10, -10, graine);
        let hachage_mixte = calculer_hachage_spatial(-10, 10, graine);

        assert_ne!(
            hachage_positif, hachage_negatif,
            "Les coordonnées symétriques (10,10) et (-10,-10) ne doivent pas entrer en collision"
        );
        assert_ne!(
            hachage_positif, hachage_mixte,
            "Chaque quadrant de la grille spatiale doit générer des valeurs uniques"
        );
    }
    // ---End of the test comparing hashing differences with their negative values---
}
// --- END UNIT TESTS ---
