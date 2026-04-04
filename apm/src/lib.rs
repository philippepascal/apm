pub mod cmd {
    pub mod agents;
    pub mod assign;
    pub mod close;
    pub mod epic;
    pub mod hook;
    pub mod init;
    pub mod start;
    pub mod validate;
    pub mod verify;
    pub mod list;
    pub mod show;
    pub mod new;
    pub mod state;
    pub mod set;
    pub mod next;
    pub mod sync;
    pub mod worktrees;
    pub mod review;
    pub mod spec;
    pub mod work;
    pub mod clean;
    pub mod workers;
    pub mod register;
    pub mod sessions;
    pub mod revoke;
}

pub use crate::cmd::*;
