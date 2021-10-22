# Chessatiel

Will eventually contain subcrates:
* `chessatiel`: the executable
* `guts`: core board and movegen library
* ~`brain`: evaluation library~ moved into chessatiel, too closely coupled with io and stats
* maybe `beak`: UCI library?


## Notes

### Engine architecture
Everything async (including recursion in core engine logic, at least for now).

* Lichess HTTP stream in
* When accepting challenge: spawn engine set
* Game ends, teardown engine
  * While game is going, there will be persistent state

Events:
* GameFull event:
  * if position changed (or initial) && our turn now:
    * start calculating
* GameState event:
  * if position changed && our turn now:
    * start calculating
* Ignore chat

Calculating:
* Spawn core future, start calculating, spread info to dedicated parts
* Core pushes best moves (+ stats?)
  * Needs full-game persistent transposition table
* Time controller signals when calculation is done
* When done, grab result from move store, send to Lichess
* Tear down core future

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
