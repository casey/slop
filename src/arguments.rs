use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[clap(long)]
  job: PathBuf,
}

impl Arguments {
  pub(crate) async fn run(self) -> Result {
    Job::load(&self.job)?.run().await
  }
}
