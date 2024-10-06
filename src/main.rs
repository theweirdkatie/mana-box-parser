// TODO: app takes WAY too long to transform thousands of lines of text for the text editor
// TODO: serde still doesn't like quotations

// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{default, error::Error};

use serde::Deserialize;

slint::include_modules!();

#[derive(Debug, Clone, Default)]
struct Card {
    quantity: usize,
    name: String,
    set: String,
    var: usize,
    foil: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut file_path = String::new();

    let mut ui_weak = ui.as_weak();
    ui.on_pick_file(move || {
        let mut dialog = rfd::FileDialog::new();
        dialog = dialog.set_title("Select a file");

        if let Some(path) = dialog.pick_file() {
            file_path = String::from(path.to_str().unwrap());
            let ui = ui_weak.unwrap();
            ui.set_file_name(path.to_str().unwrap().into());
        }
    });

    let mut ui_weak = ui.as_weak();
    ui.on_parse_file(move || {
        let ui = ui_weak.unwrap();
        let file_path = std::path::PathBuf::from(ui.get_file_name().to_string());
        let cards = get_cards(file_path).expect("Couldn't get cards!");
        let mut text = String::new();
        for card in cards {
            text += &card.to_string();
            text.push('\n');
        }
        ui.set_parsed_file(text.into());
    });

    ui.run()?;

    Ok(())
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CsvCard {
    #[serde(alias = "Binder Name")]
    binder_name: String,
    #[serde(alias = "Binder Type")]
    binder_type: String,
    #[serde(alias = "Name")]
    name: String,
    #[serde(alias = "Set code")]
    set_code: String,
    #[serde(alias = "Set name")]
    set_name: String,
    #[serde(alias = "Collector number")]
    collector_number: usize,
    #[serde(alias = "Foil")]
    foil: Foil,
    #[serde(alias = "Rarity")]
    rarity: Rarity,
    #[serde(alias = "Quantity")]
    quantity: usize,
    #[serde(alias = "ManaBox")]
    manabox: usize,
    #[serde(alias = "Scryfall ID")]
    scryfall_id: String,
    #[serde(alias = "Purchase price")]
    purchase_price: f32,
    #[serde(alias = "Misprint")]
    misprint: bool,
    #[serde(alias = "Altered")]
    altered: bool,
    #[serde(alias = "Condition")]
    condition: String,
    #[serde(alias = "Language")]
    language: String,
    #[serde(alias = "Purchase price currency")]
    purchase_price_currency: String,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all="lowercase")]
enum Foil {
    #[default]
    Normal,
    Foil,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all="lowercase")]
enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Mythic,
}

fn get_cards(file_path: std::path::PathBuf) -> Result<Vec<Card>, Box<dyn Error>> {
    let file = std::fs::File::open(file_path)?;
    let mut cards: Vec<Card> = vec![];

    let mut rdr = csv::ReaderBuilder::new().has_headers(true).quoting(false).from_reader(file);

    for result in rdr.deserialize::<CsvCard>() {
        if cards.len() >= 100 {
            break;
        }
        match result {
            Ok(r) => {
                let card = Card::from(r);
                println!("{}", card.clone().to_string());
                cards.push(card);
            }
            Err(e) => println!("Error! Couldn't parse: {}", e),
        }
    }
    Ok(cards)
}

impl From<CsvCard> for Card {
    fn from(card: CsvCard) -> Self {
        Self { quantity: card.quantity, name: card.name, set: card.set_code, var: card.collector_number, foil: if card.foil == Foil::Foil { true } else { false } }
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        if self.foil {
            format!("{}x {} ({}:{}) *f*", self.quantity, self.name, self.set, self.var)
        } else {
            format!("{}x {} ({}:{})", self.quantity, self.name, self.set, self.var)
        }
    }
}