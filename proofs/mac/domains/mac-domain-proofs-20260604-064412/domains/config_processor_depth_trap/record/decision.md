# Agent-loop decision: REJECTED (enforced-ceiling trap)

Proposal `/Users/IDC2.5/Desktop/Garnet/garnet-cli/tests/fixtures/ultrapunch/reject_overdepth.garnet` (vs baseline `/Users/IDC2.5/Desktop/Garnet/garnet-cli/tests/fixtures/ultrapunch/baseline.garnet`) passed diff-caps (no widening) but the enforced kernel (--interp) TRAPPED it (see `run_trap.txt`) — an `@max_depth` or `@caps` ceiling was exceeded. It was not sealed. Acceptance rests on the enforced run, not only the static capability gate.
