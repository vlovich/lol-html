use crate::base::Bytes;
use crate::transform_stream::Serialize;
use encoding_rs::Encoding;
use failure::Error;

#[derive(Fail, Debug, PartialEq, Copy, Clone)]
pub enum CommentTextError {
    #[fail(display = "Comment text shouldn't contain comment closing sequence (`-->`).")]
    CommentClosingSequence,
    #[fail(display = "Comment text contains a character that can't \
                      be represented in the document's character encoding.")]
    UnencodableCharacter,
}

#[derive(Debug)]
pub struct Comment<'i> {
    text: Bytes<'i>,
    raw: Option<Bytes<'i>>,
    encoding: &'static Encoding,
}

impl<'i> Comment<'i> {
    pub(in crate::token) fn new_parsed(
        text: Bytes<'i>,
        raw: Bytes<'i>,
        encoding: &'static Encoding,
    ) -> Self {
        Comment {
            text,
            raw: Some(raw),
            encoding,
        }
    }

    pub(in crate::token) fn try_from(
        text: &str,
        encoding: &'static Encoding,
    ) -> Result<Self, Error> {
        Ok(Comment {
            text: Comment::text_from_str(text, encoding)?,
            raw: None,
            encoding,
        })
    }

    #[inline]
    pub fn text(&self) -> String {
        self.text.as_string(self.encoding)
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) -> Result<(), Error> {
        self.text = Comment::text_from_str(text, self.encoding)?;
        self.raw = None;

        Ok(())
    }

    // NOTE: not a trait implementation due to the `Borrow` constraint for
    // the `Owned` associated type.
    // See: https://github.com/rust-lang/rust/issues/44950
    #[inline]
    pub fn to_owned(&self) -> Comment<'static> {
        Comment {
            text: self.text.to_owned(),
            raw: self.raw.as_ref().map(|r| r.to_owned()),
            encoding: self.encoding,
        }
    }

    fn text_from_str(text: &str, encoding: &'static Encoding) -> Result<Bytes<'static>, Error> {
        if text.find("-->").is_some() {
            Err(CommentTextError::CommentClosingSequence.into())
        } else {
            // NOTE: if character can't be represented in the given
            // encoding then encoding_rs replaces it with a numeric
            // character reference. Character references are not
            // supported in comments, so we need to bail.
            match Bytes::from_str_without_replacements(text, encoding) {
                Some(name) => Ok(name.into_owned()),
                None => Err(CommentTextError::UnencodableCharacter.into()),
            }
        }
    }
}

impl Serialize for Comment<'_> {
    #[inline]
    fn raw(&self) -> Option<&Bytes<'_>> {
        self.raw.as_ref()
    }

    #[inline]
    fn serialize_from_parts(&self, handler: &mut dyn FnMut(&Bytes<'_>)) {
        handler(&b"<!--".into());
        handler(&self.text);
        handler(&b"-->".into());
    }
}
