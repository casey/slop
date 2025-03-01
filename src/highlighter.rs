use {super::*, comrak::Options};

#[allow(unused)]
struct Highlighter {
  syntax_set: SyntaxSet,
  theme_set: ThemeSet,
}

impl Highlighter {
  #[allow(unused)]
  fn highlight(&self, markdown: &str) -> Result<String> {
    let arena = Arena::new();

    let options = ComrakOptions::default();

    let root = parse_document(&arena, markdown, &options);

    let mut queue = VecDeque::new();

    queue.push_back(root);

    while let Some(node) = queue.pop_front() {
      if let NodeValue::CodeBlock(ref mut block) = node.data.borrow_mut().value {
        dbg!(&block.info);
        if let Some(syntax) = self.syntax_set.find_syntax_by_token(&block.info) {
          let mut highlight_lines =
            HighlightLines::new(syntax, &self.theme_set.themes["Solarized (dark)"]);

          let mut highlighted = String::new();
          for line in LinesWithEndings::from(&block.literal) {
            let ranges: Vec<(Style, &str)> = highlight_lines
              .highlight_line(line, &self.syntax_set)
              .unwrap();
            highlighted += &syntect::util::as_24_bit_terminal_escaped(&ranges, true);
          }

          block.literal = highlighted + "\x1b[0m";
        }
      }

      for node in node.children() {
        queue.push_back(node);
      }
    }

    let mut output = Cursor::new(Vec::new());

    comrak::format_commonmark(root, &Options::default(), &mut output)?;

    Ok(String::from_utf8(output.into_inner()).unwrap())
  }

  #[allow(unused)]
  fn new() -> Self {
    Self {
      syntax_set: SyntaxSet::load_defaults_newlines(),
      theme_set: ThemeSet::load_defaults(),
    }
  }
}
