use guts::{BaseMovePatterns, Piece};
use guts::{Gamestate, MoveGenerator};
use std::str::FromStr;

#[test]
fn test_perft_movegen_starting_board() {
    let generator = MoveGenerator::new();

    let starting_state = Gamestate::default();

    let count = perft(&generator, &starting_state, 3);

    assert_eq!(count, 8902)
}

fn perft(generator: &MoveGenerator, state: &Gamestate, depth: usize) -> u64 {
    if depth == 0 {
        1
    } else {
        let moves = generator.generate_legal_moves_for(&state);
        moves.fold(0, |acc, m| {
            let mut state = state.clone();
            state.make_move(&m);
            acc + perft(generator, &state, depth - 1)
        })
    }
}
