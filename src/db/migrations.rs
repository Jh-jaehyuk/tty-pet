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

    ensure_column(
        connection,
        "project_pet_state",
        "custom_image_path",
        "text null",
    )?;
    ensure_column(
        connection,
        "project_pet_state",
        "custom_image_width",
        "integer null",
    )?;
    ensure_column(
        connection,
        "project_pet_state",
        "custom_image_height_scale",
        "real null",
    )?;
    ensure_column(
        connection,
        "project_pet_state",
        "custom_image_charset",
        "text null",
    )?;
    ensure_column(
        connection,
        "project_pet_state",
        "custom_image_invert",
        "integer null",
    )?;

    Ok(())
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    column_definition: &str,
) -> Result<()> {
    let existing_column = connection
        .prepare(&format!("pragma table_info({table})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .find_map(|name| match name {
            Ok(name) if name == column => Some(Ok(name)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?;

    if existing_column.is_some() {
        return Ok(());
    }

    connection.execute(
        &format!("alter table {table} add column {column} {column_definition}"),
        [],
    )?;

    Ok(())
}
