//! The string content type.
//!
//! **String** is a limited [text][] like content type which only allows
//! character escapes and character references.
//! It exists in things such as identifiers (media references, definitions),
//! titles, URLs, code (fenced) info and meta parts.
//!
//! The constructs found in string are:
//!
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]
//!
//! [text]: crate::content::text

use crate::construct::partial_whitespace::resolve_whitespace;
use crate::state::{Name, State};
use crate::tokenizer::Tokenizer;

const MARKERS: [u8; 2] = [b'&', b'\\'];

/// Start of string.
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.register_resolver("whitespace".to_string(), Box::new(resolve));
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(Name::StringBefore)
}

/// Before string.
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'&') => tokenizer.attempt(
            Name::CharacterReferenceStart,
            State::Next(Name::StringBefore),
            State::Next(Name::StringBeforeData),
        ),
        Some(b'\\') => tokenizer.attempt(
            Name::CharacterEscapeStart,
            State::Next(Name::StringBefore),
            State::Next(Name::StringBeforeData),
        ),
        _ => State::Retry(Name::StringBeforeData),
    }
}

/// At data.
pub fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(Name::DataStart, State::Next(Name::StringBefore), State::Nok)
}

/// Resolve whitespace.
pub fn resolve(tokenizer: &mut Tokenizer) {
    resolve_whitespace(tokenizer, false, false);
}
