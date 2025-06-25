use std::{collections::BTreeSet, path::Path, str::FromStr};

use clap::Parser;
use fontations::{skrifa::setting::Setting, types::Tag};
use fontquant_lib::{run, Results};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    csv: bool,
    fonts: Vec<String>,
    #[arg(short, long, default_value = "", value_parser=parse_setting)]
    location: std::vec::Vec<Setting<f32>>,
}

fn print_line(font_file: &str, location: &[Setting<f32>], results: &Results, all_keys: &[&str]) {
    let metrics = all_keys.iter().map(|name| {
        results
            .get(name)
            .map(|m| m.1.to_string())
            .unwrap_or("".to_string())
    });
    let loc_string = location
        .iter()
        .map(|s| format!("{}={}", s.selector, s.value))
        .collect::<Vec<String>>()
        .join(",");
    println!(
        "\"{}\",{:.2},{}",
        Path::new(font_file).file_name().unwrap().to_str().unwrap(),
        loc_string,
        metrics.collect::<Vec<String>>().join(",")
    );
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
    let all_results = args
        .fonts
        .par_iter()
        .progress()
        .map(|font| {
            let font_data = std::fs::read(font).unwrap();
            let fontref =
                fontations::skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            run(&fontref, &args.location).map(|results| (font, results))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to run metrics");
    if args.csv {
        let all_keys: BTreeSet<&str> = all_results
            .iter()
            .flat_map(|(_, results)| results.keys())
            .map(|k| k.as_str())
            .collect();
        let all_keys: Vec<&str> = all_keys.into_iter().collect();
        println!("Font,Location,{}", all_keys.join(","));
        for result in all_results.iter() {
            let (font, results) = result;
            print_line(font, &args.location, results, &all_keys);
        }
    } else {
        for (font, results) in all_results {
            println!("Font: {}", font);
            for (name, (_metric_key, value)) in results.iter() {
                println!(" {}: {:?}", name, value);
            }
            println!();
        }
    }
}
