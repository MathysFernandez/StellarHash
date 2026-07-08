// Fonction de hachage spatial déterministe
// Prend les coordonnées (X, Y) et la graine globale, et retourne un "pourcentage" entre 0.0 et 1.0.
pub fn calculer_hachage_spatial(x: i32, y: i32, graine: u32) -> f32 {
    // Les Constantes Magiques : De grands nombres premiers.
    let prime_x: u32 = 374761393;
    let prime_y: u32 = 668265263;
    
    // Conversion en bits (pour gérer les coordonnées négatives)
    let x_bits = x as u32;
    let y_bits = y as u32;
    
    // Le Mixeur: On utilise l'opérateur XOR (^) et la multiplication avec débordement volontaire
    let mut hash = graine;
    hash ^= x_bits.wrapping_mul(prime_x);
    hash ^= y_bits.wrapping_mul(prime_y);
    
    // L'Avalanche: On décale les bits et on remélange
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(1274126177);
    hash ^= hash >> 16;
    
    // Normalisation: Retourne un flottant entre 0.0 et 1.0
    (hash as f32) / (std::u32::MAX as f32)
}