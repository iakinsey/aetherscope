use std::{collections::HashSet, io::ErrorKind, sync::OnceLock};

use crate::{types::error::AppError, utils::fsm::stream_reader::StreamReader};

#[derive(PartialEq, Eq)]
pub enum TitleParseState {
    FindTag,
    ReadHtmlTag,
    ReadTitle,
    Terminate,
}
// TODO terminate if you see the end of head
// TODO terminate if you exceed a number of bytes, somewhere in the 4kb range
static TITLE: OnceLock<Vec<char>> = OnceLock::new();
fn title() -> &'static [char] {
    TITLE.get_or_init(|| vec!['t', 'i', 't', 'l', 'e', '>'])
}

static TAG_TERM: OnceLock<HashSet<char>> = OnceLock::new();
fn tag_term() -> &'static HashSet<char> {
    TAG_TERM.get_or_init(|| ['<', '>'].into_iter().collect())
}

pub struct TitleExtractorFSM {
    reader: StreamReader,
    state: TitleParseState,
    title: String,
}

impl TitleExtractorFSM {
    pub fn new(
        buf: Box<dyn crate::types::traits::object_store::AsyncReadSeek + Send + Unpin>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            reader: StreamReader::new(buf),
            state: TitleParseState::FindTag,
            title: "".to_string(),
        })
    }

    pub async fn perform(mut self) -> Result<String, AppError> {
        while self.state != TitleParseState::Terminate {
            match self.next().await {
                Ok(_) => continue,
                Err(AppError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.state = TitleParseState::Terminate;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(self.title)
    }

    async fn next(&mut self) -> Result<(), AppError> {
        match self.state {
            TitleParseState::FindTag => self.find_tag().await,
            TitleParseState::ReadHtmlTag => {
                self.state = TitleParseState::FindTag;
                self.read_html_tag().await
            }
            TitleParseState::ReadTitle => {
                self.state = TitleParseState::FindTag;
                self.read_title().await
            }
            TitleParseState::Terminate => Ok(()),
        }
    }

    async fn find_tag(&mut self) -> Result<(), AppError> {
        loop {
            if self.reader.read_char().await? == '<' {
                self.state = TitleParseState::ReadHtmlTag;
                break;
            }
        }
        Ok(())
    }

    async fn read_html_tag(&mut self) -> Result<(), AppError> {
        if self.reader.read_until_match(title(), '<', true).await? {
            self.state = TitleParseState::ReadTitle;
        }

        return Ok(());
    }

    async fn read_title(&mut self) -> Result<(), AppError> {
        self.title = self.reader.get_until_term(tag_term()).await?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {

    use super::*;
    use crate::types::traits::object_store::AsyncReadSeek;
    use std::io::Cursor;

    fn reader_from_static_str(s: &'static str) -> Box<dyn AsyncReadSeek + Send + Unpin + 'static> {
        Box::new(Cursor::new(s.as_bytes()))
    }

    #[tokio::test]
    async fn extracts_simple_title() {
        let html = r#"<html><head><title>Hello World</title></head></html>"#;
        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("Hello World", title);
    }

    #[tokio::test]
    async fn ignores_content_before_title() {
        let html = r#"
        garbage garbage
        <div>noise</div>
        <title>Actual Title</title>
    "#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("Actual Title", title);
    }

    #[tokio::test]
    async fn handles_empty_title() {
        let html = r#"<title></title>"#;
        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("", title);
    }

    #[tokio::test]
    async fn stops_at_first_title() {
        let html = r#"
        <title>First</title>
        <title>Second</title>
    "#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("First", title);
    }

    #[tokio::test]
    async fn eof_without_title_returns_empty() {
        let html = r#"<html><body>No title here</body></html>"#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("", title);
    }

    #[tokio::test]
    async fn title_with_special_chars() {
        let html = r#"<title>Rust &amp; Tokio <Test></title>"#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("Rust &amp; Tokio ", title);
    }

    #[tokio::test]
    async fn whitespace_preserved_inside_title() {
        let html = r#"<title>   spaced   out   </title>"#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("   spaced   out   ", title);
    }

    #[tokio::test]
    async fn title_after_multiple_tags() {
        let html = r#"
        <meta name="x">
        <link rel="y">
        <title>Deep Title</title>
    "#;

        let fsm = TitleExtractorFSM::new(reader_from_static_str(html)).unwrap();
        let title = fsm.perform().await.unwrap();

        assert_eq!("Deep Title", title);
    }
}
