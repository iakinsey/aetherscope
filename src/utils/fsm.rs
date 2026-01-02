use std::str::Chars;

use crate::types::{error::AppError, structs, traits::object_store::AsyncReadSeek};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

#[derive(PartialEq, Eq)]
enum ParseState {
    ReadNewChar,
    ReadHtmlTag,
    ReadLink,
    Terminate,
}

pub struct UriExtractorFSM {
    uris: Vec<String>,
    state: ParseState,
    buf: Box<dyn AsyncReadSeek + Send + Unpin>,
}

impl UriExtractorFSM {
    fn new(buf: Box<dyn AsyncReadSeek + Send + Unpin>) -> Self {
        Self {
            uris: vec![],
            state: ParseState::ReadNewChar,
            buf,
        }
    }

    pub async fn perform(mut self) -> Result<Vec<String>, AppError> {
        while self.state != ParseState::Terminate {
            self.next().await?;
        }

        return Ok(self.uris);
    }

    async fn next(&mut self) -> Result<(), AppError> {
        match self.state {
            ParseState::ReadNewChar => self.read_new_char().await,
            ParseState::ReadHtmlTag => self.read_html_tag().await,
            ParseState::ReadLink => self.read_link().await,
            ParseState::Terminate => Ok(()),
        }
    }

    async fn read_new_char(&mut self) -> Result<(), AppError> {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////
    // Utility functions
    ////////////////////////////////////////////////////////////////////////////

    async fn read_exact_bytes(&mut self, n: usize) -> Result<Vec<u8>, AppError> {
        let mut buf = vec![0u8; n];
        self.buf.read_exact(&mut buf).await?;
        Ok(buf)
    }

    pub async fn position(&mut self) -> Result<u64, AppError> {
        Ok(self.buf.seek(SeekFrom::Current(0)).await?)
    }

    pub async fn set_position(&mut self, pos: u64) -> Result<(), AppError> {
        self.buf.seek(SeekFrom::Start(pos)).await?;
        Ok(())
    }

    async fn read_until_match(&mut self, chars: Vec<char>) -> Result<bool, AppError> {
        let start = self.position().await?;

        loop {
            // TODO
        }

        unimplemented!();
    }

    ////////////////////////////////////////////////////////////////////////////
    // Html reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_html_tag(&mut self) -> Result<(), AppError> {
        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////
    // Link reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_link(&mut self) -> Result<(), AppError> {
        unimplemented!()
    }
}
