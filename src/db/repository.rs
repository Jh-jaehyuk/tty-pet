use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::models::{PetState, ProjectEvent};
use crate::project::ProjectIdentity;
use crate::time;

pub fn ensure_project(connection: &Connection, project: &ProjectIdentity) -> Result<()> {
    let now = time::now_unix_seconds().to_string();

    connection.execute(
        "
        insert into projects (id, root_path, git_remote_url, created_at, last_seen_at)
        values (?1, ?2, ?3, ?4, ?4)
        on conflict(id) do update set
            root_path = excluded.root_path,
            git_remote_url = excluded.git_remote_url,
            last_seen_at = excluded.last_seen_at
        ",
        params![
            project.id,
            project.root_path.to_string_lossy(),
            project.git_remote_url,
            now
        ],
    )?;

    Ok(())
}

pub fn ensure_pet_state(connection: &Connection, project_id: &str) -> Result<PetState> {
    let now = time::now_unix_seconds().to_string();

    connection.execute(
        "
        insert or ignore into project_pet_state
            (project_id, bond, mood, updated_at)
        values (?1, 0, 'idle', ?2)
        ",
        params![project_id, now],
    )?;

    pet_state(connection, project_id)
}

pub fn pet_state(connection: &Connection, project_id: &str) -> Result<PetState> {
    let state = connection.query_row(
        "
        select
            project_id,
            bond,
            mood,
            last_test_status,
            last_event_kind,
            last_event_at,
            focus_started_at,
            updated_at
        from project_pet_state
        where project_id = ?1
        ",
        params![project_id],
        |row| {
            Ok(PetState {
                project_id: row.get(0)?,
                bond: row.get(1)?,
                mood: row.get(2)?,
                last_test_status: row.get(3)?,
                last_event_kind: row.get(4)?,
                last_event_at: row.get(5)?,
                focus_started_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )?;

    Ok(state)
}

pub fn latest_event(connection: &Connection, project_id: &str) -> Result<Option<ProjectEvent>> {
    let event = connection
        .query_row(
            "
            select id, project_id, kind, created_at
            from project_events
            where project_id = ?1
            order by id desc
            limit 1
            ",
            params![project_id],
            |row| {
                Ok(ProjectEvent {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    kind: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()?;

    Ok(event)
}

pub fn record_event(
    connection: &Connection,
    project_id: &str,
    kind: &str,
    test_status: Option<&str>,
    bond_delta: i64,
) -> Result<()> {
    let now = time::now_unix_seconds().to_string();

    connection.execute(
        "
        insert into project_events (project_id, kind, created_at)
        values (?1, ?2, ?3)
        ",
        params![project_id, kind, now],
    )?;

    connection.execute(
        "
        update project_pet_state
        set
            bond = max(0, bond + ?2),
            last_test_status = coalesce(?3, last_test_status),
            last_event_kind = ?4,
            last_event_at = ?5,
            updated_at = ?5
        where project_id = ?1
        ",
        params![project_id, bond_delta, test_status, kind, now],
    )?;

    Ok(())
}

pub fn update_mood(connection: &Connection, project_id: &str, mood: &str) -> Result<()> {
    let now = time::now_unix_seconds().to_string();

    connection.execute(
        "
        update project_pet_state
        set mood = ?2, updated_at = ?3
        where project_id = ?1
        ",
        params![project_id, mood, now],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectIdentity;
    use rusqlite::Connection;
    use std::path::PathBuf;

    #[test]
    fn pass_event_updates_only_target_project() {
        let connection = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&connection).unwrap();

        let first = ProjectIdentity {
            id: "first".to_string(),
            root_path: PathBuf::from("/tmp/first"),
            git_remote_url: None,
        };
        let second = ProjectIdentity {
            id: "second".to_string(),
            root_path: PathBuf::from("/tmp/second"),
            git_remote_url: None,
        };

        ensure_project(&connection, &first).unwrap();
        ensure_project(&connection, &second).unwrap();
        ensure_pet_state(&connection, &first.id).unwrap();
        ensure_pet_state(&connection, &second.id).unwrap();
        record_event(&connection, &first.id, "test_pass", Some("pass"), 1).unwrap();

        let first_state = pet_state(&connection, &first.id).unwrap();
        let second_state = pet_state(&connection, &second.id).unwrap();

        assert_eq!(first_state.last_test_status.as_deref(), Some("pass"));
        assert_eq!(first_state.bond, 1);
        assert_eq!(second_state.last_test_status, None);
        assert_eq!(second_state.bond, 0);
    }
}
