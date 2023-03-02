pub fn configure_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logger at runtime
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %I:%M:%S %P]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("h2", log::LevelFilter::Info)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Warn)
        .level_for("serenity", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
