\l qchart.q

t:value `:bms/bfsSparse

scatterAllNodes:{qchart.points select nodes,unoptimised,adaptive,preclustered from t}
scatterAllEdges:{qchart.points select edges,unoptimised,adaptive,preclustered from t}
scatterAllDensity:{qchart.points select density,unoptimised,adaptive,preclustered from t}

scatterNodesRange:{[lbound;ubound]
  qchart.points select nodes,unoptimised,adaptive,preclustered from t where nodes within (lbound;ubound)}
scatterEdgesRange:{[lbound;ubound]
  qchart.points select edges,unoptimised,adaptive,preclustered from t where edges within (lbound;ubound)}
scatterDensityRange:{[lbound;ubound]
  qchart.points select density,unoptimised,adaptive,preclustered from t where density within (lbound;ubound)}
