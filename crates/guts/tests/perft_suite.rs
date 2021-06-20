use guts::{MoveGenerator, Position};
use itertools::Itertools;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

fn prepare() -> Vec<(Position, Vec<(usize, usize)>)> {
    use std::fs::File;
    let source_path = {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests/perftsuite.epd");
        d
    };

    dbg!(&source_path);

    let mut file = File::open(source_path).unwrap();
    let contents = {
        let mut contents = String::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_string(&mut contents).unwrap();
        contents
    };

    let lines = contents.lines();

    lines.map(parse_line).collect()
}

fn parse_line(line: &str) -> (Position, Vec<(usize, usize)>) {
    let mut split = line.split(';').map(|s| s.trim());
    let position = Position::from_str(split.next().unwrap()).unwrap();
    let perfts = split
        .map(|s| {
            let mut perfts_split = s.split(' ').map(|s| s.trim());
            let depth = perfts_split.next().unwrap();
            let value = perfts_split.next().unwrap();
            let depth = usize::from_str(&depth[1..]).unwrap();
            let value = usize::from_str(value).unwrap();
            (depth, value)
        })
        .collect_vec();

    (position, perfts)
}

#[test]
#[ignore]
fn perft_suite() {
    let move_gen = MoveGenerator::new();
    let prepared = prepare();
    for (pos, perfts) in prepared {
        for (depth, expected) in perfts {
            let result = move_gen.perft(&pos, depth);

            assert_eq!(
                expected, result,
                "Wrong perft result for {}: expected {}, got {}",
                pos, expected, result
            )
        }
    }
}
