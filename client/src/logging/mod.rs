use tracing_appender::rolling;

pub fn init_logging() {
    let file_appender = rolling::daily("logs", "app.log");
    let subscriber = tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false) // no colors in file
        .with_thread_names(true)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set up logs");
}
