#!/bin/bash

title=$1

cargo run --release > "$title.txt"
$QHOME/l32/q tabulateBms.q "$title.txt"
rm "$title.txt"

exit 0
