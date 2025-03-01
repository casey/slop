use {super::*, replace::Replace};

mod replace;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Job {
  Replace(Replace),
}

impl Job {
  pub(crate) async fn run(self) -> Result {
    match self {
      Self::Replace(replace) => replace.run().await?,
    }

    Ok(())
  }
}
