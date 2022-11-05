use futures::{stream::BoxStream, StreamExt};
use std::{fmt, io};

/// WaitError describes errors when waiting for a log line
#[derive(thiserror::Error, Debug)]
pub enum WaitError {
    /// End of stream without matching log line.
    /// Contains all received lines of the log.
    #[error("EOF without matching log line")]
    EOF(Vec<String>),

    /// IO error while reading from the stream
    #[error("IO error: {0}")]
    Io(
        #[from]
        #[source]
        io::Error,
    ),
}

pub(crate) struct LogStream<'d> {
    inner: BoxStream<'d, Result<String, io::Error>>,
}

impl<'d> fmt::Debug for LogStream<'d> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogStream").finish()
    }
}

impl<'d> LogStream<'d> {
    #[inline]
    pub fn new(stream: BoxStream<'d, Result<String, io::Error>>) -> Self {
        Self { inner: stream }
    }

    #[inline]
    pub async fn wait_for_message(mut self, message: &str) -> Result<(), WaitError> {
        let mut lines = vec![];

        while let Some(line) = self.inner.next().await.transpose()? {
            // TODO: use regex here
            if line.contains(message) {
                return Ok(());
            }
            lines.push(line);
        }

        Err(WaitError::EOF(lines))
    }
}
