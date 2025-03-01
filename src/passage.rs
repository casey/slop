use super::*;

pub(crate) struct Passage {
  end: usize,
  path: PathBuf,
  start: usize,
  text: String,
}

impl Passage {
  pub(crate) fn find(regex: &Regex, path: &Path) -> Result<Option<Self>> {
    let text = fs::read_to_string(path)?;

    if let Some(capture) = regex.find(&text) {
      Ok(Some(Self {
        path: path.into(),
        start: capture.start(),
        end: capture.end(),
        text,
      }))
    } else {
      Ok(None)
    }
  }

  pub(crate) fn path(&self) -> &Path {
    &self.path
  }

  pub(crate) fn replace(&self, replacement: &str) -> String {
    format!(
      "{}{replacement}{}",
      &self.text[..self.start],
      &self.text[self.end..],
    )
  }

  pub(crate) fn text(&self) -> &str {
    &self.text[self.start..self.end]
  }
}
