txtFh:hsym`$.z.x 0
tab:`$-4_.z.x 0
bmsTab:` sv (`:bms;tab)

parseRuntime:{"F"$-1_2_string x}

t:`nodes`edges`density`unoptimised`preclustered`preclusteredRLE`decomposed`recognitive`compactive`underswapRLE`overswapRLE xcol
  update
    unoptimised:parseRuntime each unoptimised,
    preclustered:parseRuntime each preclustered,
    preclusteredRLE:parseRuntime each preclusteredRLE,
    decomposed:parseRuntime each decomposed,
    recognitive:parseRuntime each recognitive,
    compactive:parseRuntime each compactive,
    underswapRLE:parseRuntime each underswapRLE,
    overswapRLE:parseRuntime each overswapRLE
    from ("JJFSSSSSSSS";enlist ",") 0: txtFh

$[tab in key `:bms;
  upsert[bmsTab;t];
  bmsTab set t]

exit 0
