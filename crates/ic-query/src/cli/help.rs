use std::ffi::OsString;

fn is_help_arg(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "help" | "--help" | "-h"))
}

fn is_version_arg(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "version" | "--version" | "-V"))
}

fn is_version_flag_arg(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "--version" | "-V"))
}

pub fn first_arg_is_help(args: &[OsString]) -> bool {
    args.first().is_some_and(is_help_arg)
}

fn print_help_or_version_matching(
    args: &[OsString],
    usage: impl FnOnce() -> String,
    version_text: &str,
    is_version: fn(&OsString) -> bool,
) -> bool {
    if first_arg_is_help(args) {
        println!("{}", usage());
        return true;
    }
    if args.first().is_some_and(is_version) {
        println!("{version_text}");
        return true;
    }
    false
}

pub fn print_help_or_version(
    args: &[OsString],
    usage: impl FnOnce() -> String,
    version_text: &str,
) -> bool {
    print_help_or_version_matching(args, usage, version_text, is_version_arg)
}

pub fn print_help_or_version_flag(
    args: &[OsString],
    usage: impl FnOnce() -> String,
    version_text: &str,
) -> bool {
    print_help_or_version_matching(args, usage, version_text, is_version_flag_arg)
}

fn collect_args_or_print_if<I>(
    args: I,
    should_print: impl FnOnce(&[OsString]) -> bool,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if should_print(&args) {
        return None;
    }
    Some(args)
}

pub fn collect_args_or_print_help_or_version<I>(
    args: I,
    usage: impl FnOnce() -> String,
    version_text: &str,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_if(args, |args| {
        print_help_or_version(args, usage, version_text)
    })
}

pub fn collect_args_or_print_help_or_version_flag<I>(
    args: I,
    usage: impl FnOnce() -> String,
    version_text: &str,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_if(args, |args| {
        print_help_or_version_flag(args, usage, version_text)
    })
}
