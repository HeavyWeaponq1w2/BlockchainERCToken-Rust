use color_eyre::{owo_colors::OwoColorize, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hex;
use rand::{rngs::OsRng, RngCore, TryRngCore};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Modifier, Style},
    style::Color,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use sp_core::{
    crypto::{Pair, SecretString, Ss58AddressFormat, Ss58Codec},
    sr25519::{Pair as Sr25519Pair, Public},
    Encode,
};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Has the button been pressed?
    button_pressed: bool,
    /// List of seeds loaded from file
    seeds: Vec<[u8; 32]>,
    /// Last time seeds were checked
    last_check: Option<Instant>,
    /// Path to the keys file
    keys_path: String,
    /// Last known modification time of the keys file
    last_modified: Option<std::time::SystemTime>,
}

#[derive(Debug)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub address: String,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            button_pressed: false,
            seeds: Vec::new(),
            last_check: None,
            keys_path: "./keys.txt".to_string(),
            last_modified: None,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.load_seeds()?;

        while self.running {
            self.check_for_updates()?;

            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(100))? {
                self.handle_crossterm_events()?;
            }
        }
        Ok(())
    }

    fn check_for_updates(&mut self) -> Result<()> {
        let now = Instant::now();

        if let Some(last_check) = self.last_check {
            if now.duration_since(last_check) < Duration::from_millis(100) {
                return Ok(());
            }
        }

        self.last_check = Some(now);

        let path = Path::new(&self.keys_path);
        if path.exists() {
            match path.metadata() {
                Ok(metadata) => match metadata.modified() {
                    Ok(modified_time) => {
                        if self.last_modified.is_none() || self.last_modified != Some(modified_time)
                        {
                            self.last_modified = Some(modified_time);
                            self.load_seeds()?;
                        }
                    }
                    Err(e) => eprintln!("Error getting modified time: {}", e),
                },
                Err(e) => eprintln!("Error getting metadata: {}", e),
            }
        }

        Ok(())
    }

    fn generate_random_wallet() -> (Sr25519Pair, String, [u8; 32]) {
        let mut seed = [0u8; 32];
        let mut rng = OsRng;
        let _ = rng.try_fill_bytes(&mut seed);

        let pair = Sr25519Pair::from_seed(&seed);

        let address = pair.public().to_ss58check();

        (pair, address, seed)
    }

    fn save_wallet_to_file(file_path: &str, seed: &[u8; 32]) -> Result<(), std::io::Error> {
        let path = Path::new(file_path);

        let mut file = OpenOptions::new().append(true).create(true).open(path)?;

        let seed_hex = hex::encode(seed);

        writeln!(file, "{}", seed_hex)?;

        Ok(())
    }

    fn load_wallets_from_file(file_path: &str) -> Result<Vec<[u8; 32]>, std::io::Error> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;

        let reader = BufReader::new(file);
        let mut seeds: Vec<[u8; 32]> = Vec::new();

        for line in reader.lines() {
            let line = line?;

            if line.trim().is_empty() {
                continue;
            }

            let seed_bytes = hex::decode(line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            if seed_bytes.len() != 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Seed must be 32 bytes",
                ));
            }

            let mut seed = [0u8; 32];
            seed.copy_from_slice(&seed_bytes[..32]);

            seeds.push(seed);
        }

        Ok(seeds)
    }

    fn load_seeds(&mut self) -> Result<()> {
        match Self::load_wallets_from_file(&self.keys_path) {
            Ok(seeds) => {
                self.seeds = seeds;
                Ok(())
            }
            Err(e) => {
                eprintln!("Error loading seeds: {}", e);
                Err(e.into())
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let mut lines = Vec::new();

        if self.seeds.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("No wallets found. ", Style::default().fg(Color::Yellow)),
                Span::raw("Press 'A' to generate one!"),
            ]));
        } else {
            for (i, seed) in self.seeds.iter().enumerate() {
                let pair = Sr25519Pair::from_seed(&seed);
                let address = pair.public().to_ss58check();
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("Wallet {}: ", i + 1),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw(address),
                ]));
            }
        }

        let text = Text::from(lines);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(frame.size());

        let title = Line::from(vec![
            Span::styled("Substrate ", Style::default().fg(Color::Green)),
            Span::styled("Wallet ", Style::default().fg(Color::Yellow)),
            Span::styled("Manager", Style::default().fg(Color::Blue)),
        ])
        .centered();

        let button_text = if self.button_pressed {
            "New wallet generated! Press 'A' to generate another one."
        } else {
            "Press 'A' to generate a new wallet"
        };

        let button = Paragraph::new(button_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default()),
            )
            .style(Style::default().fg(Color::Green))
            .centered();

        frame.render_widget(button.block(Block::bordered().title(title)), layout[0]);

        let wallet_count = self.seeds.len();
        let wallet_title = format!("Wallets ({} total)", wallet_count);

        let seed_paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(wallet_title))
            .style(Style::default());

        frame.render_widget(seed_paragraph, layout[1]);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('a')) => self.press_button(),
            // Add other key handlers here.
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn press_button(&mut self) {
        self.button_pressed = true;
        let (_, address, seed) = Self::generate_random_wallet();

        if let Err(e) = Self::save_wallet_to_file(&self.keys_path, &seed) {
            eprintln!("Failed to save wallet: {}", e);
        }
    }
}
