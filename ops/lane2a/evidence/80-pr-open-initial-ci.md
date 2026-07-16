# Lane 2A draft PR and initial CI

Draft PR: [Island-Dev-Crew/garnet#509](https://github.com/Island-Dev-Crew/garnet/pull/509)

Opened head: `8dc96231368d1f0c39b9df177ee486bb948c78bc`

Base: `1a0e5d729164ab30ae40523db206b1c36ee80045`

## Initial required chain

CI run: [29542449542](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542)

- [agent documentation contracts](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767299055): SUCCESS
- [rustfmt](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767373089): SUCCESS
- [clippy](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767403729): SUCCESS
- [Ubuntu cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767403759): in progress when captured
- [Windows cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767403786): in progress when captured
- [macOS cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449542/job/87767403762): in progress when captured

The Ubuntu-only `agent documentation contracts` job runs the Python W-PLAY
reporter and trust companion. The three OS jobs run the Rust workspace; this
record does not misstate them as cross-OS Python-reporter execution.

## Dogfood body loopback

The initial [PR dogfood evidence job](https://github.com/Island-Dev-Crew/garnet/actions/runs/29542449558/job/87767299219)
failed because the prepared body lacked the repository's six required evidence
headings. This was a PR-body process failure, not a code or browser gate failure.

The body now includes current truth, checked local verification, explicitly
pending remote checks, committed evidence bundle paths, and deferred boundaries.
The unchanged gate passes locally:

`python3 scripts/check_dogfood_pr_body.py --base origin/main --head HEAD --body-file /tmp/garnet-lane2a-pr-body.md`

Result: `dogfood-pr-body: ok (33 changed files checked)`.

This evidence-only commit intentionally triggers a new `synchronize` event so
GitHub evaluates the corrected body. No workflow, threshold, or exclusion was
weakened to obtain the rerun.
