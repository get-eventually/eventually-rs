# Contributing to `eventually`

Thank you so much for considering contributing to `eventually`! :tada:

- [Contributing to `eventually`](#contributing-to-eventually)
  - [What should I know before I get started?](#what-should-i-know-before-i-get-started)
    - [I want to contribute, but I don't know what can I contribute to...](#i-want-to-contribute-but-i-dont-know-what-can-i-contribute-to)
  - [Pull Requests](#pull-requests)
  - [How should my Pull Request look like?](#how-should-my-pull-request-look-like)
    - [Describe the scope of the PR](#describe-the-scope-of-the-pr)
    - [Git commit messages](#git-commit-messages)
    - [Rust styleguide](#rust-styleguide)

## What should I know before I get started?

The current project structure is using a [Rust virtual workspace][workspace],
where all the crates share the same dependencies and lock file.
(This might be changed in the future)

The workspace has the following crates:
- [`eventually`]\: the main crate, users should depend solely on this crate
- [`eventually-app-example`]\: an example application to showcase a (not-so) real-world example of how to use the library

Although `eventually-postgres` and `eventually-redis` are currently in the workspace, they might be moved outside the repository, into their own respective repository inside the organization. (For more info, check out #131 and #132)

When submitting a PR involving one or more of these crates, please make sure to use the **corresponding label**.

### I want to contribute, but I don't know what can I contribute to...

If you want to contribute but you have no idea how, or what specific features are
missing, please check the [_What next?_ tracking issue](https://github.com/eventually-rs/eventually-rs/issues/133).

Being an open-source project, we are always looking for new contributors, so no matter
your level of Event Sourcing skills or language mastery, your PRs are always welcome :heart:

If you want to experiment and/or get familiar with the library, you can consider
trying to write a small application using the crate and send a message to our
[Gitter chat][gitter] to give visibility: we can add your example project to the
repository's `README` in order for other users to find it and draw inspiration
from your experience.

## Pull Requests

Before opening a Pull Request, check the [Issues section][issues] for an issue that might describe the feature you're considering adding.

If no such issue has been created, [feel free to create one][new-issue] and use the `rfc` label to discuss technical
implementation before the code review process. You can also reach out to our [Gitter chat][gitter] to discuss new
features or other technical aspects of the crate.

In order to submit a PR, follow these steps:
1. **Fork** the repository
2. **Clone** your fork of the repository in your local machine
3. Add the upstream repository in your local machine copy:
    ```bash
    git remote add upstream git@github.com:eventually-rs/eventually-rs
    git fetch upstream
    ```
4. Create a new branch starting from `upstream/master`:
    ```bash
    git checkout -b "<branch-name>" --track upstream/master
    ```
5. Do your magic :tada:
6. Once ready, open a PR pointing to `upstream/master` :+1:

After the PR is created, wait for the CI pipeline to run and the test to pass. Whenever introducing new changes to the
repository, make sure your changes are **covered** by **unit** or **integration tests**.

When all PR badges are green, the review process can start.
Maintainer and collaborators will try to finish the PR review as fast as possible.

Once the PR has been approved, the maintainer or collaborators will **squash-merge** the PR onto `master`.

## How should my Pull Request look like?

### Describe the scope of the PR

Please, include a meaningful PR description, detailing is the scope of the PR and the changes
introduced to fulfill it.

If an issue has been opened already, and if the technical discussion has already happened in the issue, you can avoid
including a detailed PR description by linking the issue to the PR (e.g. `Closes #..` or `Fixes #..`).

### Git commit messages

The project makes use of [Conventional Commits][conventional-commits] style for the Git commit messages.

Please, make sure to follow the style specifications, and try to leave a clear commit history: this will make the review
process easier for reviewers, as the review can be carried _commit-by-commit_.

### Rust styleguide

As you might've guessed, `eventually` is a Rust project.

The CI pipeline will check the code style by using `clippy` and `cargo check`
every time a new commit has been pushed to an open PR.

Make sure you run `rustfmt` **before committing any changes**, as failing to do so will most likely fail the CI pipeline steps.

[workspace]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
[`eventually`]: ./eventually
[`eventually-app-example`]: ./eventually-app-example
[issues]: https://github.com/eventually-rs/eventually-rs/issues
[new-issue]: https://github.com/eventually-rs/eventually-rs/issues/new
[gitter]: https://gitter.im/eventually-rs/community
[conventional-commits]: https://www.conventionalcommits.org/en/v1.0.0/
