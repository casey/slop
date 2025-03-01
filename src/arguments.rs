use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[clap(long)]
  job: PathBuf,
}

impl Arguments {
  pub(crate) async fn run(self) -> Result {
    let job: Job = serde_yaml::from_reader(File::open(self.job)?)?;

    job.run().await
  }
}
