use std::{process::Stdio, sync::Arc};

use tokio::{process::Command, sync::RwLock};

use crate::{event::Event, Greeter, Mode};

#[derive(SmartDefault, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerOption {
  #[default]
  Shutdown,
  Reboot,
}

pub async fn power(greeter: &mut Greeter, option: PowerOption) {
  let command = match option {
    PowerOption::Shutdown => greeter.power_shutdown_cmd.as_deref(),
    PowerOption::Reboot => greeter.power_reboot_cmd.as_deref(),
  };
  let command = match command {
    Some(args) if !args.is_empty() => {
      match greeter.power_setsid {
        true => {
          let mut cmd = Command::new("setsid");
          cmd.args(args.split(' '));
          cmd
        }
        false => {
          let mut args = args.split(' ');
          let mut cmd = Command::new(args.next().unwrap_or_default());
          cmd.args(args);
          cmd
        }
      }
    }
    _ => {
      let mut cmd = Command::new("shutdown");
      match option {
        PowerOption::Shutdown => cmd.arg("-h"),
        PowerOption::Reboot => cmd.arg("-r"),
      };
      cmd.arg("now");
      cmd
    }
  };
  let mut command = command;
  command.stdin(Stdio::null());
  command.stdout(Stdio::null());
  command.stderr(Stdio::null());
  if let Some(ref sender) = greeter.events {
    let _ = sender.send(Event::PowerCommand(command)).await;
  }
}

pub enum PowerPostAction {
  Noop,
  ClearScreen,
}

pub async fn run(greeter: &Arc<RwLock<Greeter>>, mut command: Command) -> PowerPostAction {
  tracing::info!("executing power command: {:?}", command);

  greeter.write().await.mode = Mode::Processing;

  let message = match command.output().await {
    Ok(result) => match (result.status, result.stderr) {
      (status, _) if status.success() => None,
      (status, output) => {
        let status = format!("{} {status}", fl!("command_exited"));
        let output = String::from_utf8(output).unwrap_or_default();

        Some(format!("{status}\n{output}"))
      }
    },

    Err(err) => Some(format!("{}: {err}", fl!("command_failed"))),
  };

  tracing::info!("power command exited with: {:?}", message);

  let mode = greeter.read().await.previous_mode;

  let mut greeter = greeter.write().await;

  if message.is_none() {
    PowerPostAction::ClearScreen
  } else {
    greeter.mode = mode;
    greeter.message = message;

    PowerPostAction::Noop
  }
}
