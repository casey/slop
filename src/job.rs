use {super::*, fix::Fix, replace::Replace};

mod fix;
mod replace;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
pub(crate) enum Job {
  Fix(Fix),
  Replace(Replace),
}

impl Job {
  pub(crate) fn load(path: &Path) -> Result<Self> {
    Ok(serde_yaml::from_reader(File::open(path)?)?)
  }

  pub(crate) async fn run(self) -> Result {
    match self {
      Self::Fix(fix) => fix.run().await?,
      Self::Replace(replace) => replace.run().await?,
    }
    Ok(())
  }
}
