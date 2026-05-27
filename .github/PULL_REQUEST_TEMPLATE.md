<!--
Thanks for the PR! Please keep these sections so reviewers can move quickly.
-->

## Summary

<!-- One or two sentences on what this PR changes and why. -->

## Linked issue

<!-- e.g. Closes #123. Use "Refs #123" for partial work. -->

## Test plan

- [ ] `cargo fmt --all --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` passes
- [ ] (If touching `ui_web`) `cargo build -p synergismforkd_ui_web --target wasm32-unknown-unknown` succeeds
- [ ] Manually verified the behavior (describe what you checked):

## Save format impact

<!-- Check one. See CLAUDE.md / CONTRIBUTING.md for why this matters. -->

- [ ] No save format impact
- [ ] Adds field(s) to a `synergismforkd_logic::state` slice (list them; describe default value and migration)
- [ ] Modifies an existing state slice field (describe migration plan)
