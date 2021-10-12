use clap::Clap;

#[derive(Clap, Debug, Clone)]
pub(crate) struct Opt {
    #[clap(subcommand)]
    cmd: Sub,
}

#[derive(Debug, Clone, Clap)]
pub(crate) enum Sub {
    Load(Load),
}

#[derive(Debug, Clone, Clap)]
pub(crate) struct Load {
    target: String,
}
