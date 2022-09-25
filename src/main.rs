use anyhow::Context;
use clap::Parser;
use should_color::{clap_color, resolve, ColorChoice};

#[derive(Debug, Parser)]
#[clap(version, color = clap_color())]
struct Cli {
    /// Input file
    input: Option<std::path::PathBuf>,

    /// Output file
    #[clap(short, long)]
    output: Option<std::path::PathBuf>,

    /// Flag
    #[clap(short, long)]
    debug: bool,

    /// Coloring
    #[clap(long, value_name = "WHEN", arg_enum, global = true)]
    color: Option<ColorChoice>,
}

static mut COLOR_STDOUT: bool = false;
static mut COLOR_STDERR: bool = false;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // resolve from cli preference, environment variables, default value
    let color_choice = resolve(cli.color).unwrap_or(ColorChoice::Auto);
    // Safety: the program is single-threaded.
    unsafe {
        COLOR_STDOUT = color_choice.for_stream(atty::Stream::Stdout);
        COLOR_STDERR = color_choice.for_stream(atty::Stream::Stderr);
    }

    init_logger(&cli)?;
    cfg_log::debug!("{cli:?}");

    // Safety: the program is single-threaded.
    colored::control::set_override(unsafe { COLOR_STDOUT });

    if let Some(ref path) = cli.input {
        let file = std::fs::File::open(path)
            .context(format!("cannot open {path:?}"))
            .log_err()?;
        process_stream(std::io::BufReader::new(file))
            .context("cannot process stream")
            .log_err()?;
    } else {
        process_stream(std::io::stdin().lock())
            .context("cannot process stdin")
            .log_err()?;
    }

    Ok(())
}

trait LogErr {
    fn log_err(self) -> Self;
}

impl<T, E: std::fmt::Debug> LogErr for Result<T, E> {
    fn log_err(self) -> Self {
        self.map_err(|e| {
            cfg_log::error!("{:?}", e);
            e
        })
    }
}

#[logging_timer::stime]
fn process_stream<S: std::io::BufRead + std::fmt::Debug>(mut stream: S) -> anyhow::Result<()> {
    cfg_log::debug!("processing stream {:?}", stream);

    let mut content = String::new();
    stream.read_to_string(&mut content)?;

    print!("{content}");

    Ok(())
}

fn init_logger(cli: &Cli) -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        if cli.debug {
            simplelog::LevelFilter::Trace
        } else {
            simplelog::LevelFilter::Info
        },
        simplelog::ConfigBuilder::new()
            .set_target_level(simplelog::LevelFilter::Error)
            .set_location_level(simplelog::LevelFilter::Debug)
            .set_thread_level(simplelog::LevelFilter::Trace)
            .set_level_padding(simplelog::LevelPadding::Left)
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .unwrap_or_else(|e| e)
            .build(),
        simplelog::TerminalMode::Stderr,
        // Safety: the program is single-threaded.
        if unsafe { COLOR_STDERR } {
            simplelog::ColorChoice::Always
        } else {
            simplelog::ColorChoice::Never
        },
    )?;

    Ok(())
}
