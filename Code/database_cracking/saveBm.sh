#!/bin/bash

title=$1

cargo run > "$title.txt"
$QHOME/l32/q tabulateBms.q "$title.txt"
rm "$title.txt"

exit 0
