\l qchart.q

t:value `:bms/bfsSparse

scatterAllNodes:{qchart.points select nodes,unoptimised,adaptive,preclustered,preclusteredRLE from t}
scatterAllEdges:{qchart.points select edges,unoptimised,adaptive,preclustered,preclusteredRLE from t}
scatterAllDensity:{qchart.points select density,unoptimised,adaptive,preclustered,preclusteredRLE from t}

scatterNodesRange:{[lbound;ubound]
  qchart.points select nodes,unoptimised,adaptive,preclustered,preclusteredRLE from t where nodes within (lbound;ubound)}
scatterEdgesRange:{[lbound;ubound]
  qchart.points select edges,unoptimised,adaptive,preclustered,preclusteredRLE from t where edges within (lbound;ubound)}
scatterDensityRange:{[lbound;ubound]
  qchart.points select density,unoptimised,adaptive,preclustered,preclusteredRLE from t where density within (lbound;ubound)}
