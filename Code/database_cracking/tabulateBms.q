txtFh:hsym`$.z.x 0
tab:`$-4_.z.x 0
bmsTab:` sv (`:bms;tab)

parseRuntime:{"F"$-1_2_string x}

t:`nodes`edges`density`unoptimised`adaptive`preclustered xcol
  update
    unoptimised:parseRuntime each unoptimised,
    adaptive:parseRuntime each adaptive,
    preclustered:parseRuntime each preclustered
    from ("JJFSSS";enlist ",") 0: txtFh

$[tab in key `:bms;
  upsert[bmsTab;t];
  bmsTab set t]

exit 0
