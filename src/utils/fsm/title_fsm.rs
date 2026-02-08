use std::{io::ErrorKind, sync::OnceLock};

use crate::{types::error::AppError, utils::fsm::stream_reader::StreamReader};

#[derive(PartialEq, Eq)]
pub enum TitleParseState {
    FindTag,
    ReadHtmlTag,
    Terminate,
}

static TITLE: OnceLock<Vec<char>> = OnceLock::new();
fn title() -> &'static [char] {
    TITLE.get_or_init(|| vec!['t', 'i', 't', 'l', 'e', '>'])
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
        if !self.reader.match_next(&['a'], true).await? {
            return Ok(());
        }

        if !self.reader.read_until_match(title(), '>', true).await? {
            return Ok(());
        }

        unimplemented!()
    }
}
