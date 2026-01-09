use std::{collections::HashSet, io::ErrorKind, str::from_utf8, sync::OnceLock};

use crate::types::{error::AppError, traits::object_store::AsyncReadSeek};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

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

    ////////////////////////////////////////////////////////////////////////////
    // Html reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_html_tag(&mut self) -> Result<(), AppError> {
        let mut data = vec![];

        if !self.match_next(vec!['t', 't', 'p'], true).await? {
            self.state = ParseState::ReadNewChar;
            return Ok(());
        }

        let next = self.match_next_or(follows_http(), true).await?;

        if let Some(next) = next {
            data.push("http");

            if next == 's' {
                data.push("s");
            }
        } else {
            self.state = ParseState::ReadNewChar;
            return Ok(());
        }

        unimplemented!()
    }

    ////////////////////////////////////////////////////////////////////////////
    // Link reading states
    ////////////////////////////////////////////////////////////////////////////

    async fn read_link(&mut self) -> Result<(), AppError> {
        unimplemented!()
    }
}
