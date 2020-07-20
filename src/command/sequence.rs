//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands

use {
    super::{Command, CommandParts},
    crate::{
        app::AppContext,
        errors::ProgramError,
        verb::PrefixSearchResult,
    },
};

/// an unparsed sequence with its separator (which may be
/// different from the one provided by local_separator())
#[derive(Debug)]
pub struct Sequence {
    pub separator: String,
    pub raw: String,
}

impl Sequence {
    /// return the separator to use to parse sequences.
    pub fn local_separator() -> String {
        match std::env::var("BROOT_CMD_SEPARATOR") {
            Ok(sep) if !sep.is_empty() => sep,
            _ => String::from(";"),
        }
    }
    pub fn new(separator: String, raw: String) -> Self {
        Self {
            separator,
            raw,
        }
    }
    pub fn new_single(cmd: String) -> Self {
        Self {
            separator: "".to_string(),
            raw: cmd,
        }
    }
    pub fn new_local(raw: String) -> Self {
        Self {
            separator: Self::local_separator(),
            raw,
        }
    }
    pub fn parse(
        &self,
        con: &AppContext,
    ) -> Result<Vec<(String, Command)>, ProgramError> {
        debug!("Splitting cmd sequence with {:?}", &self.separator);
        let mut commands = Vec::new();
        if self.separator.is_empty() {
            add_commands(&self.raw, &mut commands, con)?;
        } else {
            for input in self.raw.split(&self.separator) {
                add_commands(input, &mut commands, con)?;
            }
        }
        Ok(commands)
    }
}

/// an input may be made of two parts:
///  - a search pattern
///  - a verb followed by its arguments
/// we need to build a command for each part so
/// that the search is effectively done before
/// the verb invocation
fn add_commands(
    input: &str,
    commands: &mut Vec<(String, Command)>,
    con: &AppContext,
) -> Result<(), ProgramError> {
    let raw_parts = CommandParts::from(input.to_string());
    let (pattern, verb_invocation) = raw_parts.split();
    if let Some(pattern) = pattern {
        debug!("adding pattern: {:?}", pattern);
        commands.push((input.to_string(), Command::from_parts(pattern, false)));
    }
    if let Some(verb_invocation) = verb_invocation {
        debug!("adding verb_invocation: {:?}", verb_invocation);
        let command = Command::from_parts(verb_invocation, true);
        if let Command::VerbInvocate(invocation) = &command {
            // we check that the verb exists to avoid running a sequence
            // of actions with some missing
            match con.verb_store.search(&invocation.name) {
                PrefixSearchResult::NoMatch => {
                    return Err(ProgramError::UnknownVerb {
                        name: invocation.name.to_string(),
                    });
                }
                PrefixSearchResult::Matches(_) => {
                    return Err(ProgramError::AmbiguousVerbName {
                        name: invocation.name.to_string(),
                    });
                }
                _ => {}
            }
            commands.push((input.to_string(), command));
        }
    }
    Ok(())
}
