use guts::{MoveGenerator, Position};

#[test]
fn test_perft_movegen_starting_board() {
    let generator = MoveGenerator::new();

    let starting_position = Position::default();

    let count = perft(&generator, &starting_position, 3);

    assert_eq!(count, 8902)
}

fn perft(generator: &MoveGenerator, position: &Position, depth: usize) -> usize {
    if depth == 0 {
        1
    } else {
        let moves = generator.generate_legal_moves_for(&position);
        moves.into_iter().fold(0, |acc, m| {
            let mut position = position.clone();
            position.make_move(&m);
            acc + perft(generator, &position, depth - 1)
        })
    }
}
