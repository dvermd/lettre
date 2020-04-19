//! The sendmail transport sends the email using the local sendmail command.
//!

use crate::{transport::sendmail::error::SendmailResult, Message, Transport};
use log::info;
use std::{
    convert::AsRef,
    fmt::Display,
    io::prelude::*,
    process::{Command, Stdio},
};
use uuid::Uuid;

pub mod error;

/// Sends an email using the `sendmail` command
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendmailTransport {
    command: String,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport {
            command: "/usr/sbin/sendmail".to_string(),
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<String>>(command: S) -> SendmailTransport {
        SendmailTransport {
            command: command.into(),
        }
    }
}

impl<'a, B> Transport<'a, B> for SendmailTransport {
    type Result = SendmailResult;

    fn send(&mut self, email: Message<B>) -> Self::Result
    where
        B: Display,
    {
        let email_id = Uuid::new_v4();

        // Spawn the sendmail command
        let mut process = Command::new(&self.command)
            .arg("-i")
            .arg("-f")
            .arg(
                email
                    .envelope()
                    .from()
                    .map(|f| f.as_ref())
                    .unwrap_or("\"\""),
            )
            .args(email.envelope().to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(email.to_string().as_bytes())?;

        info!("Wrote {} message to stdin", email_id);

        let output = process.wait_with_output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(error::Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}