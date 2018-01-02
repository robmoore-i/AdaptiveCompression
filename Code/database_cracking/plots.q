\l qchart.q

t:value `:bms/bfsSparse

scatterNodes:{qchart.points select nodes,unoptimised,adaptive,preclustered from t}
scatterEdges:{qchart.points select edges,unoptimised,adaptive,preclustered from t}
scatterDensity:{qchart.points select density,unoptimised,adaptive,preclustered from t}
