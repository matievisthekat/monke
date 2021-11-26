pub(crate) mod marker;

mod parse_error;
pub(crate) use parse_error::ParseError;

use crate::event::Event;
use crate::grammar;
use crate::source::Source;
use lexer::Token;
use marker::Marker;
use std::mem;
use syntax::SyntaxKind;

const RECOVERY_SET: [SyntaxKind; 1] = [SyntaxKind::SetKw];

pub(crate) struct Parser<'t, 'input> {
  source: Source<'t, 'input>,
  events: Vec<Event>,
  expected_kinds: Vec<SyntaxKind>,
}

impl<'t, 'input> Parser<'t, 'input> {
  pub(crate) fn new(source: Source<'t, 'input>) -> Self {
    Self {
      source,
      events: Vec::new(),
      expected_kinds: Vec::new(),
    }
  }

  pub(crate) fn parse(mut self) -> Vec<Event> {
    grammar::root(&mut self);
    self.events
  }

  pub(crate) fn start(&mut self) -> Marker {
    let pos = self.events.len();
    self.events.push(Event::Placeholder);
    Marker::new(pos)
  }

  pub(crate) fn bump(&mut self) {
    self.expected_kinds.clear();
    self.source.peek_token().unwrap();
    self.events.push(Event::AddToken);
  }
  pub(crate) fn expect(&mut self, kind: SyntaxKind) {
    if self.at(kind) {
      self.bump();
    } else {
      self.error();
    }
  }

  pub(crate) fn error(&mut self) {
    let current_token = self.source.peek_token();

    let (found, range) = if let Some(Token { kind, range, .. }) = current_token {
      (Some((*kind).into()), *range)
    } else {
      // If we’re at the end of the input we use the range of the very last token in the
      // input.
      (None, self.source.last_token_range().unwrap())
    };

    self.events.push(Event::Error(ParseError {
      expected: mem::take(&mut self.expected_kinds),
      found,
      range,
    }));

    if !self.at_set(&RECOVERY_SET) && !self.at_end() {
      let m = self.start();
      self.bump();
      m.complete(self, SyntaxKind::Error);
    }
  }

  pub(crate) fn at(&mut self, kind: SyntaxKind) -> bool {
    self.expected_kinds.push(kind);
    self.peek() == Some(kind)
  }

  fn at_set(&mut self, set: &[SyntaxKind]) -> bool {
    self.peek().map_or(false, |k| set.contains(&k))
  }

  pub(crate) fn at_end(&mut self) -> bool {
    self.peek().is_none()
  }

  fn peek(&mut self) -> Option<SyntaxKind> {
    self.source.peek_kind()
  }
}

#[cfg(test)]
mod tests {
  use crate::check;
  use expect_test::expect;

  #[test]
  fn parse_nothing() {
    check("", expect![[r#"Root@0..0"#]]);
  }

  #[test]
  fn parse_whitespace() {
    check(
      "   ",
      expect![[r#"
Root@0..3
  Whitespace@0..3 "   ""#]],
    );
  }

  #[test]
  fn parse_comment() {
    check(
      "# hello!",
      expect![[r##"
Root@0..8
  Comment@0..8 "# hello!""##]],
    );
  }
}
