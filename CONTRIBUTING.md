# Contributing to APM

Thank you for your interest in contributing to APM.

## Contributor License Agreement (CLA)

Before we can accept your contribution, you must sign our Contributor License
Agreement. This grants us the necessary rights to distribute your contribution
under the project's license terms.

When you open your first pull request, the CLA Assistant bot will post a comment
with a link to sign the agreement. You only need to sign once.

### Why a CLA?

APM is licensed under the Business Source License 1.1 (BSL). The CLA ensures
that:

- You confirm you have the right to contribute the code
- The project can continue to be distributed under its current and future
  license terms
- We can offer alternative licensing to organizations that need it

### What the CLA covers

- You retain copyright of your contribution
- You grant us a perpetual, worldwide, non-exclusive license to use, modify,
  and distribute your contribution
- You confirm the contribution is your original work (or you have permission
  to submit it)

## Getting started

1. Fork the repository
2. Create a branch for your change
3. Make your changes following the project's code style (see `CLAUDE.md`)
4. Run `cargo test --workspace` and ensure all tests pass
5. Open a pull request

## Code style

- Imperative mood commit messages: "Add X", "Fix Y"
- First line of commit message under 72 characters
- No unnecessary abstractions — solve the specific problem
- Add tests for new functionality

## Reporting issues

Open an issue on GitHub. Include:

- What you expected to happen
- What actually happened
- Steps to reproduce
- Version information (`apm --version`)
