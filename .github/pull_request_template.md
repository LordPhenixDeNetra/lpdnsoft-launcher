## Task and requirements

<!--
What was asked, and what "done" means for it. Name the backlog item (P1, P2…) when there is
one. If the answer is "nothing was asked", say why the change exists.
-->

## What changed

<!--
The pieces, in the order a reviewer should read them. Name the decision made, if any, and
point at its ADR.
-->

## Tests

<!--
Which tests prove the behaviour, and why they would fail without this change. A test that
passes both before and after proves nothing.
-->

## Checks run

- [ ] `cargo xtask ci` — i18n completeness, `fmt --check`, `clippy -D warnings`, tests,
      `cargo deny check` (exit 0)
- [ ] `BACKLOG.md` updated: the task carries its new status, and its pull request number when
      it is delivered

## What is not done

<!--
Known gaps, each with the reason. "None" is acceptable only when it is true: a reviewer
cannot guess what you left out.
-->

## Checklist

- [ ] Every visible string is in `locales/en/launcher.ftl` **and** `locales/fr/launcher.ftl`
- [ ] A public API change comes with an integration test in `crates/<crate>/tests/`
- [ ] A decision taken is recorded in `docs/adr/`, not only in this description
- [ ] Commits are signed off (`git commit -s`) and carry no tool attribution
