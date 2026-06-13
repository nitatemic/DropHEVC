# DropHEVC

DropHEVC est une application de bureau multiplateforme (Windows, macOS, Linux) permettant de convertir des vidéos en HEVC (H.265) de manière optimisée pour les écosystèmes Apple/iOS.

## Choix Techniques

*   **Tauri 2** : Framework léger et performant offrant un backend natif en Rust et un frontend web.
*   **Backend (Rust)** : Orchestration des tâches, gestion des processus système (FFmpeg), parsing en temps réel de la progression pour une interface fluide sans bloquage.
*   **Frontend (Vanilla JS, HTML, CSS)** : Aucune dépendance lourde, pas de TypeScript pour rester simple et modulaire, un design system personnalisé "Glassmorphism Dark" pour un rendu premium.
*   **FFmpeg** : Stratégie de double disponibilité. L'application vérifie si FFmpeg est présent. S'il ne l'est pas, un bouton permet de le télécharger automatiquement depuis GitHub Releases pour l'embarquer de manière isolée dans le dossier de données de l'application.

## Structure du Projet

```text
DropHEVC/
├── package.json             # Dépendances Node.js (Tauri CLI, plugins)
├── src/                     # Frontend Vanilla JS
│   ├── index.html           # Structure de l'interface
│   ├── main.js              # Logique UI, gestion de file, événements drag & drop
│   └── styles.css           # Thème premium sombre
└── src-tauri/               # Backend Rust
    ├── Cargo.toml           # Dépendances Rust (tokio, reqwest, zip, etc.)
    ├── tauri.conf.json      # Configuration Tauri 2
    └── src/
        ├── main.rs          # Point d'entrée
        ├── lib.rs           # Initialisation Tauri et Plugins
        ├── commands.rs      # Commandes IPC appelées par le frontend
        └── ffmpeg.rs        # Logique de détection, téléchargement et lancement de FFmpeg
```

## Prérequis

Pour développer ou compiler cette application, vous devez avoir installé :
1.  **Node.js** (v18+)
2.  **Rust** (via rustup) et Cargo.

## Installation et Lancement en Dev

1. Clonez ce dépôt.
2. Installez les dépendances npm :
   ```bash
   npm install
   ```
3. Lancez l'environnement de développement :
   ```bash
   npm run tauri dev
   ```

## Build Desktop

Pour créer l'exécutable final (exe, dmg, appimage) :

```bash
npm run tauri build
```
Les fichiers de sortie seront dans `src-tauri/target/release/bundle/`.

## Où modifier les presets FFmpeg ?

Les paramètres d'encodage (le "preset") sont définis dans le backend Rust, dans le fichier `src-tauri/src/commands.rs`, au sein de la fonction `convert_video`. 
Cherchez la construction de la commande :
```rust
let mut child = Command::new(&ffmpeg_path)
    .arg("-y")
    .arg("-i")
    .arg(&input_path)
    .arg("-c:v").arg("libx265")
    .arg("-tag:v").arg("hvc1")
    .arg("-preset").arg("medium")
    .arg("-crf").arg("23")
    .arg("-c:a").arg("copy")
    ...
```
Vous pouvez modifier ces arguments pour ajouter du CRF personnalisé, changer de preset, ou ajouter une accélération matérielle (comme `hevc_nvenc` ou `hevc_videotoolbox`).
