txtFh:hsym`$.z.x 0
tab:`$-4_.z.x 0
bmsTab:` sv (`:bms;tab)

parseRuntime:{"F"$-1_2_string x}

t:`nodes`edges`density`unoptimised`adaptiveRLE`preclustered`preclusteredRLE xcol
  update
    unoptimised:parseRuntime each unoptimised,
    adaptiveRLE:parseRuntime each adaptiveRLE,
    preclustered:parseRuntime each preclustered,
    preclusteredRLE:parseRuntime each preclusteredRLE
    from ("JJFSSSS";enlist ",") 0: txtFh

$[tab in key `:bms;
  upsert[bmsTab;t];
  bmsTab set t]

exit 0
