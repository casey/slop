use super::*;

#[serde_as]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Replace {
  check: Vec<String>,
  commit: String,
  path: PathBuf,
  prompt: String,
  #[serde_as(as = "DisplayFromStr")]
  regex: Regex,
}

impl Replace {
  fn find(&self) -> Result<Option<Passage>> {
    for entry in WalkDir::new(&self.path) {
      let entry = entry?;
      let path = entry.path();

      if entry.file_type().is_dir() || path.extension().is_none_or(|extension| extension != "rs") {
        continue;
      }

      if let Some(passage) = Passage::find(&self.regex, path)? {
        return Ok(Some(passage));
      }
    }

    Ok(None)
  }

  fn prompt(&self, passage: &Passage) -> String {
    self.prompt.replace("%%", passage.text())
  }

  pub(crate) async fn run(self) -> Result {
    let api_key = fs::read_to_string(dirs::home_dir().unwrap().join(".slop"))?
      .trim()
      .to_owned();

    let llm = LLMBuilder::new()
      .backend(LLMBackend::Anthropic)
      .model("claude-3-7-sonnet-20250219")
      .api_key(api_key)
      .build()?;

    while let Some(passage) = self.find()? {
      eprintln!("Found match:\n\n{}\n", passage.text());

      let prompt = self.prompt(&passage);

      let messages = vec![ChatMessage::user().content(prompt).build()];

      let response = match llm.chat(&messages).await {
        Ok(response) => response,
        Err(LLMError::HttpError(err)) if err.contains("529") => continue,
        Err(err) => return Err(err.into()),
      };

      let replacement = response.to_string();

      eprintln!("Replacement:\n\n{replacement}\n");

      fs::write(passage.path(), passage.replace(&replacement))?;

      eprintln!("Running check…");

      let status = Command::new(&self.check[0])
        .args(&self.check[1..])
        .current_dir(&self.path)
        .status()?;

      if !status.success() {
        return Err("Check failed:".into());
      }

      eprintln!("Comitting…");

      let status = Command::new("git")
        .arg("commit")
        .arg("--message")
        .arg(&self.commit)
        .arg("--")
        .arg(passage.path().strip_prefix(&self.path).unwrap())
        .current_dir(&self.path)
        .status()?;

      if !status.success() {
        return Err("Commit failed:".into());
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load() {
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();

    let path = tmp.path().join("job.yaml");

    fs::write(
      &path,
      r"type: replace
commit: rewrite function
path: foo/bar
regex: '(?ms)^fn hello\(\) \{.*?^\}$'
check: ['cargo', 'test']
prompt: hello %% goodbye
",
    )
    .unwrap();

    match Job::load(&path).unwrap() {
      Job::Replace(Replace {
        check,
        commit,
        path,
        prompt,
        regex,
      }) => {
        assert_eq!(check, ["cargo", "test"]);
        assert_eq!(commit, "rewrite function");
        assert_eq!(path, Path::new("foo/bar"));
        assert_eq!(prompt, "hello %% goodbye");
        assert_eq!(regex.as_str(), r"(?ms)^fn hello\(\) \{.*?^\}$");
      }
      _ => panic!("Expected Job::Replace"),
    }
  }
}
