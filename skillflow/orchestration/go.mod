module apex-skillflow-orchestration

go 1.23

require apex-skillflow-core v0.1.0

// apex-skillflow-core is a local dependency; replace directive handles the path.
replace apex-skillflow-core => ../core
