use std::{collections::HashSet, io::ErrorKind, str::from_utf8};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

use crate::types::{error::AppError, traits::object_store::AsyncReadSeek};

pub struct StreamReader {
    buf: Box<dyn AsyncReadSeek + Send + Unpin>,
}

impl StreamReader {
    pub fn new(buf: Box<dyn AsyncReadSeek + Send + Unpin>) -> Self {
        Self { buf }
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

    pub async fn match_next(&mut self, pattern: &[char], rewind: bool) -> Result<bool, AppError> {
        let position = self.position().await?;
        let mut matches = true;

        for &c in pattern {
            let next = self.read_char().await?;
            if next != c {
                matches = false;
                if rewind {
                    self.set_position(position).await?;
                }
                break;
            }
        }

        Ok(matches)
    }

    pub async fn match_next_or(
        &mut self,
        chars: &HashSet<char>,
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

    pub async fn get_until_mismatch(
        &mut self,
        legal_chars: &HashSet<char>,
    ) -> Result<String, AppError> {
        let mut result = vec![];

        loop {
            let next = match self.read_char().await {
                Ok(n) => n,
                Err(AppError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            if legal_chars.contains(&next) {
                result.push(next);
            } else {
                break;
            }
        }

        Ok(String::from_iter(result))
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
}
