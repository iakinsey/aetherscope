use std::{collections::HashSet, io::ErrorKind, str::from_utf8, sync::OnceLock, thread::current};

use crate::{
    types::{error::AppError, traits::object_store::AsyncReadSeek},
    utils::web::normalize_url,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use url::Url;

#[derive(PartialEq, Eq)]
enum ParseState {
    ReadNewChar,
    ReadHtmlTag,
    ReadLink,
    Terminate,
}

////////////////////////////////////////////////////////////////////////////////
// Patterns
////////////////////////////////////////////////////////////////////////////////

static FOLLOWS_HTTP: OnceLock<HashSet<char>> = OnceLock::new();

fn follows_http() -> &'static HashSet<char> {
    FOLLOWS_HTTP.get_or_init(|| ['s', ':'].into_iter().collect())
}

static LEGAL_URL_CHARS: OnceLock<HashSet<char>> = OnceLock::new();

fn legal_url_chars() -> &'static HashSet<char> {
    LEGAL_URL_CHARS.get_or_init(|| {
        [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
            'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
            'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '.', '_', '~', ':', '/',
            '?', '#', '[', ']', '@', '!', '$', '%', '&', '\'', '(', ')', '*', '+', ',', ';', '=',
        ]
        .into_iter()
        .collect()
    })
}

static HREF: OnceLock<&[char]> = OnceLock::new();

fn href() -> &'static [char] {
    HREF.get_or_init(|| &['h', 'r', 'e', 'f', '='])
}

static TAG_QUOTES: OnceLock<HashSet<char>> = OnceLock::new();

fn tag_quotes() -> &'static HashSet<char> {
    TAG_QUOTES.get_or_init(|| ['"', '\''].into_iter().collect())
}

pub struct UriExtractorFSM {
    uris: Vec<String>,
    state: ParseState,
    buf: Box<dyn AsyncReadSeek + Send + Unpin>,
    origin: Url,
}

impl UriExtractorFSM {
    pub fn new(
        buf: Box<dyn AsyncReadSeek + Send + Unpin>,
        origin: String,
    ) -> Result<Self, AppError> {
        let origin = Url::parse(&origin)?;

        Ok(Self {
            uris: vec![],
            state: ParseState::ReadNewChar,
            buf,
            origin,
        })
    }

    pub async fn perform(mut self) -> Result<Vec<String>, AppError> {
        while self.state != ParseState::Terminate {
            match self.next().await {
                Ok(_) => continue,
                Err(AppError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.state = ParseState::Terminate;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        self.uris.sort_unstable();
        self.uris.dedup();

        return Ok(self.uris);
    }

    async fn next(&mut self) -> Result<(), AppError> {
        match self.state {
            ParseState::ReadNewChar => self.read_new_char().await,
            ParseState::ReadHtmlTag => {
                self.state = ParseState::ReadNewChar;
                self.read_html_tag().await
            }
            ParseState::ReadLink => {
                self.state = ParseState::ReadNewChar;
                self.read_link().await
            }
            ParseState::Terminate => Ok(()),
        }
    }

    async fn read_new_char(&mut self) -> Result<(), AppError> {
        loop {
            match self.read_char().await? {
                'h' => {
                    self.state = ParseState::ReadLink;
                    break;
                }
                '<' => {
                    self.state = ParseState::ReadHtmlTag;
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Utility functions
    ////////////////////////////////////////////////////////////////////////////

    async fn read_exact_bytes(&mut self, n: usize) -> Result<Vec<u8>, AppError> {
        let mut buf = vec![0u8; n];
        self.buf.read_exact(&mut buf).await?;
        Ok(buf)
    }

    pub async fn read_exact_chars(&mut self, n: usize) -> Result<Vec<char>, AppError> {
        let mut out = Vec::with_capacity(n);
        let mut buf = [0u8; 4];
        let mut len = 0usize;

        while out.len() < n {
            let mut b = [0u8; 1];
            self.buf.read_exact(&mut b).await?;
            buf[len] = b[0];
            len += 1;

            match from_utf8(&buf[..len]) {
                Ok(s) => {
                    let ch = s.chars().next().ok_or(AppError::InvalidUtf8)?;
                    out.push(ch);
                    len = 0;
                }
                Err(e) if e.error_len().is_none() => {
                    if len == 4 {
                        return Err(AppError::InvalidUtf8);
                    }
                }
                Err(_) => return Err(AppError::InvalidUtf8),
            }
        }

        Ok(out)
    }

    pub async fn read_char(&mut self) -> Result<char, AppError> {
        let mut buf = [0u8; 4];
        let mut len = 0usize;

        loop {
            let mut b = [0u8; 1];
            self.buf.read_exact(&mut b).await?;
            buf[len] = b[0];
            len += 1;

            match from_utf8(&buf[..len]) {
                Ok(s) => {
                    let ch = s.chars().next().ok_or(AppError::InvalidUtf8)?;
                    return Ok(ch);
                }
                Err(e) if e.error_len().is_none() => {
                    if len == 4 {
                        return Err(AppError::InvalidUtf8);
                    }
                }
                Err(_) => return Err(AppError::InvalidUtf8),
            }
        }
    }

    pub async fn position(&mut self) -> Result<u64, AppError> {
        Ok(self.buf.seek(SeekFrom::Current(0)).await?)
    }

    pub async fn set_position(&mut self, pos: u64) -> Result<(), AppError> {
        self.buf.seek(SeekFrom::Start(pos)).await?;
        Ok(())
    }

    // Returns true if the next characters in the buffer match against pattern, false if not
    pub async fn match_next(&mut self, pattern: Vec<char>, rewind: bool) -> Result<bool, AppError> {
        let position = self.position().await?;
        let mut matches = true;

        for char in pattern {
            let next = self.read_char().await?;

            if next == char {
                continue;
            }

            matches = false;

            if rewind {
                self.set_position(position).await?;
            }

            break;
        }

        Ok(matches)
    }

    pub async fn get_until_mismatch(
        &mut self,
        legal_chars: &'static HashSet<char>,
    ) -> Result<String, AppError> {
        let mut result = vec![];

        loop {
            let next = match self.read_char().await {
                Ok(n) => n,
                Err(AppError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e),
            };

            if legal_chars.contains(&next) {
                result.push(next);
                continue;
            } else {
                break;
            }
        }

        Ok(String::from_iter(result))
    }

    pub async fn match_next_or(
        &mut self,
        chars: &'static HashSet<char>,
        rewind: bool,
    ) -> Result<Option<char>, AppError> {
        let position = self.position().await?;
        let next = self.read_char().await?;

        if chars.contains(&next) {
            Ok(Some(next))
        } else {
            if rewind {
                self.set_position(position).await?;
            }

            Ok(None)
        }
    }

    pub async fn read_until_match(
        &mut self,
        pattern: &[char],
        term_char: char,
        rewind: bool,
    ) -> Result<bool, AppError> {
        let mut index = 0;
        let position = self.position().await?;

        loop {
            let next = self.read_char().await?;

            if next == term_char {
                if rewind {
                    self.set_position(position).await?;
                }

                return Ok(false);
            }

            if next == pattern[index] {
                index += 1;
                if index == pattern.len() {
                    return Ok(true);
                }
            } else {
                index = 0;
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Html reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_link(&mut self) -> Result<(), AppError> {
        let mut data = vec![];

        if !self.match_next(vec!['t', 't', 'p'], true).await? {
            return Ok(());
        }

        let next = self.match_next_or(follows_http(), true).await?;

        if let Some(next) = next {
            data.push("http");

            if next == 's' {
                data.push("s");

                if !self.match_next(vec![':'], true).await? {
                    return Ok(());
                }
            } else if next != ':' {
                return Ok(());
            }

            data.push(":");
        } else {
            return Ok(());
        }

        if self.match_next(vec!['/', '/'], true).await? {
            data.push("//")
        } else {
            return Ok(());
        }

        let url = self.get_until_mismatch(legal_url_chars()).await?;
        data.push(&url);

        if url.len() > 0 {
            let uri = data.join("");
            self.uris.push(uri);
        }

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Link reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_html_tag(&mut self) -> Result<(), AppError> {
        if !self.match_next(vec!['a'], true).await? {
            return Ok(());
        }

        if !self.read_until_match(href(), '>', true).await? {
            return Ok(());
        }

        if self.match_next_or(tag_quotes(), true).await?.is_none() {
            return Ok(());
        }

        let url = self.get_until_mismatch(legal_url_chars()).await?;

        if url.len() > 0 {
            let url = normalize_url(&self.origin, &url)?;
            self.uris.push(url.to_string());
        }

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use std::io::Cursor;

    use crate::{types::traits::object_store::AsyncReadSeek, utils::fsm::url_fsm::UriExtractorFSM};

    fn reader_from_static_str(s: &'static str) -> Box<dyn AsyncReadSeek + Send + Unpin + 'static> {
        Box::new(Cursor::new(s.as_bytes()))
    }

    #[tokio::test]
    async fn test_html_tags() {
        let expected = vec![
            "http://example.com/test",
            "http://example.com/test/testme",
            "http://testagain.com/",
        ];

        let contents = reader_from_static_str(
            r#"
            I am an html document.
            <a href="/test">Hello world</a>
            <a href="testme">Hello world</a>
            <a tag="h1234" href="testagain.com">Hello world</a>
        "#,
        );
        let extractor =
            UriExtractorFSM::new(contents, "http://example.com/test/".to_string()).unwrap();
        let uris = extractor.perform().await.unwrap();

        assert_eq!(expected, uris);
    }

    #[tokio::test]
    async fn test_links() {
        let expected = vec![
            "http://test.com/a_test?test=test#test",
            "https://testme.com/help",
            "http://example.com",
        ];
        let contents = reader_from_static_str(
            r#"
        The quick brown fox jumps over the lazy dog http://test.com/a_test?test=test#test
        This is ahttps://testme.com/help^terminateshttp://example.com
        "#,
        );

        // TODO start here instead, it seems the url library is not very resilient and some additional
        // parsing may be necessary
        let extractor =
            UriExtractorFSM::new(contents, "http://example.com/test".to_string()).unwrap();
        let uris = extractor.perform().await.unwrap();

        assert_eq!(expected, uris);
    }

    #[tokio::test]
    async fn test_tags_and_links() {
        let expected = vec![
            "http://example.com/test/endpoint",
            "https://testme.com/help",
        ];
        let contents = reader_from_static_str(
            r#"
            This is ahttps://testme.com/help
            <a href="endpoint">Hello</a>
        "#,
        );

        // TODO start here instead, it seems the url library is not very resilient and some additional
        // parsing may be necessary
        let extractor =
            UriExtractorFSM::new(contents, "http://example.com/test/".to_string()).unwrap();
        let uris = extractor.perform().await.unwrap();

        assert_eq!(expected, uris);
    }

    #[tokio::test]
    async fn test_empty_string() {
        let contents = reader_from_static_str("");

        let extractor = UriExtractorFSM::new(contents, "http://example.com".to_string()).unwrap();
        let uris = extractor.perform().await.unwrap();
        let expected: Vec<String> = vec![];

        assert_eq!(expected, uris);
    }

    #[tokio::test]
    async fn test_url_ends_at_file_termination() {
        let url = "http://test.com/a_test?test=test#test";
        let contents = reader_from_static_str(url);

        let extractor = UriExtractorFSM::new(contents, "http://example.com".to_string()).unwrap();
        let uris = extractor.perform().await.unwrap();

        assert_eq!(vec![url], uris);
    }
}
