//! Heading (setext) is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! heading_setext ::= line *(eol line) eol whitespace_optional (1*'-' | 1*'=') whitespace_optional
//!
//! whitespace ::= 1*space_or_tab
//! whitespace_optional ::= [ whitespace ]
//! line ::= code - eol
//! eol ::= '\r' | '\r\n' | '\n'
//! ```
//!
//! Heading (setext) in markdown relates to the `<h1>` and `<h2>` elements in
//! HTML.
//! See [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements* in the
//! HTML spec][html] for more info.
//!
//! In markdown, it is also possible to create headings with a
//! [heading (atx)][heading_atx] construct.
//! The benefit of setext headings is that their text can include line endings,
//! and by extensions also hard breaks (e.g., with
//! [hard break (escape)][hard_break_escape]).
//! However, their limit is that they cannot form `<h3>` through `<h6>`
//! headings.
//! Due to this limitation, it is recommended to use atx headings.
//!
//! [Thematic breaks][thematic_break] formed with dashes and without whitespace
//! could be interpreted as a heading (setext).
//! Which one forms depends on whether there is text directly in fron of the
//! sequence.
//!
//! > 🏛 **Background**: the word *setext* originates from a small markup
//! > language by Ian Feldman from 1991.
//! > See [*§ Setext* on Wikipedia][wiki-setext] for more info.
//! > The word *atx* originates from a tiny markup language by Aaron Swartz
//! > from 2002.
//! > See [*§ atx, the true structured text format* on `aaronsw.com`][atx] for
//! > more info.
//!
//! ## Tokens
//!
//! *   [`HeadingSetext`][Token::HeadingSetext]
//! *   [`HeadingSetextText`][Token::HeadingSetextText]
//! *   [`HeadingSetextUnderline`][Token::HeadingSetextUnderline]
//!
//! ## References
//!
//! *   [`setext-underline.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/setext-underline.js)
//! *   [*§ 4.3 Setext headings* in `CommonMark`](https://spec.commonmark.org/0.30/#setext-headings)
//!
//! [flow]: crate::content::flow
//! [heading_atx]: crate::construct::heading_atx
//! [thematic_break]: crate::construct::thematic_break
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [html]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [wiki-setext]: https://en.wikipedia.org/wiki/Setext
//! [atx]: http://www.aaronsw.com/2002/atx/

use crate::constant::TAB_SIZE;
use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::token::Token;
use crate::tokenizer::{Code, EventType, State, Tokenizer};
use crate::util::skip::opt_back as skip_opt_back;

/// Kind of underline.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Dash (rank 2) heading.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// alpha
    /// -----
    /// ```
    Dash,

    /// Equals to (rank 1) heading.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// alpha
    /// =====
    /// ```
    EqualsTo,
}

impl Kind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            Kind::Dash => '-',
            Kind::EqualsTo => '=',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `-` or `=`.
    fn from_char(char: char) -> Kind {
        match char {
            '-' => Kind::Dash,
            '=' => Kind::EqualsTo,
            _ => unreachable!("invalid char"),
        }
    }
}

/// At a line ending, presumably an underline.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    let max = if tokenizer.parse_state.constructs.code_indented {
        TAB_SIZE - 1
    } else {
        usize::MAX
    };
    let paragraph_before = !tokenizer.events.is_empty()
        && tokenizer.events[skip_opt_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Token::LineEnding, Token::SpaceOrTab],
        )]
        .token_type
            == Token::Paragraph;

    // Require a paragraph before and do not allow on a lazy line.
    if paragraph_before && !tokenizer.lazy && tokenizer.parse_state.constructs.heading_setext {
        tokenizer.go(space_or_tab_min_max(0, max), before)(tokenizer, code)
    } else {
        State::Nok
    }
}

/// After optional whitespace, presumably an underline.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char(char) if char == '-' || char == '=' => {
            tokenizer.enter(Token::HeadingSetextUnderline);
            inside(tokenizer, code, Kind::from_char(char))
        }
        _ => State::Nok,
    }
}

/// In an underline sequence.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> State {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.consume(code);
            State::Fn(Box::new(move |t, c| inside(t, c, kind)))
        }
        _ => {
            tokenizer.exit(Token::HeadingSetextUnderline);
            tokenizer.attempt_opt(space_or_tab(), after)(tokenizer, code)
        }
    }
}

/// After an underline sequence, after optional whitespace.
///
/// ```markdown
///   | aa
/// > | ==
///       ^
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            tokenizer.register_resolver("heading_setext".to_string(), Box::new(resolve));
            State::Ok(0)
        }
        _ => State::Nok,
    }
}

/// Resolve heading (setext).
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    let mut paragraph_enter = None;
    let mut paragraph_exit = None;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        // Find paragraphs.
        if event.event_type == EventType::Enter {
            if event.token_type == Token::Paragraph {
                paragraph_enter = Some(index);
            }
        } else if event.token_type == Token::Paragraph {
            paragraph_exit = Some(index);
        }
        // We know this is preceded by a paragraph.
        // Otherwise we don’t parse.
        else if event.token_type == Token::HeadingSetextUnderline {
            let enter = paragraph_enter.take().unwrap();
            let exit = paragraph_exit.take().unwrap();

            // Change types of Enter:Paragraph, Exit:Paragraph.
            tokenizer.events[enter].token_type = Token::HeadingSetextText;
            tokenizer.events[exit].token_type = Token::HeadingSetextText;

            // Add Enter:HeadingSetext, Exit:HeadingSetext.
            let mut heading_enter = tokenizer.events[enter].clone();
            heading_enter.token_type = Token::HeadingSetext;
            let mut heading_exit = tokenizer.events[index].clone();
            heading_exit.token_type = Token::HeadingSetext;

            tokenizer.map.add(enter, 0, vec![heading_enter]);
            tokenizer.map.add(index + 1, 0, vec![heading_exit]);
        }

        index += 1;
    }
}
