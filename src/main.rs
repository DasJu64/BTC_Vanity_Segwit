use bitcoin::{Address, Network, PublicKey, secp256k1::Secp256k1};
use eframe::egui;
use rayon::prelude::*;
use secp256k1::rand::rngs::OsRng;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Instant;
use std::sync::mpsc::Receiver;
use rfd::FileDialog;

fn format_number(n: u64) -> String {
    let mut s = String::new();
    let n_str = n.to_string();
    let mut count = 0;
    
    for c in n_str.chars().rev() {
        if count > 0 && count % 3 == 0 {
            s.insert(0, ' ');
        }
        s.insert(0, c);
        count += 1;
    }
    s
}

#[derive(Clone)]
struct Stats {
    attempts_per_second: f64,
    elapsed_seconds: u64,
    total_attempts: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            attempts_per_second: 0.0,
            elapsed_seconds: 0,
            total_attempts: 0,
        }
    }
}

struct VanityApp {
    prefix: String,
    num_threads: usize,
    running: Arc<AtomicBool>,
    attempts: Arc<AtomicUsize>,
    found_addresses: Vec<(String, String, String)>, // (address, private_key, public_key)
    start_time: Option<Instant>,
    stats: Stats,
    save_filename: String,
    receiver: Option<Receiver<(String, String, String)>>,
}

impl Default for VanityApp {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            num_threads: num_cpus::get(),
            running: Arc::new(AtomicBool::new(false)),
            attempts: Arc::new(AtomicUsize::new(0)),
            found_addresses: Vec::new(),
            start_time: None,
            stats: Stats::default(),
            save_filename: format!("bitcoin_vanity_{}", 
                chrono::Local::now().format("%Y%m%d_%H%M%S")),
            receiver: None,
        }
    }
}

impl eframe::App for VanityApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Vérifier les résultats du thread de recherche
        if let Some(rx) = &self.receiver {
            if let Ok((addr, wif, pubkey)) = rx.try_recv() {
                self.found_addresses.push((addr, wif, pubkey));
                self.running.store(false, Ordering::Relaxed);
            }
        }

        if self.running.load(Ordering::Relaxed) {
            self.update_stats();
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Configuration section
                ui.group(|ui| {
                    ui.heading("Configuration");
                    ui.label("Pour les adresses SegWit, le format commence toujours par 'bc1'");
                    ui.horizontal(|ui| {
                        ui.label("Préfixe recherché:");
                        if !self.running.load(Ordering::Relaxed) {
                            let response = ui.add(egui::TextEdit::singleline(&mut self.prefix)
                                .desired_width(200.0));
                            if response.changed() {
                                self.prefix = self.prefix.to_lowercase();
                            }
                            response.request_focus();
                        } else {
                            ui.label(&self.prefix);
                        }
                    });
                    if !self.prefix.is_empty() {
                        let example = format!("Exemple: bc1{}... ou bc1q{}...", self.prefix, self.prefix);
                        ui.label(example);
                    }

                    ui.horizontal(|ui| {
                        ui.label("Nombre de threads:");
                        if !self.running.load(Ordering::Relaxed) {
                            ui.add(egui::Slider::new(&mut self.num_threads, 1..=num_cpus::get())
                                .text("threads"));
                        } else {
                            ui.label(self.num_threads.to_string());
                        }
                    });
                });

                ui.add_space(10.0);

                // Control buttons
                ui.horizontal(|ui| {
                    if !self.running.load(Ordering::Relaxed) {
                        if ui.button("🔍 Démarrer la recherche").clicked() && !self.prefix.is_empty() {
                            self.start_search();
                        }
                    } else {
                        if ui.button("⏹️ Arrêter").clicked() {
                            self.running.store(false, Ordering::Relaxed);
                        }
                    }
                });

                ui.add_space(10.0);

                // Statistics section
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ui.heading("Statistiques");
                    ui.label(format!(
                        "Tentatives: {} | Vitesse: {} addr/s | Temps écoulé: {}s",
                        format_number(self.stats.total_attempts),
                        format_number(self.stats.attempts_per_second as u64),
                        self.stats.elapsed_seconds
                    ));
                });

                // Results section
                if !self.found_addresses.is_empty() {
                    ui.add_space(10.0);
                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        ui.heading("Résultat trouvé! 🎉");
                        let addresses = self.found_addresses.clone();
                        for (address, private_key, public_key) in &addresses {
                            ui.add_space(5.0);
                            ui.label(format!("✓ Adresse Bitcoin: {}", address));
                            ui.label(format!("🔑 Clé privée (WIF): {}", private_key));
                            ui.label(format!("🔐 Clé publique: {}", public_key));
                            
                            ui.add_space(5.0);
                            
                            let clicked = ui.horizontal(|ui| {
                                ui.label("Nom du fichier: ");
                                ui.text_edit_singleline(&mut self.save_filename);
                                ui.label(".txt");
                                ui.button("💾 Sauvegarder").clicked()
                            }).inner;
                            
                            if clicked {
                                self.save_to_file(address, private_key, public_key);
                                self.save_filename = format!("bitcoin_vanity_{}", 
                                    chrono::Local::now().format("%Y%m%d_%H%M%S"));
                            }
                        }
                    });
                }
            });
        });
    }
}

impl VanityApp {
    fn start_search(&mut self) {
        let prefix = self.prefix.clone();
        let running = self.running.clone();
        let attempts = self.attempts.clone();
        let num_threads = self.num_threads;
        
        running.store(true, Ordering::Relaxed);
        attempts.store(0, Ordering::Relaxed);
        self.start_time = Some(Instant::now());
        
        // Création d'un canal pour envoyer les résultats
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Thread de recherche
        std::thread::spawn(move || {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();
            
            pool.install(|| {
                let secp = Secp256k1::new();
                
                while running.load(Ordering::Relaxed) {
                    if let Some((addr, wif, pubkey)) = (0..1000).into_par_iter()
                        .find_map_first(|_| {
                            if !running.load(Ordering::Relaxed) {
                                return None;
                            }
                            
                            let mut rng = OsRng;
                            let (secret_key, public_key) = secp.generate_keypair(&mut rng);
                            let address = Address::p2wpkh(
                                &PublicKey::new(public_key),
                                Network::Bitcoin,
                            ).expect("Failed to create address");
                            
                            let addr_str = address.to_string();
                            attempts.fetch_add(1, Ordering::Relaxed);
                            
                            if addr_str.contains(&prefix) {
                                let wif = bitcoin::PrivateKey::new(secret_key, Network::Bitcoin)
                                    .to_wif();
                                Some((addr_str, wif, public_key.to_string()))
                            } else {
                                None
                            }
                        }) {
                        // Envoi du résultat trouvé
                        let _ = tx.send((addr, wif, pubkey));
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            });
        });

        // Stockage du récepteur pour récupérer les résultats
        self.receiver = Some(rx);
    }
    
    fn update_stats(&mut self) {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            let attempts = self.attempts.load(Ordering::Relaxed) as f64;
            self.stats.attempts_per_second = attempts / elapsed.as_secs_f64();
            self.stats.elapsed_seconds = elapsed.as_secs();
            self.stats.total_attempts = attempts as u64;
        }
    }
    
    fn save_to_file(&self, address: &str, private_key: &str, public_key: &str) {
        let file_path = if let Some(file_path) = FileDialog::new()
            .set_title("Sauvegarder l'adresse Bitcoin")
            .set_file_name(&format!("{}.txt", self.save_filename))
            .add_filter("Fichier texte", &["txt"])
            .save_file() {
            file_path
        } else {
            return; // L'utilisateur a annulé la sélection
        };
        
        if let Ok(mut file) = File::create(file_path) {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let content = format!(
                "Bitcoin Vanity Address\n\
                 ========================\n\n\
                 Adresse Bitcoin: {}\n\
                 Clé privée (WIF): {}\n\
                 Clé publique: {}\n\n\
                 Statistiques\n\
                 -----------\n\
                 Préfixe recherché: {}\n\
                 Nombre de threads: {}\n\
                 Tentatives totales: {}\n\
                 Temps de recherche: {}s\n\
                 Vitesse moyenne: {} addr/s\n\n\
                 Généré le: {}\n\
                 ========================\n",
                address, private_key, public_key,
                self.prefix, self.num_threads,
                format_number(self.stats.total_attempts),
                self.stats.elapsed_seconds,
                format_number(self.stats.attempts_per_second as u64),
                timestamp
            );
            
            if let Err(e) = file.write_all(content.as_bytes()) {
                eprintln!("Erreur lors de l'écriture du fichier: {}", e);
            }
        }
    }
}

fn main() {
    env_logger::init();
    
    let mut options = eframe::NativeOptions::default();
    options.viewport.inner_size = Some(egui::vec2(800.0, 600.0));
    options.viewport.min_inner_size = Some(egui::vec2(600.0, 400.0));
    options.default_theme = eframe::Theme::Dark;
    
    eframe::run_native(
        "Générateur d'adresses Bitcoin Vanity - SegWit",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(VanityApp::default())
        }),
    ).expect("Failed to start application");
}
