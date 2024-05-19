use crate::bank::Bank;
use crate::server::command::{parse_command, Command, ParseError};
use std::io::Write;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::oneshot::{channel, Sender},
};

#[derive(Default, Clone, Debug)]
pub struct Context {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn handle_quit<W: AsyncWriteExt + Unpin>(writer: &mut W) -> Result<()> {
    writer.write_all("Bye bye\n\n".as_bytes()).await?;

    Ok(())
}

async fn handle_help<W: AsyncWriteExt + Unpin>(writer: &mut W) -> Result<()> {
    let help = br"Supported commands:
  new_bank
  change_bank <bank_id>
  restore_bank <bank_id>
  which_bank
  register_account <balance>
  new_account <balance> - alias for register_account
  get_balance <account_id>
  deposit <account_id> <amount>
  withdraw <account_id> <amount>
  transfer <sender_account_id> <receiver_account_id> <amount>
  list_account_operations <account_id>
  get_account_operations <account_id> - alias for list_account_operations
  list_all_operations
  get_all_operations - alias for list_all_operations
  quit

";
    writer.write_all(help).await?;

    Ok(())
}

async fn handle_command<W: AsyncWriteExt + Unpin>(
    sender: &UnboundedSender<(Command, Sender<String>)>,
    command: &Command,
    writer: &mut W,
) -> Result<()> {
    match *command {
        Command::Quit => handle_quit(writer).await?,
        Command::Help => handle_help(writer).await?,
        _ => {
            let (response_sender, response_receiver) = channel::<String>();
            sender.send((*command, response_sender))?;
            let response = response_receiver.await?;
            writer.write_all(response.as_bytes()).await?;
        }
    };

    Ok(())
}

async fn handle_parse_error<W: AsyncWriteExt + Unpin>(
    e: ParseError,
    command: &str,
    writer: &mut W,
) -> Result<()> {
    writer
        .write_all(
            format!(
                "Command: {}\nStatus: error\nType: parse\nError: {}\n\n",
                command.trim(),
                e
            )
            .as_bytes(),
        )
        .await?;

    Ok(())
}

pub async fn handle<Reader, Writer, Terminal>(
    sender: &UnboundedSender<(Command, Sender<String>)>,
    reader: Reader,
    writer: &mut Writer,
    terminal: &mut Terminal,
) -> Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
    Terminal: Write,
{
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                terminal.write_all("Client disconnected\n".as_bytes())?;
                break;
            }
            Ok(_) => match parse_command(&line) {
                Ok(command) => {
                    handle_command(sender, &command, writer).await?;
                    if command == Command::Quit {
                        terminal.write_all("Client quited\n".as_bytes())?;
                        break;
                    }
                }
                Err(e) => handle_parse_error(e, &line, writer).await?,
            },
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // just ignore invalid data
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn unknown_command_works() {
        let mut terminal = Vec::new();

        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "test_command".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            "Command: test_command\nStatus: error\nType: parse\nError: unknown command\n\n",
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_empty_command_works() {
        let mut terminal = Vec::new();
        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client disconnected\n".to_owned()
        );
    }

    #[tokio::test]
    async fn handle_quit_command_works() {
        let mut terminal = Vec::new();
        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "quit".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client quited\n".to_owned()
        );

        assert_eq!("Bye bye\n\n", from_utf8(writer.as_slice()).unwrap());
    }

    #[tokio::test]
    async fn handle_any_other_legal_command_works() {
        let mut terminal = Vec::new();
        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "new_bank".as_bytes();
        let mut writer = Vec::new();

        tokio::spawn(async move {
            let (command, response_sender) = receiver.recv().await.unwrap();
            assert_eq!(command, Command::NewBank);
            response_sender
                .send("Response from command actor\n\n".to_owned())
                .unwrap();
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            "Response from command actor\n\n",
            from_utf8(writer.as_slice()).unwrap()
        );
    }
}
