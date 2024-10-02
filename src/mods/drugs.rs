use tokio::io::AsyncWriteExt;
use rand::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::Config;

pub struct Drugs {
    pub fat: bool,
    pub stats: Arc<Mutex<Stats>>,
}

pub struct Stats {
    pub hits: usize,
    pub sips: usize,
    pub chugged: usize,
    pub smoked: usize,
    pub toked: usize,
    pub chain: usize,
    pub drag: f64,
}

impl Drugs {
    pub fn new() -> Self {
        Drugs {
            fat: false,
            stats: Arc::new(Mutex::new(Stats {
                hits: 25,
                sips: 8,
                chugged: 0,
                smoked: 0,
                toked: 0,
                chain: 0,
                drag: 0.0,
            })),
        }
    }

    fn color(msg: &str, foreground: &str, background: Option<&str>) -> String {
        match background {
            Some(bg) => format!("\x03{},{}{}\x0f", foreground, bg, msg),
            None => format!("\x03{}{}\x0f", foreground, msg),
        }
    }

    fn beer() -> String {
        let glass = Self::color(" ", "15", Some("15")); // light_grey on light_grey
        let content = (0..9)
            .map(|_| {
                let chars = "       :.";
                Self::color(
                    &chars.chars().choose(&mut thread_rng()).unwrap().to_string(),
                    "07", // orange
                    Some("08"), // yellow
                )
            })
            .collect::<String>();
        format!("{}{}{}", glass, content, glass)
    }

    fn cigarette(size: usize) -> String {
        let filter = format!(
            "{}{}",
            Self::color(";.`-,:.`;", "08", Some("07")), // yellow on orange
            Self::color(" ", "08", Some("08")),          // yellow on yellow
        );
        let cigarette = Self::color(&"|".repeat(size), "15", Some("00")); // light_grey on white
        let cherry = format!(
            "{}{}",
            Self::color("\u{259A}", Self::random_choice(&["04", "08", "07"]), Some("01")), // random color on black
            Self::color("\u{259A}", Self::random_choice(&["04", "08", "07"]), Some("14")), // random color on grey
        );
        let smoke_chars = ";:-.,_`~'";
        let smoke = Self::color(
            &format!(
                "-{}",
                (0..Self::random_range(5, 9))
                    .map(|_| smoke_chars.chars().choose(&mut thread_rng()).unwrap())
                    .collect::<String>()
            ),
            "14", // grey
            None,
        );
        format!("{}{}{}{}", filter, cigarette, cherry, smoke)
    }

    fn joint(size: usize) -> String {
        let joint = Self::color(&"/".repeat(size), "15", Some("00")); // light_grey on white
        let cherry = format!(
            "{}{}",
            Self::color("\u{259A}", Self::random_choice(&["04", "08", "07"]), Some("01")), // random color on black
            Self::color("\u{259A}", Self::random_choice(&["04", "08", "07"]), Some("14")), // random color on grey
        );
        let smoke_chars = ";:-.,_`~'";
        let smoke = Self::color(
            &format!(
                "-{}",
                (0..Self::random_range(5, 9))
                    .map(|_| smoke_chars.chars().choose(&mut thread_rng()).unwrap())
                    .collect::<String>()
            ),
            "14", // grey
            None,
        );
        format!("{}{}{}", joint, cherry, smoke)
    }

    fn mug(size: usize) -> Vec<String> {
        let glass = Self::color(" ", "15", Some("15")); // light_grey on light_grey
        let empty = format!("{}         {}", glass, glass);
        let foam = format!(
            "{}{}{}",
            glass,
            Self::color(":::::::::", "15", Some("00")), // light_grey on white
            glass
        );
        let bottom = Self::color("           ", "15", Some("15")); // light_grey on light_grey
        let mut mug = vec![
            foam.clone(),
            Self::beer(),
            Self::beer(),
            Self::beer(),
            Self::beer(),
            Self::beer(),
            Self::beer(),
            Self::beer(),
        ];
        for _ in 0..(8 - size) {
            mug.pop();
            mug.insert(0, empty.clone());
        }
        for i in 0..mug.len() {
            if i == 2 || i == 7 {
                mug[i] = format!("{}{}{}", mug[i], glass, glass);
            } else if i > 2 && i < 7 {
                mug[i] = format!("{}  {}", mug[i], glass);
            }
        }
        mug.push(bottom);
        mug
    }

    pub async fn handle_drugs_command<W: AsyncWriteExt + Unpin>(
        &mut self,
        writer: &mut W,
        config: &Config,
        command: &str,
        channel: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stats = self.stats.lock().await;
        let parts: Vec<&str> = command.split_whitespace().collect();
        let action = parts[0].trim_start_matches('%');

        match action {
            "chug" => {
                if stats.sips == 0 {
                    stats.sips = 8;
                    stats.chugged += 1;
                }
                for line in Self::mug(stats.sips) {
                    writer
                        .write_all(format!("PRIVMSG {} :{}\r\n", channel, line).as_bytes())
                        .await?;
                }
                stats.sips = stats.sips.saturating_sub(Self::random_range(1, 3));
            }
            "smoke" | "toke" => {
                let option = if action == "smoke" { "smoked" } else { "toked" };
                if stats.hits == 0 {
                    stats.hits = 25;
                    if option == "smoked" {
                        stats.smoked += 1;
                    } else {
                        stats.toked += 1;
                    }
                    self.fat = false;
                } else {
                    let object = if action == "smoke" {
                        Self::cigarette(stats.hits)
                    } else {
                        Self::joint(stats.hits)
                    };
                    if self.fat {
                        for _ in 0..3 {
                            writer
                                .write_all(format!("PRIVMSG {} :{}\r\n", channel, object).as_bytes())
                                .await?;
                        }
                    } else {
                        writer
                            .write_all(format!("PRIVMSG {} :{}\r\n", channel, object).as_bytes())
                            .await?;
                    }
                    stats.hits = stats.hits.saturating_sub(Self::random_range(1, 3));
                }
            }
            "100" | "extendo" | "fatfuck" if Self::luck(100) => {
                if action == "fatfuck" {
                    self.fat = true;
                    writer
                        .write_all(
                            format!(
                                "PRIVMSG {} :{}{}{}\r\n",
                                channel,
                                Self::color(" !!! ", "04", Some("03")), // red on green
                                Self::color(
                                    "AWWW SHIT, IT'S TIME FOR THAT MARLBORO FATFUCK",
                                    "01", // black
                                    Some("03") // green
                                ),
                                Self::color(" !!! ", "04", Some("03")) // red on green
                            )
                            .as_bytes(),
                        )
                        .await?;
                } else {
                    stats.hits = 100;
                    if action == "100" {
                        writer
                            .write_all(
                                format!(
                                    "PRIVMSG {} :{}{}{}\r\n",
                                    channel,
                                    Self::color(" !!! ", "00", Some("04")), // white on red
                                    Self::color(
                                        "AWWW SHIT, IT'S TIME FOR THAT NEWPORT 100",
                                        "04", // red
                                        Some("00") // white
                                    ),
                                    Self::color(" !!! ", "00", Some("04")) // white on red
                                )
                                .as_bytes(),
                            )
                            .await?;
                    } else {
                        writer
                            .write_all(
                                format!(
                                    "PRIVMSG {} :{}{}{}\r\n",
                                    channel,
                                    Self::color(" !!! ", "04", Some("03")), // red on green
                                    Self::color(
                                        "OHHH FUCK, IT'S TIME FOR THAT 420 EXTENDO",
                                        "08", // yellow
                                        Some("03") // green
                                    ),
                                    Self::color(" !!! ", "04", Some("03")) // red on green
                                )
                                .as_bytes(),
                            )
                            .await?;
                    }
                }
            }
            "beer" => {
                let target = if parts.len() > 1 { parts[1] } else { channel };
                self.handle_beer_command(writer, target, channel).await?;
            }
            _ => {}
        }

        writer.flush().await?;
        Ok(())
    }

    async fn handle_beer_command<W: AsyncWriteExt + Unpin>(
        &self,
        writer: &mut W,
        target: &str,
        channel: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (beer_choice, beer_temp) = self.generate_beer();
        let beer = self.format_beer(&beer_choice);

        let action = format!(
            "PRIVMSG {} :\x01ACTION throws {} {} {} =)\x01\r\n",
            channel,
            Self::color(target, "00", None),
            beer_temp,
            beer
        );
        writer.write_all(action.as_bytes()).await?;

        if beer_choice == "bud" && Self::luck(100) {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let gay_msg = format!(
                "PRIVMSG {} :\x01ACTION suddenly feels more gay...\x01\r\n",
                channel
            );
            writer.write_all(gay_msg.as_bytes()).await?;
        }

        Ok(())
    }

    fn generate_beer(&self) -> (String, String) {
        let beer_choice = ["bud", "modelo", "ultra"].choose(&mut thread_rng()).unwrap().to_string();
        let beer_temp = ["a piss warm", "an ice cold", "an empty"].choose(&mut thread_rng()).unwrap().to_string();
        (beer_choice, beer_temp)
    }

    fn format_beer(&self, choice: &str) -> String {
        match choice {
            "bud" => format!(
                "{}{}{}",
                Self::color(" ", "00", Some("00")),
                Self::color(" BUD ", "00", Some(["02", "05"].choose(&mut thread_rng()).unwrap())),
                Self::color("c", "14", Some("00"))
            ),
            "modelo" => format!(
                "{}{}{}",
                Self::color(" ", "07", Some("07")),
                Self::color("Modelo", "02", Some("08")),
                Self::color("c", "14", Some("07"))
            ),
            "ultra" => format!(
                "{}{}",
                Self::color(" ULTRA ", "02", Some("00")),
                Self::color("🬃", "04", Some("00"))
            ),
            _ => String::new(),
        }
    }

    fn random_choice<T: Clone>(choices: &[T]) -> T {
        choices.choose(&mut thread_rng()).unwrap().clone()
    }

    fn random_range(start: usize, end: usize) -> usize {
        thread_rng().gen_range(start..end)
    }

    fn luck(odds: u32) -> bool {
        thread_rng().gen_range(1..=odds) == 1
    }
}
