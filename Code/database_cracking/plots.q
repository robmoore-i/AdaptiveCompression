// USAGE: q plots.q t1 t2 t3 ...
// Loads bms/t1 bms/t2 bms/t3 as t1, t2, t3 in the q session.

\l qchart.q

ts set' {value ` sv (`:bms,x)} each ts:`$.z.x

scatterAllNodes:{qchart.points select nodes,unoptimised,adaptive,preclustered,preclusteredRLE from t}
scatterAllEdges:{qchart.points select edges,unoptimised,adaptive,preclustered,preclusteredRLE from t}
scatterAllDensity:{qchart.points select density,unoptimised,adaptive,preclustered,preclusteredRLE from t}

scatterNodesRange:{[lbound;ubound]
  qchart.points select nodes,unoptimised,adaptive,preclustered,preclusteredRLE from t where nodes within (lbound;ubound)}
scatterEdgesRange:{[lbound;ubound]
  qchart.points select edges,unoptimised,adaptive,preclustered,preclusteredRLE from t where edges within (lbound;ubound)}
scatterDensityRange:{[lbound;ubound]
  qchart.points select density,unoptimised,adaptive,preclustered,preclusteredRLE from t where density within (lbound;ubound)}
