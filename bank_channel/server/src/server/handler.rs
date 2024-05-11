use crate::bank::Bank;
use crate::server::command::{parse_command, Command, ParseError};
use std::io::{BufRead, Write};
use std::sync::mpsc::{channel, Sender};

#[derive(Default, Clone, Debug)]
pub struct Context {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn handle_quit<W: Write>(writer: &mut W) -> Result<()> {
    writer.write_all("Bye bye\n\n".as_bytes())?;

    Ok(())
}

fn handle_help<W: Write>(writer: &mut W) -> Result<()> {
    writer.write_all("Supported commands:\n".as_bytes())?;
    writer.write_all("  new_bank\n".as_bytes())?;
    writer.write_all("  change_bank <bank_id>\n".as_bytes())?;
    writer.write_all("  restore_bank <bank_id>\n".as_bytes())?;
    writer.write_all("  which_bank\n".as_bytes())?;
    writer.write_all("  register_account <balance>\n".as_bytes())?;
    writer.write_all("  new_account <balance> - alias for register_account\n".as_bytes())?;
    writer.write_all("  get_balance <account_id>\n".as_bytes())?;
    writer.write_all("  deposit <account_id> <amount>\n".as_bytes())?;
    writer.write_all("  withdraw <account_id> <amount>\n".as_bytes())?;
    writer
        .write_all("  transfer <sender_account_id> <receiver_account_id> <amount>\n".as_bytes())?;
    writer.write_all("  list_account_operations <account_id>\n".as_bytes())?;
    writer.write_all(
        "  get_account_operations <account_id> - alias for list_account_operations\n".as_bytes(),
    )?;
    writer.write_all("  list_all_operations\n".as_bytes())?;
    writer.write_all("  get_all_operations - alias for list_all_operations\n".as_bytes())?;
    writer.write_all("  quit\n".as_bytes())?;
    writer.write_all("\n".as_bytes())?;

    Ok(())
}

fn handle_command(
    sender: &Sender<(Command, Sender<String>)>,
    command: &Command,
    writer: &mut impl Write,
) -> Result<()> {
    match *command {
        Command::Quit => handle_quit(writer)?,
        Command::Help => handle_help(writer)?,
        _ => {
            let (response_sender, response_receiver) = channel::<String>();
            sender.send((*command, response_sender))?;
            let response = response_receiver.recv()?;
            writer.write_all(response.as_bytes())?;
        }
    };

    Ok(())
}

fn handle_parse_error(e: ParseError, command: &str, writer: &mut impl Write) -> Result<()> {
    writer.write_all(
        format!(
            "Command: {}\nStatus: error\nType: parse\nError: {}\n\n",
            command.trim(),
            e
        )
        .as_bytes(),
    )?;

    Ok(())
}

pub fn handle<R: BufRead, W: Write, T: Write>(
    sender: &Sender<(Command, Sender<String>)>,
    reader: &mut R,
    writer: &mut W,
    terminal: &mut T,
) -> Result<()> {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    terminal.write_all("Client disconnected\n".as_bytes())?;
                    break;
                }

                match parse_command(&line) {
                    Ok(command) => {
                        handle_command(sender, &command, writer)?;
                        if command == Command::Quit {
                            terminal.write_all("Client quited\n".as_bytes())?;
                            break;
                        }
                    }
                    Err(e) => handle_parse_error(e, &line, writer)?,
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // just ignore invalid data
                continue;
            }
            Err(e) => return Err(e.into()),
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn unknown_command_works() {
        let mut reader = "test_command".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, _) = channel::<(Command, Sender<String>)>();

        handle(&sender, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Command: test_command\nStatus: error\nType: parse\nError: unknown command\n\n"
                .to_owned()
        );
    }

    #[test]
    fn handle_empty_command_works() {
        let mut reader = "".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, _) = channel::<(Command, Sender<String>)>();
        handle(&sender, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client disconnected\n".to_owned()
        );
    }

    #[test]
    fn handle_quit_command_works() {
        let mut reader = "quit".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, _) = channel::<(Command, Sender<String>)>();
        handle(&sender, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Bye bye\n\n".to_owned()
        );
        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client quited\n".to_owned()
        );
    }

    #[test]
    fn handle_any_other_command_works() {
        let mut reader = "new_bank".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, receiver) = channel::<(Command, Sender<String>)>();

        std::thread::spawn(move || {
            let (command, response_sender) = receiver.recv().unwrap();
            assert_eq!(command, Command::NewBank);
            response_sender
                .send("Response from command actor\n\n".to_owned())
                .unwrap();
        });

        handle(&sender, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Response from command actor\n\n".to_owned()
        );
    }
}
