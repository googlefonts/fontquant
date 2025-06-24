use std::str::FromStr;

use clap::Parser;
use fontations::{skrifa::setting::Setting, types::Tag};
use fontquant_lib::run;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    font: String,
    #[arg(short, long, default_value = "", value_parser=parse_setting)]
    location: std::vec::Vec<Setting<f32>>,
}

fn parse_setting(s: &str) -> Result<Vec<Setting<f32>>, String> {
    if s.is_empty() {
        return Ok(vec![]);
    }
    let settings_text = s.split(',');
    let mut settings: Vec<Setting<f32>> = Vec::new();
    for setting in settings_text {
        let parts: Vec<&str> = setting.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid setting format: {}", s));
        }
        let axis = parts[0].to_string();
        let value: f32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid value for setting: {}", s))?;
        settings.push(Setting::new(
            Tag::from_str(&axis).map_err(|_| format!("Invalid tag: {}", axis))?,
            value,
        ));
    }
    Ok(settings)
}

fn main() {
    let args = Cli::parse();
    let font_data = std::fs::read(args.font).unwrap();
    let fontref = fontations::skrifa::FontRef::new(&font_data).expect("Failed to parse font");

    let results = run(&fontref, &args.location).expect("Failed to run metrics");
    for (name, (_metric_key, value)) in results.iter() {
        println!("{}: {:?}", name, value);
    }
}
