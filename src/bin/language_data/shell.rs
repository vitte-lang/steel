const POSIX_BUILTINS: &[&str] = &[
    ".", ":", "[", "alias", "bg", "break", "cd", "command", "continue", "eval", "exec", "exit",
    "export", "false", "fg", "getopts", "hash", "jobs", "kill", "newgrp", "pwd", "read",
    "readonly", "return", "set", "shift", "test", "times", "trap", "true", "type", "ulimit",
    "umask", "unalias", "unset", "wait",
];

const BASH_BUILTINS: &[&str] = &[
    "bind", "builtin", "caller", "compgen", "complete", "compopt", "declare", "dirs", "disown",
    "echo", "enable", "help", "history", "local", "logout", "mapfile", "popd", "printf", "pushd",
    "readarray", "shopt", "source",
];

const ZSH_BUILTINS: &[&str] = &[
    "autoload", "emulate", "fc", "functions", "integer", "limit", "noglob", "print", "rehash",
    "sched", "setopt", "source", "typeset", "unfunction", "unhash", "unlimit", "unsetopt",
    "vared", "whence", "zcompile", "zformat", "zle", "zmodload", "zparseopts", "zstyle", "ztcp",
];

// Union list for current unified sh/zsh mode.
const SHELL_BUILTINS_UNION: &[&str] = &[
    ".", ":", "[", "alias", "autoload", "bg", "bind", "break", "builtin", "caller", "cd", "command",
    "compgen", "complete", "compopt", "continue", "declare", "dirs", "disown", "echo", "emulate",
    "enable", "eval", "exec", "exit", "export", "false", "fc", "fg", "functions", "getopts", "hash",
    "help", "history", "integer", "jobs", "kill", "limit", "local", "logout", "mapfile", "newgrp",
    "noglob", "popd", "print", "printf", "pushd", "pwd", "read", "readarray", "readonly", "rehash",
    "return", "sched", "set", "setopt", "shift", "shopt", "source", "test", "times", "trap", "true",
    "type", "typeset", "ulimit", "umask", "unalias", "unfunction", "unhash", "unlimit", "unset",
    "unsetopt", "vared", "wait", "whence", "zcompile", "zformat", "zle", "zmodload", "zparseopts",
    "zstyle", "ztcp",
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::SHELL_KEYWORDS }
pub(super) fn builtins(dialect: ShellDialect) -> &'static [&'static str] {
    match dialect {
        ShellDialect::Posix => POSIX_BUILTINS,
        ShellDialect::Bash => BASH_BUILTINS,
        ShellDialect::Zsh => ZSH_BUILTINS,
        ShellDialect::Union => SHELL_BUILTINS_UNION,
    }
}

#[allow(dead_code)]
pub(super) fn posix_builtins() -> &'static [&'static str] { POSIX_BUILTINS }
#[allow(dead_code)]
pub(super) fn bash_builtins() -> &'static [&'static str] { BASH_BUILTINS }
#[allow(dead_code)]
pub(super) fn zsh_builtins() -> &'static [&'static str] { ZSH_BUILTINS }
use super::super::ShellDialect;
