all: zahr

zahr:
	g++ -std=c++14 -I zstr/src/ -lz -lncurses src/zahradnice.cpp src/grammar.cpp -o zahradnice -Os -s

RELEASE_DIR=release
install: soko
	rm -rf ${RELEASE_DIR}
	mkdir -p ${RELEASE_DIR}/zahradnice/programs
	gzip -k programs/*.cfg
	mv programs/*.cfg.gz ${RELEASE_DIR}/zahradnice/programs
	cp zahradnice ${RELEASE_DIR}/zahradnice
	cd ${RELEASE_DIR}; \
	tar -czf zahradnice.tar zahradnice/

SOKOWEB=http://www.sneezingtiger.com/sokoban/levels
SOKOFILES=picokosmosText.htm #sasquatch5Text.htm

soko:
	cp programs/partial/sokoban.cfg programs/sokoban.cfg
	for sokofile in ${SOKOFILES}; do \
    wget -N "${SOKOWEB}/$$sokofile"; \
	  grep "^Level\|#" "$$sokofile" > sokoban.txt; \
	  cat sokoban.txt \
      | sed 's/Level \([0-9]\+\)/\1/g' \
      | sed 's/^[^0-9].*//g' \
      | awk '{if($$1 != "") { a=$$1}; print a; }' > numbers.txt; \
	  paste numbers.txt sokoban.txt \
      | sed 's/^\([0-9]*\)\t\([^+@]*\)$$/~\1 \2/'\
      | sed 's/~[0-9] /~~~~/'\
      | sed 's/~[0-9][0-9] /~~~~~/'\
      | tr '#' 'X'  | sed 's/X/##/g' \
      | tr ' ' 'S'  | sed 's/S/  /g' \
      | tr '$$' 'b' | sed 's/b/st/g' \
      | tr '*' 'B'  | sed 's/B/ST/g' \
      | tr '.' 'C'  | sed 's/C/../g' \
      | tr '~' ' ' \
      | sed 's/^\([0-9]\+\)\t\([^@]*\)@/~\1@@\2@P/' \
      | sed 's/^\([0-9]\+\)\t\([^+]*\)+/~\1@@\2@:/' \
      | sed 's/  @P/~~@P/g' | sed 's/ @P/~@P/g' \
      | sed 's/  @:/~~@:/g' | sed 's/ @:/~@:/g' \
      | sed 's/^\s*\(Level.*\)$$/==\/TP/g' >> programs/sokoban.cfg; \
  done;\
  rm -f numbers.txt sokoban.txt

