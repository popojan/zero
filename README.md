# zero

Experimental ASCII animation/game "engine". Programs are defined as plain-text files comprising of local rewrite rules (kind of Type-0 grammars rule templates).

Rewrite of [zahradnice](https://github.com/popojan/zahradnice) into rust/bevy.

Extras (compared to zahradnice):
* sounds
* navigation (jumping from one program to another)
* multiplatform, works outside of terminal (price paid is a bloated engine binary)
* some more program drafts content

### Font

True type font `iosevka-term-regular.ttf` to be downloaded manually into `assets/fonts/`.

### Example Programs
* Arkanoid draft
* Battery Jam inspired draft (2 players / human vs random)
* Flowers animation (original toy idea)
* Hello World (minimal program)
* High Noon (ZX Spectrum) inspired prototype
* Conway's Game of Life (invalid, missing central clock leading to unexpected results when patterns are large)
* Snake classical game example 
* Sokoban draft (original picocosmos levels by Aymeric du Peloux downloaded and recreated by attached Makefile)
* Tetris working example
* Zen Puzzle Garden inspired prototype
