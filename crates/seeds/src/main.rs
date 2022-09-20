use anyhow::{anyhow, Result};
use brain::neural_networks::TwoHiddenLayerNetwork;
use clap::Parser;
use clap::Subcommand;
use itertools::Itertools;
use rayon::ThreadPoolBuilder;
use seeds::generate_tournament_openings::generate_tournament_openings;
use seeds::pgn::pgn_to_annotated_fen;
use seeds::pst_optimization::train_pst;
use seeds::run_tournament::{run_tournament, IdAndFilename};
use seeds::train_mnist::train_mnist;
use seeds::train_neural_network::train_nn;
use seeds::AnnotatedPosition;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

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
    TrainNN {
        #[clap(short = 'i', long)]
        input_folder: PathBuf,
        #[clap(short = 'o', long)]
        output_file: PathBuf,
        #[clap(long)]
        learning_rate: f64,
    },
    TrainMnist {
        #[clap(long)]
        training_set_path: PathBuf,
        #[clap(long)]
        test_set_path: PathBuf,
        #[clap(short = 'o', long)]
        output_file: PathBuf,
        #[clap(long)]
        learning_rate: f64,
    },
    GenerateTournamentOpenings {
        #[clap(short = 'i', long)]
        input_folder: PathBuf,
        #[clap(short = 'o', long)]
        output_file: PathBuf,
        #[clap(long, default_value_t = 10)]
        number: usize,
    },
    RunTournament {
        hashes: Vec<IdAndFilename>,
        #[clap(short = 'o', long)]
        output_folder: PathBuf,
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
        } => optimize_pst(input_folder, output_file, learning_rate),
        Commands::TrainNN {
            input_folder,
            output_file,
            learning_rate,
        } => optimize_nn(input_folder, output_file, learning_rate),
        Commands::TrainMnist {
            training_set_path,
            test_set_path,
            output_file,
            learning_rate,
        } => optimize_mnist(training_set_path, test_set_path, output_file, learning_rate),
        Commands::GenerateTournamentOpenings {
            input_folder,
            output_file,
            number,
        } => generate_openings(input_folder, output_file, number),
        Commands::RunTournament {
            hashes,
            output_folder,
        } => do_run_tournament(hashes, output_folder),
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

fn optimize_pst(input_folder: PathBuf, output_file: PathBuf, learning_rate: f64) -> Result<()> {
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
    let coefficients = train_pst(learning_rate, training_set);

    let serialized = bincode::serialize(&coefficients)?;
    std::fs::File::create(output_file)?.write_all(&serialized)?;
    println!("Done optimizing, data written");

    Ok(())
}

fn optimize_nn(input_folder: PathBuf, output_file: PathBuf, learning_rate: f64) -> Result<()> {
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

    // TODO lazy loading
    const TRAINING_EXAMPLES: usize = 100_000;
    const VALIDATION_EXAMPLES: usize = 5_000;
    let training_set = training_set
        .into_iter()
        .take(TRAINING_EXAMPLES + VALIDATION_EXAMPLES)
        .collect_vec();
    let (training_set, validation_set) = training_set.split_at(TRAINING_EXAMPLES);

    println!("Training...");
    let network = train_nn(learning_rate, training_set, validation_set);

    let serialized = bincode::serialize(&network)?;
    std::fs::File::create(output_file)?.write_all(&serialized)?;
    println!("Done optimizing, data written");

    let read_network = bincode::deserialize::<TwoHiddenLayerNetwork<768, 64, 16, 1>>(&serialized)?;
    assert_eq!(network, read_network);
    Ok(())
}

fn optimize_mnist(
    input_file_training: PathBuf,
    input_file_test: PathBuf,
    output_file: PathBuf,
    learning_rate: f64,
) -> Result<()> {
    println!("Loading sets...");
    let mut training_set = String::new();
    let mut source = std::fs::File::open(input_file_training)?;
    let _ = source.read_to_string(&mut training_set);
    let training_set = training_set
        .lines()
        .dropping(1)
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    let mut test_set = String::new();
    let mut source = std::fs::File::open(input_file_test)?;
    let _ = source.read_to_string(&mut test_set);
    let test_set = test_set
        .lines()
        .dropping(1)
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();

    println!("Training...");
    let network = train_mnist(learning_rate, &training_set, &test_set);

    let serialized = bincode::serialize(&network)?;
    std::fs::File::create(output_file)?.write_all(&serialized)?;
    println!("Done optimizing, data written");

    let read_network = bincode::deserialize::<seeds::train_mnist::Net>(&serialized)?;
    assert_eq!(network, read_network);
    Ok(())
}

fn generate_openings(input_folder: PathBuf, output_file: PathBuf, number: usize) -> Result<()> {
    println!("Loading annotated FENs...");
    let files = std::fs::read_dir(input_folder)?;
    let mut positions = Vec::new();
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
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|ap| ap.pos);
        positions.extend(fens);
    }

    println!("Generating...");
    let coefficients = generate_tournament_openings(&mut positions, number);
    let s = coefficients.join("\n");

    std::fs::File::create(output_file)?.write_all(s.as_bytes())?;
    println!("Done generating, data written");

    Ok(())
}

fn do_run_tournament(mut hashes: Vec<IdAndFilename>, output_folder: PathBuf) -> Result<()> {
    if !hashes.iter().any(|i| i.name == "main") {
        hashes.push(IdAndFilename {
            name: "main".to_string(),
            id: "main".to_string(),
        })
    }
    run_tournament(&hashes, output_folder)?;
    Ok(())
}
