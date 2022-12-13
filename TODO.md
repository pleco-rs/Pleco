## Branding

Create logo and branding
Setup Github org
Setup docs and move content to there: https://github.com/pleco-rs/Pleco/issues/51
Revise all readmes
Create website
Revise what files are included in the crates

## Publishing

Publish new version of crates
Raise PR for other protocols to use pleco again

## Development

Test engine in Arena: https://github.com/pleco-rs/Pleco/issues/132
Find some AI code review tool to find improvements
AI PR reviews
Add better output like this one does (https://github.com/MitchelPaulin/Walleye)
Port over Stockfish end of game table: https://github.com/pleco-rs/Pleco/issues/113
Review unstable features and which ones we can add back: https://github.com/pleco-rs/Pleco/issues/77
Do some code profiling to see where the bottlenecks are
Suggestions from here (Fix nightly warnings): https://github.com/sfleischman105/Pleco/issues/131 (then remove `uninit_assumed_init` and `missing_safety_doc`)

## Integrations

Give speed comparison vs Stockfish: https://github.com/pleco-rs/Pleco/issues/128
Create comparison with other projects: https://github.com/pleco-rs/Pleco/issues/126
Update Chess engine competitive list
Setup Lichess playable bot

## New features

Implement PGN Parser: https://github.com/pleco-rs/Pleco/issues/71
Consider splitting up repos
Look at all changes stockfish has made since Pleco was created, port over meaningful ones that will benefit Pleco

================================================

Update more packages
Use the Chess.dom analyser to see weaknesses
Consider moving the board ranking to the pleco_engine package
Add cool runtime output
