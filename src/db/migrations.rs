use anyhow::Result;
use rusqlite::Connection;

pub fn run(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "
        create table if not exists projects (
            id text primary key,
            root_path text not null unique,
            git_remote_url text null,
            created_at text not null,
            last_seen_at text not null
        );

        create table if not exists project_pet_state (
            project_id text primary key references projects(id),
            bond integer not null default 0,
            mood text not null default 'idle',
            last_test_status text null,
            last_event_kind text null,
            last_event_at text null,
            focus_started_at text null,
            updated_at text not null
        );

        create table if not exists project_events (
            id integer primary key autoincrement,
            project_id text not null references projects(id),
            kind text not null,
            created_at text not null
        );
        ",
    )?;

    Ok(())
}
