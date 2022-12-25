use std::io::Write;
use std::rc::Rc;
use clap::arg;
use path_absolutize::Absolutize;
use crate::app::context::{AppContext, get_context_preview, KeyHashType, set_context_key_hash};
use crate::cli::args::AppData;
use crate::cli::runtime::{CommandProcessor, CommandProcessorContext};
use crate::{error, try_decrypt, try_encrypt};
use crate::error::{Error, Result};

macro_rules! command_processor_template {
    ($($name: tt),+) => {
        fn get_command(&self) -> Vec<&'static str> {
            vec![$($name),+]
        }

        fn new() -> Self where Self: Sized {
            Self {}
        }

        fn box_clone(&self) -> Box<dyn CommandProcessor<AppContext>> {
            Box::new((*self).clone())
        }
    };
}
macro_rules! command_processor_nohint {
    () => {
        fn get_hint(&self, _ctx: &mut AppContext, _arguments: &Vec<String>) -> Option<String> {
            None
        }
    };
}
macro_rules! command_processor_filehint {
    () => {
        fn get_hint(&self, ctx: &mut AppContext, arguments: &Vec<String>) -> Option<String> {
            let begin_with = if arguments.len() < 1 { "" } else { &arguments[0].as_str() };

            // TODO: result here!
            let files: Vec<String> = std::fs::read_dir(&ctx.cli_current_path)
                .expect("Cannot read the directory")
                .filter_map(|s| s.ok())
                .map(|s| s.file_name())
                .into_iter()
                .filter_map(|s| s.into_string().ok())
                .filter(|s| s.starts_with(begin_with))
                .collect();

            Some(files.join(" "))
        }
    };
}

#[derive(Debug, Clone)]
pub struct CmdSetKey {}

impl CommandProcessor<AppContext> for CmdSetKey {
    command_processor_template!("set-key");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        set_context_key_hash(ctx)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdUnsetKey {}

impl CommandProcessor<AppContext> for CmdUnsetKey {
    command_processor_template!("unset-key");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        ctx.key_hash = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdSetPreview {}

impl CommandProcessor<AppContext> for CmdSetPreview {
    command_processor_template!("set-preview");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        ctx.data.preview = Some(true);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdUnsetPreview {}

impl CommandProcessor<AppContext> for CmdUnsetPreview {
    command_processor_template!("unset-preview");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        ctx.data.preview = Some(false);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdGetAllParameters {}

impl CommandProcessor<AppContext> for CmdGetAllParameters {
    command_processor_template!("get-all");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        let args = [
            [format!("cli_current_path: {}", ctx.cli_current_path.display())],
            [format!("preview: {}", ctx.data.preview.unwrap_or(false))],
            [format!("key hash: {}", match ctx.key_hash {
                Some(_) => "Set",
                None => "Unset"
            })],
            [format!("keep_original: {}", ctx.data.keep_original)],
        ];
        for arg in args {
            ctx.term.write_line(arg[0].as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdHelp {}

impl CommandProcessor<AppContext> for CmdHelp {
    command_processor_template!("help");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        cmd_context.get_all_command_processors().keys().try_for_each(|s| ctx.term.write_line(s))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdPwd {}

impl CommandProcessor<AppContext> for CmdPwd {
    command_processor_template!("pwd");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        ctx.term.write_line(ctx.cli_current_path.absolutize()?.display().to_string().as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdHistory {}

impl CommandProcessor<AppContext> for CmdHistory {
    command_processor_template!("history");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        cmd_context.history.iter().try_for_each(|s| ctx.term.write_line(s))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdExit {}

impl CommandProcessor<AppContext> for CmdExit {
    command_processor_template!("exit");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        ctx.cli_exit = true;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdLs {}

impl CommandProcessor<AppContext> for CmdLs {
    command_processor_template!("ls");
    command_processor_nohint!();

    fn process_command(&self, ctx: &mut AppContext, cmd_context: &CommandProcessorContext<AppContext>, _command: &str, _arguments: &Vec<String>) -> Result<()> {
        std::fs::read_dir(&ctx.cli_current_path)?.try_for_each(|s| ctx.term.write_line(s?.path().display().to_string().as_str()))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdEncrypt {}

impl CommandProcessor<AppContext> for CmdEncrypt {
    command_processor_template!("e", "encrypt");
    command_processor_filehint!();

    fn process_command(&self, ctx: &mut AppContext, _cmd_context: &CommandProcessorContext<AppContext>, _command: &str, arguments: &Vec<String>) -> Result<()> {
        if arguments.len() < 1 {
            return Err(Error::new_const(error::ErrorKind::InvalidArgument, &"Expected 1 argument"));
        }
        let file_path = &arguments[0];

        try_encrypt(
            file_path.as_ref(),
            match ctx.key_hash {
                None => Err(Error::new_const(error::ErrorKind::InvalidArgument, &"No key hash")),
                Some(v) => Ok(v)
            }?,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdDecrypt {}

impl CommandProcessor<AppContext> for CmdDecrypt {
    command_processor_template!("d", "decrypt");
    command_processor_filehint!();

    fn process_command(&self, ctx: &mut AppContext, _cmd_context: &CommandProcessorContext<AppContext>, _command: &str, arguments: &Vec<String>) -> Result<()> {
        if arguments.len() < 1 {
            return Err(Error::new_const(error::ErrorKind::InvalidArgument, &"Expected 1 argument"));
        }
        let file_path = &arguments[0];

        try_decrypt(
            file_path.as_ref(),
            match ctx.key_hash {
                None => Err(Error::new_const(error::ErrorKind::InvalidArgument, &"No key hash")),
                Some(v) => Ok(v)
            }?,
            get_context_preview(ctx)?,
        )?;
        Ok(())
    }
}

pub fn register_all_commands(cmd_context: &mut CommandProcessorContext<AppContext>) {
    let commands: [Box<dyn CommandProcessor<AppContext>>; 12] = [
        Box::from(CmdSetKey::new()),
        Box::from(CmdUnsetKey::new()),
        Box::from(CmdSetPreview::new()),
        Box::from(CmdUnsetPreview::new()),
        Box::from(CmdGetAllParameters::new()),
        Box::from(CmdEncrypt::new()),
        Box::from(CmdDecrypt::new()),
        Box::from(CmdHistory::new()),
        Box::from(CmdLs::new()),
        Box::from(CmdPwd::new()),
        Box::from(CmdHelp::new()),
        Box::from(CmdExit::new()),
    ];

    for command in commands {
        for command_name in (&command).get_command() {
            cmd_context.register_command_processor(
                command_name,
                command.box_clone(),
            );
        }
    }
}