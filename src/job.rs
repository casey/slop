use {super::*, replace::Replace};

mod replace;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
pub(crate) enum Job {
  Replace(Replace),
}

impl Job {
  pub(crate) fn load(path: &Path) -> Result<Self> {
    Ok(serde_yaml::from_reader(File::open(path)?)?)
  }

  pub(crate) async fn run(self) -> Result {
    match self {
      Self::Replace(replace) => replace.run().await?,
    }
    Ok(())
  }
}
