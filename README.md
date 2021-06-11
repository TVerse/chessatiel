# Chessatiel

Will eventually contain subcrates:
* `chessatiel`: the executable
* `guts`: core board and movegen library
* `brain`: evaluation library
* maybe `beak`: UCI library?


## Notes

### Movegen order
* King
* Non-king out-of-check
* Pinned
* Everything else

### (TODO) Magic bitboards

Magic bitboard generation steps:
* For all squares
* For all occupancies of that square's ray/file (excluding edges and the square itself, so 5*5 bits [NOT TRUE if piece starts on edge])
* Calculate the available moves

For all inputs/outputs:
* Generate a random number, choose a shift
* "Hash" all inputs
* Verify there are no unwanted collisions
* If there are, change shift
* If shift too large (too many bits): generate a new number

Can also systematically search for the number but that's maybe for later.
