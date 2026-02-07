use clap::Parser;

/// Minimal SafariBooks port (cookies only).
#[derive(Parser, Debug, PartialEq)]
#[command(version)]
pub struct Args {
    /// Book digits ID from the O'Reilly URL.
    pub bookid: String,

    /// Do not delete the log file on success.
    #[arg(long = "preserve-log")]
    pub preserve_log: bool,
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::{CommandFactory, Parser};

    #[test]
    fn parses_positional_bookid_only() {
        // safaribooks-rs 9781491958698
        let args = Args::try_parse_from(["safaribooks-rs", "9781491958698"]).unwrap();
        assert_eq!(args.bookid, "9781491958698");
        assert!(!args.preserve_log);
    }

    #[test]
    fn parses_with_preserve_log_flag() {
        // safaribooks-rs --preserve-log 9781491958698
        let args =
            Args::try_parse_from(["safaribooks-rs", "--preserve-log", "9781491958698"]).unwrap();
        assert_eq!(args.bookid, "9781491958698");
        assert!(args.preserve_log);
    }

    #[test]
    fn error_when_missing_bookid() {
        // safaribooks-rs --preserve-log
        let err = Args::try_parse_from(["safaribooks-rs", "--preserve-log"]).unwrap_err();
        let msg = err.to_string();
        // clap’s message should indicate the missing required argument
        assert!(msg.contains("<BOOKID>"));
    }

    #[test]
    fn error_on_unknown_flag() {
        // safaribooks-rs --kindle 9781491958698
        let err =
            Args::try_parse_from(["safaribooks-rs", "--kindle", "9781491958698"]).unwrap_err();
        let msg = err.to_string().to_lowercase();
        assert!(msg.contains("unexpected argument '--kindle' found"));
    }

    #[test]
    fn help_and_version_are_available() {
        // `--help` and `--version` are auto-provided by clap
        let help = Args::command().render_help().to_string();
        assert!(help.contains("Usage:"));
        assert!(help.contains("<BOOKID>"));
        assert!(help.contains("--preserve-log"));

        let version = Args::command().render_version();
        assert!(!version.trim().is_empty());
    }
}
