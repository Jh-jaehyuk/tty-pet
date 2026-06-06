use anyhow::Result;
use rusqlite::Connection;

use crate::db::repository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    Poke,
    Treat,
    Call,
    Nap,
}

impl Interaction {
    pub fn event_kind(self) -> &'static str {
        match self {
            Interaction::Poke => "poke",
            Interaction::Treat => "treat",
            Interaction::Call => "call",
            Interaction::Nap => "nap",
        }
    }

    pub fn bond_delta(self) -> i64 {
        match self {
            Interaction::Poke => 0,
            Interaction::Treat => 2,
            Interaction::Call => 0,
            Interaction::Nap => 0,
        }
    }

    pub fn confirmation(self) -> &'static str {
        match self {
            Interaction::Poke => "tty-pet: poke.",
            Interaction::Treat => "tty-pet: treat delivered.",
            Interaction::Call => "tty-pet: called.",
            Interaction::Nap => "tty-pet: nap time.",
        }
    }
}

pub fn record(connection: &Connection, project_id: &str, interaction: Interaction) -> Result<()> {
    repository::record_event(
        connection,
        project_id,
        interaction.event_kind(),
        None,
        interaction.bond_delta(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treat_increases_bond_more_than_other_interactions() {
        assert_eq!(Interaction::Treat.bond_delta(), 2);
        assert_eq!(Interaction::Poke.bond_delta(), 0);
        assert_eq!(Interaction::Call.bond_delta(), 0);
        assert_eq!(Interaction::Nap.bond_delta(), 0);
    }
}
