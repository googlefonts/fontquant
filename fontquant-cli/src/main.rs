use clap::Parser;
use fontquant_lib::run;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    font: String,
}

fn main() {
    let args = Cli::parse();
    let font_data = std::fs::read(args.font).unwrap();
    let fontref = fontations::skrifa::FontRef::new(&font_data).expect("Failed to parse font");

    let results = run(&fontref, &[]).expect("Failed to run metrics");
    for (name, (_metric_key, value)) in results.iter() {
        println!("{}: {:?}", name, value);
    }
}
