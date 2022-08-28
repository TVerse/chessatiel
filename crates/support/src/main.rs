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

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

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
        #[clap(long, default_value_t = 0)]
        dropped_positions_start_of_game: usize,
        #[clap(long, default_value_t = 0)]
        dropped_positions_end_of_game: usize,
    },
    OptimizePST {
        #[clap(short = 'i', long)]
        input_folder: PathBuf,
        #[clap(short = 'o', long)]
        output_file: PathBuf,
        #[clap(long)]
        learning_rate: f64,
    },
}

fn main() -> Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profile = dhat::Profiler::new_heap();
    let args = Args::parse();

    let threadpoolbuilder = ThreadPoolBuilder::new()
        .num_threads(16)
        .thread_name(|idx| format!("rayon-{idx}"));
    ThreadPoolBuilder::build_global(threadpoolbuilder)?;

    match args.command {
        Commands::Convert {
            input_folder,
            output_folder,
            dropped_positions_start_of_game,
            dropped_positions_end_of_game,
        } => convert(
            input_folder,
            output_folder,
            dropped_positions_start_of_game,
            dropped_positions_end_of_game,
        ),
        Commands::OptimizePST {
            input_folder,
            output_file,
            learning_rate,
        } => optimize(input_folder, output_file, learning_rate),
    }
}

fn convert(
    input_folder: PathBuf,
    output_folder: PathBuf,
    dropped_positions_start_of_game: usize,
    dropped_positions_end_of_game: usize,
) -> Result<()> {
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
        let annotated = pgn_to_annotated_fen(
            &pgn,
            dropped_positions_start_of_game,
            dropped_positions_end_of_game,
        )?;
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

fn optimize(input_folder: PathBuf, output_file: PathBuf, learning_rate: f64) -> Result<()> {
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
    let coefficients = train(learning_rate, training_set);

    let serialized = bincode::serialize(&coefficients)?;
    std::fs::File::create(output_file)?.write_all(&serialized)?;
    println!("Done optimizing, data written");

    Ok(())
}
