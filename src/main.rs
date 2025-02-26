use {
  clap::Parser,
  comrak::{Arena, ComrakOptions, nodes::NodeValue, parse_document},
  llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
    error::LLMError,
  },
  regex::Regex,
  serde::Deserialize,
  serde_with::{DisplayFromStr, serde_as},
  std::{
    collections::VecDeque,
    error::Error,
    fs::{self, File},
    io::Cursor,
    path::PathBuf,
    process::Command,
  },
  syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
  },
  walkdir::WalkDir,
};

mod highlighter;

type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

struct Match {
  end: usize,
  path: PathBuf,
  start: usize,
  text: String,
}

impl Match {
  fn as_str(&self) -> &str {
    &self.text[self.start..self.end]
  }

  fn replace(&self, replacement: &str) -> String {
    format!(
      "{}{replacement}{}",
      &self.text[..self.start],
      &self.text[self.end..],
    )
  }
}

#[derive(Parser)]
struct Arguments {
  #[clap(long)]
  job: PathBuf,
}

#[serde_as]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
  path: PathBuf,
  #[serde_as(as = "DisplayFromStr")]
  regex: Regex,
  prompt: String,
  check: Vec<String>,
  commit: String,
}

impl Job {
  fn find(&self) -> Result<Option<Match>> {
    for entry in WalkDir::new(&self.path) {
      let entry = entry?;
      let path = entry.path();

      if entry.file_type().is_dir()
        || !path
          .extension()
          .map(|extension| extension == "rs")
          .unwrap_or_default()
      {
        continue;
      }

      let text = fs::read_to_string(path)?;

      if let Some(capture) = self.regex.find(&text) {
        return Ok(Some(Match {
          path: path.into(),
          start: capture.start(),
          end: capture.end(),
          text,
        }));
      }
    }

    Ok(None)
  }

  fn prompt(&self, m: &Match) -> String {
    self.prompt.replace("%%", &m.text[m.start..m.end])
  }
}

#[allow(unused)]
fn extract_replacement(markdown: &str) -> Result<String> {
  let arena = Arena::new();

  let options = ComrakOptions::default();

  let root = parse_document(&arena, markdown, &options);

  let mut queue = VecDeque::new();

  queue.push_back(root);

  let mut code_blocks = Vec::new();

  while let Some(node) = queue.pop_front() {
    if let NodeValue::CodeBlock(block) = &node.data.borrow().value {
      code_blocks.push(block.literal.clone());
    }

    for node in node.children() {
      queue.push_back(node);
    }
  }

  if code_blocks.is_empty() {
    Ok(markdown.into())
  } else if code_blocks.len() == 1 {
    Ok(code_blocks.pop().unwrap())
  } else {
    eprintln!("{markdown}");
    Err(format!("{} code blocks found", code_blocks.len()).into())
  }
}

#[tokio::main]
async fn main() -> Result {
  let arguments = Arguments::parse();

  let job: Job = serde_yaml::from_reader(File::open(arguments.job)?)?;

  let api_key = fs::read_to_string(dirs::home_dir().unwrap().join(".slop"))?
    .trim()
    .to_owned();

  let llm = LLMBuilder::new()
    .backend(LLMBackend::Anthropic)
    .model("claude-3-7-sonnet-20250219")
    .api_key(api_key)
    .build()?;

  while let Some(m) = job.find()? {
    eprintln!("Found match:\n\n{}\n", m.as_str());

    let prompt = job.prompt(&m);

    let messages = vec![ChatMessage::user().content(prompt).build()];

    let response = match llm.chat(&messages).await {
      Ok(response) => response,
      Err(LLMError::HttpError(err)) if err.contains("529") => continue,
      Err(err) => return Err(err.into()),
    };

    let replacement = response.to_string();

    eprintln!("Replacement:\n\n{replacement}\n");

    fs::write(&m.path, m.replace(&replacement))?;

    eprintln!("Running check…");

    let status = Command::new(&job.check[0])
      .args(&job.check[1..])
      .current_dir(&job.path)
      .status()?;

    if !status.success() {
      return Err("Check failed:".into());
    }

    eprintln!("Comitting…");

    let status = Command::new("git")
      .arg("commit")
      .arg("--message")
      .arg(&job.commit)
      .arg("--")
      .arg(m.path.strip_prefix(&job.path).unwrap())
      .current_dir(&job.path)
      .status()?;

    if !status.success() {
      return Err("Commit failed:".into());
    }
  }

  Ok(())
}
