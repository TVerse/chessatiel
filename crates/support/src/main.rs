use anyhow::{anyhow, Result};
use clap::Parser;
use clap::Subcommand;
use rayon::ThreadPoolBuilder;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use support::pgn::pgn_to_annotated_fen;
use support::pst_optimization::train;
use support::AnnotatedPosition;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Convert {
        #[clap(short = 'i', long)]
        input_folder: PathBuf,
        #[clap(short = 'o', long)]
        output_folder: PathBuf,
    },
    OptimizePST {
        #[clap(short = 'i', long)]
        input_folder: PathBuf,
        #[clap(short = 'o', long)]
        output_file: PathBuf,
        #[clap(long)]
        learning_rate: f32,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    let threadpoolbuilder = ThreadPoolBuilder::new()
        .num_threads(16)
        .thread_name(|idx| format!("rayon-{idx}"));
    ThreadPoolBuilder::build_global(threadpoolbuilder)?;

    match args.command {
        Commands::Convert {
            input_folder,
            output_folder,
        } => convert(input_folder, output_folder),
        Commands::OptimizePST {
            input_folder,
            output_file,
            learning_rate,
        } => optimize(input_folder, output_file, learning_rate),
    }
}

fn convert(input_folder: PathBuf, output_folder: PathBuf) -> Result<()> {
    let files = std::fs::read_dir(input_folder).unwrap();

    println!("Parsing...");
    for directory_entry in files {
        let directory_entry = directory_entry?;
        let path = directory_entry.path();
        if path.is_dir() {
            continue;
        }
        let mut pgn = String::new();
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Path {} was not a file?", path.display()))?
            .to_owned();
        let mut source = std::fs::File::open(path)?;
        let _ = source.read_to_string(&mut pgn)?;
        let annotated = pgn_to_annotated_fen(&pgn)?;
        let mut out = String::new();
        annotated
            .iter()
            .for_each(|ap| out.push_str(&ap.to_string()));
        let mut output_path = output_folder.clone();
        output_path.push(file_name);
        let mut dest = std::fs::File::create(output_path)?;
        dest.write_all(out.as_bytes())?
    }
    println!("Done parsing, data written");

    Ok(())
}

fn optimize(input_folder: PathBuf, output_file: PathBuf, learning_rate: f32) -> Result<()> {
    println!("Loading annotated FENs...");
    let files = std::fs::read_dir(input_folder)?;
    let mut training_set = Vec::new();
    for dir_entry in files {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        if path.is_dir() {
            continue;
        }
        let mut fens = String::new();
        let mut source = std::fs::File::open(path)?;
        let _ = source.read_to_string(&mut fens);
        let fens = fens
            .lines()
            .map(|s| AnnotatedPosition::from_str(s).map_err(|s| anyhow!("{}", s)))
            .collect::<Result<Vec<_>>>()?;
        training_set.extend(fens.into_iter());
    }

    println!("Training...");
    let coefficients = train(learning_rate, &*training_set);

    let string = serde_json::to_string(&coefficients)?;
    std::fs::File::create(output_file)?.write_all(string.as_bytes())?;
    println!("Done optimizing, data written");

    Ok(())
}
