# StellarHash

StellarHash est une simulation spatiale 2D mettant en œuvre des techniques avancées de génération procédurale déterministe. Développé en Rust avec le moteur Bevy, ce projet contourne les limites de la mémoire (RAM) liées à la création d'environnements massifs en utilisant une approche Just-In-Time.

Au lieu de pré-calculer et de stocker les données d'une galaxie, StellarHash s'appuie sur une fonction de hachage de coordonnées (X, Y) couplée à une graine globale (Seed) pour générer les systèmes stellaires à la volée lors des déplacements de la caméra. L'utilisation du paradigme ECS (Entity-Component-System) permet de maximiser les performances multi-threadées lors de l'instanciation et de la destruction des entités cosmiques.


## Use:
```bash
cargo run --release
```
