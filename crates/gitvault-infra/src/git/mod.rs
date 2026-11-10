//! Version control through the `git` command line.
//!
//! Using the `git` binary (instead of libgit2) keeps the build light and
//! honors the user's proxy and certificate settings. The GitHub token is passed
//! through environment variables, never on the command line.

mod command;
mod git_cli;
mod parse;

pub use git_cli::GitCli;
