#!/bin/bash

# TODO automate cloning/building/setting up

cutechess-cli -tournament round-robin \
-rounds 100 \
-openings file=./openings/openings.epd format=epd policy=round  \
-each tc=150/1+0.1 restart=on proto=uci \
-engine name=Chessatiel_full cmd=/home/tim/coding/chessatiel/chessatiel_full stderr=chessatiel_full.log \
-engine name=Chessatiel_no_q cmd=/home/tim/coding/chessatiel/chessatiel_no_q stderr=chessatiel_no_q.log \
-engine name=Chessatiel_no_tt cmd=/home/tim/coding/chessatiel/chessatiel_no_tt stderr=chessatiel_no_tt.log \
-engine name=Chessatiel_no_pst cmd=/home/tim/coding/chessatiel/chessatiel_no_pst stderr=chessatiel_no_pst.log \
-concurrency 4 \
-maxmoves 100 \
-games 8 \
-epdout tournament.epd \
-pgnout tournament.pgn
