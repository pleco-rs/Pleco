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
 
 ##### Bot_expert

 An experimental improvement on [IterativeSearcher]. Here, two ideas were tried, and yet somehow failed. 
 
 Firstly, the transpotition table was used, to limited success. It's hard to benchmark a static structure if it needs to be cleared every move in the benchmark. 
 
 The second change was merging the parallel and sequential branches of the ITerative Searcher. Iterative Searcher had two methods tht did similar things, one being `jamboree` and the other being `alphabeta`. jamboree was performeed until a couple plies from the max_depth was reached, and then switched to `alphabeta` search. The only difference between the two methods is that jamboree splits the list of moves into two, and does the first bit sequentailly to find a good alpha cut-off (or trigger a beta-cutoff), and then fork-join the remaining moves. Bot_expert attempted to combine these two methods into one, and force the whole moves list to be searched seuqentially if the depth was high enough.

 ##### Threaded Searcher && Threaded Searcher Param

 More experiments to determine if global variables could be used to determine some parameters, rather than passing them in through directly as parameters. Mostly a worked in progress, but using globals seems significantly slower than passing parameters inside the function. I highly suspect it is due to cache-misses.

##### LazySMP Searcher

Current Work-in-Progress, using a [Lazy SMP](https://chessprogramming.wikispaces.com/Lazy+SMP) algorithm rather than a fork-join model. 