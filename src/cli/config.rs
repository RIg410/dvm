use structopt::StructOpt;

// env variables
pub const DVM_LOG: &str = "DVM_LOG";
pub const DVM_LOG_STYLE: &str = "DVM_LOG_COLOR";
pub const DVM_SENTRY_DSN: &str = "DVM_SENTRY_DSN";
pub const DVM_DATA_SOURCE: &str = "DVM_DATA_SOURCE";

#[derive(Debug, StructOpt, Clone)]
pub struct LoggingOptions {
    /// Enables maximum verbosity logging mode.
    #[structopt(long = "verbose", short = "v")]
    pub verbose: bool,

    /// Log filters.
    /// The same as standard RUST_LOG environment variable.
    /// Possible values in verbosity ordering: error, warn, info, debug and trace.
    /// For complex filters see documentation: https://docs.rs/env_logger/#filtering-results
    #[structopt(
        long = "log",
        env = DVM_LOG,
        default_value = "info,dvm=info,hyper=warn,mio=warn",
        verbatim_doc_comment
    )]
    pub log_filters: String,

    /// Log colors and other styles.
    /// The same as standard RUST_LOG_STYLE environment variable.
    /// Possible values in verbosity ordering: auto, always, never.
    #[structopt(
        long = "log-color",
        env = DVM_LOG_STYLE,
        default_value = "auto",
        verbatim_doc_comment
    )]
    pub log_style: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct IntegrationsOptions {
    /// Optional key-uri, enables crash logging service integration.
    /// If value ommited, crash logging service will not be initialized.
    #[structopt(name = "Sentry DSN", long = "sentry-dsn", env = DVM_SENTRY_DSN)]
    #[cfg(feature = "sentry")]
    pub sentry_dsn: Option<sentry::internals::Dsn>,
}
