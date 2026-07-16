# Lane 2A synchronized required checks

PR: [Island-Dev-Crew/garnet#509](https://github.com/Island-Dev-Crew/garnet/pull/509)

Checked head: `dfb753c1f7bc38ed713567cd59385ba6a7e25066`

## Evidence-body gate

- [PR dogfood evidence](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607090/job/87767760106): SUCCESS

The corrected body was evaluated on a fresh `synchronize` event. No rerun of
the stale opening payload and no gate exclusion was used.

## CI dependency chain

Run: [29542607095](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095)

- [agent documentation contracts](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767760125): SUCCESS
- [rustfmt](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767858886): SUCCESS
- [clippy](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767894424): SUCCESS
- [Ubuntu cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767894456): SUCCESS
- [Windows cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767894431): SUCCESS
- [macOS cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542607095/job/87767894443): SUCCESS

The Ubuntu `agent documentation contracts` job ran the Python W-PLAY reporter,
its regressions, and the trust-kernel companion gate. The three OS jobs ran the
Rust workspace. This evidence does not describe cargo-only jobs as cross-OS
Python-reporter execution.

At capture time all 34 check runs on the checked head were terminal with
success, skipped, or neutral conclusions; none was failed, cancelled, timed
out, queued, or in progress.

This link-record commit is evidence-only. Its pushed head must receive a fresh
required-chain run before PR #509 is marked ready for Jon.
