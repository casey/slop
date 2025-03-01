use {
  crate::{arguments::Arguments, job::Job, passage::Passage},
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
    path::{Path, PathBuf},
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

mod arguments;
mod highlighter;
mod job;
mod passage;

type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

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
  Arguments::parse().run().await
}
