// TODO: still need to implement differences in set codes between manabox and tappedout

// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, fs::File, io::Write, path::PathBuf, rc::Rc};
use csv::StringRecord;
use serde::Deserialize;

slint::include_modules!();

#[derive(Debug, Clone, Default)]
struct Card {
    quantity: usize,
    name: String,
    set: String,
    var: String,
    foil: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let file_path = Rc::new(RefCell::new(String::new()));

    let ui_weak = ui.as_weak();
    let fp = file_path.clone();
    ui.on_pick_file(move || {
        let mut dialog = rfd::FileDialog::new();
        dialog = dialog.set_title("Select a file");

        if let Some(path) = dialog.pick_file() {
            let mut fp = fp.borrow_mut();
            *fp = String::from(path.to_str().unwrap());
            let ui = ui_weak.unwrap();
            ui.set_file_name(path.to_str().unwrap().into());
        }
    });

    let ui_weak = ui.as_weak();
    let fp = file_path.clone();
    ui.on_parse_file(move || {
        let ui = ui_weak.unwrap();
        let fp = fp.borrow();

        let dialog = rfd::FileDialog::new()
            .add_filter("txt", &["txt"])
            .set_title("Save file");

        if let Some(mut path) = dialog.save_file() {
            // set extension, regardless of user input
            if path.file_name().is_some() {
                path.set_extension("txt");
            } else {
                path.set_file_name("cards");
                path.set_extension("txt");
            }

            // reading file
            let cards = get_cards(&fp).expect("Couldn't get cards!");
            let len = cards.len() as i32;
            ui.set_total_cards(len);

            // write to new file
            let mut nf = File::create(path).expect("Could not create file");
            for (i, card) in cards.iter().enumerate() {
                let mut buf = card.to_string();
                buf.push('\n');
                let _ = nf.write(&buf.as_bytes());
                ui.set_cards_completed(i as i32 +1);
            }
        }
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
    collector_number: String,
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
    #[serde(skip)]
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
#[serde(rename_all = "lowercase")]
enum Foil {
    #[default]
    Normal,
    Foil,
    Etched,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Mythic,
}

fn get_cards(path: &String) -> Result<Vec<Card>, Box<dyn Error>> {
    let file_path = PathBuf::from(path.clone());
    let file = File::open(file_path)?;
    let mut cards: Vec<Card> = vec![];
    let mut bad_cards = vec![];

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    for result in rdr.deserialize::<CsvCard>() {
        match result {
            Ok(r) => {
                if r.set_name.to_ascii_lowercase().contains("token") {
                    println!("Ignoring token: {}", Card::from(r).to_string());
                    continue;
                } else if r.set_code.chars().nth(0) == Some('T') && r.set_code.len() > 3 {
                    println!("Ignoring token: {}", Card::from(r).to_string());
                    continue;
                } else {
                    let card = Card::from(r);
                    println!("{}", card.clone().to_string());
                    cards.push(card);
                }
            }
            Err(e) => {
                bad_cards.push(e);
            },
        }
    }

    println!("{} ERRORS", bad_cards.len());

    for bad_position in bad_cards {
        let mut rec = StringRecord::new();
        if let Some(pos) = bad_position.position() {
            rdr.seek(pos.clone())?;
            let _ = rdr.read_record(&mut rec);
            println!("Bad record: ({}) {rec:?}", bad_position);
        }
    }

    Ok(cards)
}

impl From<CsvCard> for Card {
    fn from(card: CsvCard) -> Self {
        Self {
            quantity: card.quantity,
            name: card.name,
            set: card.set_code,
            var: card.collector_number,
            foil: if card.foil == Foil::Foil { true } else { false },
        }
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        if self.foil {
            format!(
                "{}x {} ({}:{}) *f*",
                self.quantity, self.name, self.set, self.var
            )
        } else {
            format!(
                "{}x {} ({}:{})",
                self.quantity, self.name, self.set, self.var
            )
        }
    }
}
