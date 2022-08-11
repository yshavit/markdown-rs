//! Code (text) is a construct that occurs in the [text][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! ; Restriction: the number of markers in the closing sequence must be equal
//! ; to the number of markers in the opening sequence.
//! code_text ::= sequence 1*code sequence
//!
//! sequence ::= 1*'`'
//! ```
//!
//! The above grammar shows that it is not possible to create empty code.
//! It is possible to include grave accents (ticks) in code, by wrapping it
//! in bigger or smaller sequences:
//!
//! ```markdown
//! Include more: `a``b` or include less: ``a`b``.
//! ```
//!
//! When turning markdown into HTML, each line ending is turned into a space.
//!
//! It is also possible to include just one grave accent (tick):
//!
//! ```markdown
//! Include just one: `` ` ``.
//! ```
//!
//! Sequences are “gready”, in that they cannot be preceded or succeeded by
//! more grave accents (ticks).
//! To illustrate:
//!
//! ```markdown
//! Not code: ``x`.
//!
//! Not code: `x``.
//!
//! Escapes work, this is code: \``x`.
//!
//! Escapes work, this is code: `x`\`.
//! ```
//!
//! Yields:
//!
//! ```html
//! <p>Not code: ``x`.</p>
//! <p>Not code: `x``.</p>
//! <p>Escapes work, this is code: `<code>x</code>.</p>
//! <p>Escapes work, this is code: <code>x</code>`.</p>
//! ```
//!
//! That is because, when turning markdown into HTML, the first and last space,
//! if both exist and there is also a non-space in the code, are removed.
//! Line endings, at that stage, are considered as spaces.
//!
//! Code (text) relates to the `<code>` element in HTML.
//! See [*§ 4.5.15 The `code` element*][html-code] in the HTML spec for more
//! info.
//!
//! In markdown, it is possible to create code with the
//! [code (fenced)][code_fenced] or [code (indented)][code_indented] constructs
//! in the [flow][] content type.
//! Compared to code (indented), fenced code is more explicit and more similar
//! to code (text), and it has support for specifying the programming language
//! that the code is in, so it is recommended to use that instead of indented
//! code.
//!
//! ## Tokens
//!
//! *   [`CodeText`][Token::CodeText]
//! *   [`CodeTextData`][Token::CodeTextData]
//! *   [`CodeTextSequence`][Token::CodeTextSequence]
//! *   [`LineEnding`][Token::LineEnding]
//!
//! ## References
//!
//! *   [`code-text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-text.js)
//! *   [*§ 6.1 Code spans* in `CommonMark`](https://spec.commonmark.org/0.30/#code-spans)
//!
//! [flow]: crate::content::flow
//! [text]: crate::content::text
//! [code_indented]: crate::construct::code_indented
//! [code_fenced]: crate::construct::code_fenced
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use crate::state::{Name, State};
use crate::token::Token;
use crate::tokenizer::Tokenizer;

/// Start of code (text).
///
/// ```markdown
/// > | `a`
///     ^
/// > | \`a`
///      ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`')
            if tokenizer.parse_state.constructs.code_text
                && (tokenizer.previous != Some(b'`')
                    || (!tokenizer.events.is_empty()
                        && tokenizer.events[tokenizer.events.len() - 1].token_type
                            == Token::CharacterEscape)) =>
        {
            tokenizer.enter(Token::CodeText);
            tokenizer.enter(Token::CodeTextSequence);
            State::Retry(Name::CodeTextSequenceOpen)
        }
        _ => State::Nok,
    }
}

/// In the opening sequence.
///
/// ```markdown
/// > | `a`
///     ^
/// ```
pub fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'`') = tokenizer.current {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(Name::CodeTextSequenceOpen)
    } else {
        tokenizer.exit(Token::CodeTextSequence);
        State::Retry(Name::CodeTextBetween)
    }
}

/// Between something and something else
///
/// ```markdown
/// > | `a`
///      ^^
/// ```
pub fn between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Next(Name::CodeTextBetween)
        }
        Some(b'`') => {
            tokenizer.enter(Token::CodeTextSequence);
            State::Retry(Name::CodeTextSequenceClose)
        }
        _ => {
            tokenizer.enter(Token::CodeTextData);
            State::Retry(Name::CodeTextData)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | `a`
///      ^
/// ```
pub fn data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'`') => {
            tokenizer.exit(Token::CodeTextData);
            State::Retry(Name::CodeTextBetween)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::CodeTextData)
        }
    }
}

/// In the closing sequence.
///
/// ```markdown
/// > | `a`
///       ^
/// ```
pub fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`') => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.consume();
            State::Next(Name::CodeTextSequenceClose)
        }
        _ => {
            if tokenizer.tokenize_state.size == tokenizer.tokenize_state.size_b {
                tokenizer.exit(Token::CodeTextSequence);
                tokenizer.exit(Token::CodeText);
                tokenizer.tokenize_state.size = 0;
                tokenizer.tokenize_state.size_b = 0;
                State::Ok
            } else {
                let index = tokenizer.events.len();
                tokenizer.exit(Token::CodeTextSequence);
                // More or less accents: mark as data.
                tokenizer.events[index - 1].token_type = Token::CodeTextData;
                tokenizer.events[index].token_type = Token::CodeTextData;
                tokenizer.tokenize_state.size_b = 0;
                State::Retry(Name::CodeTextBetween)
            }
        }
    }
}
