# Bitcoin Vanity Address Generator (SegWit)

Un générateur d'adresses Bitcoin vanity avec interface graphique, optimisé pour les adresses SegWit (bc1).

![Bitcoin Vanity Address Generator](https://raw.githubusercontent.com/bitcoin/bitcoin/master/share/pixmaps/bitcoin128.png)

## Fonctionnalités

- ✨ Génération d'adresses Bitcoin SegWit personnalisées
- 🚀 Utilisation du multi-threading pour des performances optimales
- 🎨 Interface graphique moderne et intuitive
- 💾 Sauvegarde sécurisée des adresses générées
- 📊 Statistiques en temps réel
- 🌙 Thème sombre par défaut

## Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (édition 2021 ou supérieure)
- Un système d'exploitation compatible (Windows, Linux, macOS)

## Installation

1. Clonez le dépôt :
```bash
git clone https://github.com/DasJu64/BTC_Vanity_Segwit.git
cd BTC_Vanity_Segwit
```

2. Compilez le projet :
```bash
cargo build --release
```

3. Exécutez le programme :
```bash
cargo run --release
```

## Utilisation

1. Lancez l'application
2. Entrez le préfixe souhaité pour votre adresse (après "bc1")
3. Ajustez le nombre de threads selon votre CPU
4. Cliquez sur "Démarrer la recherche"
5. Une fois une adresse trouvée, sauvegardez-la dans un fichier texte

## Avertissements de Sécurité

⚠️ **IMPORTANT** :
- Gardez vos clés privées en sécurité
- Ne partagez jamais vos clés privées
- Vérifiez toujours l'adresse générée avant utilisation
- Il est recommandé d'utiliser ce générateur hors ligne pour plus de sécurité

## Dépendances Principales

- `bitcoin` v0.31.0 - Bibliothèque Bitcoin
- `secp256k1` v0.28.0 - Cryptographie
- `eframe` v0.24.0 - Interface graphique
- `rayon` v1.8.0 - Multi-threading
- `rfd` v0.11.4 - Dialogues de fichiers natifs

## Performance

La vitesse de génération dépend de :
- La puissance de votre CPU
- Le nombre de threads utilisés
- La longueur du préfixe recherché

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
1. Fork le projet
2. Créer une branche pour votre fonctionnalité
3. Commit vos changements
4. Push sur votre fork
5. Ouvrir une Pull Request

## Licence et Usage

Ce projet est mis à la disposition de tous, librement et gratuitement. Vous pouvez l'utiliser, le modifier et le distribuer comme bon vous semble. 

⚠️ Note : Ce logiciel est fourni "tel quel" et peut présenter des bugs, Merci de vérifier la validité de vos clés !

---
