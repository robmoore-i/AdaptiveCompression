// USAGE: q plots.q t1 t2 t3 ...
// Loads bms/t1 bms/t2 bms/t3 as t1, t2, t3 in the q session.

\l qchart.q

ts set' {value ` sv (`:bms,x)} each ts:`$.z.x

mergeResults:{[tMany;tSingle]
  `nodes`edges`density`unoptimised`adaptive`adaptiveRLE`preclustered`preclusteredRLE xcols 0!(3!tMany) lj 3!tSingle}

scatterAllNodes:  {[t]qchart.points delete edges,density from t}
scatterAllEdges:  {[t]qchart.points delete nodes,density from t}
scatterAllDensity:{[t]qchart.points delete nodes,edges   from t}

scatterNodesRange:{[t;lbound;ubound]
  qchart.points select from (delete edges,density from t) where nodes within (lbound;ubound)}
scatterEdgesRange:{[t;lbound;ubound]
  qchart.points select from (delete nodes,density from t) where edges within (lbound;ubound)}
scatterDensityRange:{[t;lbound;ubound]
  qchart.points select from (delete nodes,edges from t) where density within (lbound;ubound)}
