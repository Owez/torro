# `torro/tests`

Contains integration-specific tests for torro. All simple unit tests are kept as inline `test::x` modules for each specific file for improved structure.

## A note to maintainers

Please do not continously update large test files (included inside `tests/data`) as it may effect git cloning preformance and overall repository neatness. Anything that adversely affects this for no specific reason may be rejected.
