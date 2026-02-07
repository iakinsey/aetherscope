use std::{collections::HashSet, io::ErrorKind, sync::OnceLock};
use url::Url;

use crate::{
    types::error::AppError,
    utils::{fsm::stream_reader::StreamReader, web::normalize_url},
};

#[derive(PartialEq, Eq)]
enum ParseState {
    ReadNewChar,
    ReadHtmlTag,
    ReadLink,
    Terminate,
}

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

static HREF: OnceLock<Vec<char>> = OnceLock::new();
fn href() -> &'static [char] {
    HREF.get_or_init(|| vec!['h', 'r', 'e', 'f', '='])
}

static TAG_QUOTES: OnceLock<HashSet<char>> = OnceLock::new();
fn tag_quotes() -> &'static HashSet<char> {
    TAG_QUOTES.get_or_init(|| ['"', '\''].into_iter().collect())
}

pub struct UriExtractorFSM {
    reader: StreamReader,
    state: ParseState,
    uris: Vec<String>,
    origin: Url,
}

impl UriExtractorFSM {
    pub fn new(
        buf: Box<dyn crate::types::traits::object_store::AsyncReadSeek + Send + Unpin>,
        origin: String,
    ) -> Result<Self, AppError> {
        Ok(Self {
            reader: StreamReader::new(buf),
            state: ParseState::ReadNewChar,
            uris: vec![],
            origin: Url::parse(&origin)?,
        })
    }

    pub async fn perform(mut self) -> Result<Vec<String>, AppError> {
        while self.state != ParseState::Terminate {
            match self.next().await {
                Ok(_) => continue,
                Err(AppError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.state = ParseState::Terminate;
                }
                Err(e) => return Err(e),
            }
        }

        self.uris.sort_unstable();
        self.uris.dedup();
        Ok(self.uris)
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
            match self.reader.read_char().await? {
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

    async fn read_link(&mut self) -> Result<(), AppError> {
        let mut data = vec![];

        if !self.reader.match_next(&['t', 't', 'p'], true).await? {
            return Ok(());
        }

        let next = self.reader.match_next_or(follows_http(), true).await?;

        if let Some(next) = next {
            data.push("http");

            if next == 's' {
                data.push("s");
                if !self.reader.match_next(&[':'], true).await? {
                    return Ok(());
                }
            } else if next != ':' {
                return Ok(());
            }

            data.push(":");
        } else {
            return Ok(());
        }

        if self.reader.match_next(&['/', '/'], true).await? {
            data.push("//");
        } else {
            return Ok(());
        }

        let url = self.reader.get_until_mismatch(legal_url_chars()).await?;
        data.push(&url);

        if !url.is_empty() {
            self.uris.push(data.join(""));
        }

        Ok(())
    }

    async fn read_html_tag(&mut self) -> Result<(), AppError> {
        if !self.reader.match_next(&['a'], true).await? {
            return Ok(());
        }

        if !self.reader.read_until_match(href(), '>', true).await? {
            return Ok(());
        }

        if self
            .reader
            .match_next_or(tag_quotes(), true)
            .await?
            .is_none()
        {
            return Ok(());
        }

        let url = self.reader.get_until_mismatch(legal_url_chars()).await?;

        if !url.is_empty() {
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
        let mut expected = vec![
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
        let mut uris = extractor.perform().await.unwrap();

        expected.sort();
        uris.sort();

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
