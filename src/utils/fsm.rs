use std::{io::ErrorKind, str::Chars};

use crate::types::{error::AppError, traits::object_store::AsyncReadSeek};
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
        loop {
            match self.read_exact_chars(1).await?.as_slice() {
                ['h'] => {
                    self.state = ParseState::ReadLink;
                    break;
                }
                ['<'] => {
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

            match str::from_utf8(&buf[..len]) {
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

    pub async fn position(&mut self) -> Result<u64, AppError> {
        Ok(self.buf.seek(SeekFrom::Current(0)).await?)
    }

    pub async fn set_position(&mut self, pos: u64) -> Result<(), AppError> {
        self.buf.seek(SeekFrom::Start(pos)).await?;
        Ok(())
    }

    pub async fn match_next(&mut self, pattern: Vec<char>, rewind: bool) -> Result<bool, AppError> {
        let position = self.position().await?;
        let mut matches = true;

        for char in pattern {
            let next = self.read_exact_chars(1).await?;

            if next.as_slice() == [char] {
                continue;
            }

            matches = false;
            break;
        }

        if rewind {
            self.set_position(position).await?;
        }

        Ok(matches)
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
