## Bots

`src/bots/` is a folder containing a collection of Bots / Searches designed to play a game of chess to varying degrees of skill. This README gives an overview of each of the bots created so far.

### Basic Searchers

Located at `../bots/basic/`, these Searchers both completed and able to be used in matches

##### Random Searcher
Picks a move completely at random.

##### MiniMax Searcher
Implements a [MiniMax](https://chessprogramming.wikispaces.com/Minimax) algorithm to choose a move. The entire tree of move will be searched up to a set ply.

##### Parallel MiniMax Searcher
The same MiniMax Searcher, but utilizes the MiniMax algorithm in a "divide and conquer" fashion. Uses the [Rayon](https://github.com/nikomatsakis/rayon) library to implement this parallelism.  

##### AlphaBeta Searcher
Uses the same (sequential) MiniMax algorithm, but utilizes [Alpha-Beta Pruning](https://chessprogramming.wikispaces.com/Alpha-Beta) to decrease the number of moves searched to a certain ply. 

##### Jamboree Searcher
A parallel implementation of the AlphaBeta Searcher. See the [ChessWiki](https://chessprogramming.wikispaces.com/Jamboree) for more information.

### Experimental Searchers

Located at the `../bots/` root folder, these are Searchers that are not yet finalized, and used for prototyping certain features.

##### Iterative Searcher

Specifically, this searcher is at `bot_iterative_parallel_mvv_lva.rs`, and improves on the Jamboree Searcher. Firstly, it implements [Iterative Deepening](https://chessprogramming.wikispaces.com/Iterative+Deepening), and when doing so uses the previous deoth's score as a bound, ala [Aspirtation Windows](https://chessprogramming.wikispaces.com/Aspiration+Windows).
 
##### LazySMP Searcher

Current Work-in-Progress, using a [Lazy SMP](https://chessprogramming.wikispaces.com/Lazy+SMP) algorithm rather than a fork-join model. 