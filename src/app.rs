use anyhow::Result;
use rusqlite::Connection;

use crate::config::{self, AppPaths};
use crate::db::models::CustomImageConfig;
use crate::db::{self, repository};
use crate::interactions::{self, Interaction};
use crate::pet::custom_image::{self, AsciiCharset, CustomImageOptions};
use crate::project::{git, identity, ProjectIdentity};
use crate::tui;

pub struct AppContext {
    pub paths: AppPaths,
    pub project: ProjectIdentity,
}

pub enum ImageAction {
    Set(ImageSetRequest),
    Clear,
    Status,
}

pub struct ImageSetRequest {
    pub path: std::path::PathBuf,
    pub width: u32,
    pub height_scale: f32,
    pub charset: AsciiCharset,
    pub invert: bool,
}

impl AppContext {
    pub fn load() -> Result<Self> {
        let paths = config::resolve_paths()?;
        let project = identity::resolve_current_project()?;

        Ok(Self { paths, project })
    }

    pub fn open_database(&self) -> Result<Connection> {
        db::open(&self.paths.db_path)
    }
}

pub fn watch() -> Result<()> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    repository::ensure_pet_state(&connection, &context.project.id)?;

    tui::terminal::run(context)
}

pub fn mark_test_pass() -> Result<()> {
    mark_test("test_pass", Some("pass"), 1)?;
    println!("tty-pet: marked tests as passed.");
    Ok(())
}

pub fn mark_test_fail() -> Result<()> {
    mark_test("test_fail", Some("fail"), 0)?;
    println!("tty-pet: marked tests as failed.");
    Ok(())
}

pub fn interact(interaction: Interaction) -> Result<()> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    repository::ensure_pet_state(&connection, &context.project.id)?;
    interactions::record(&connection, &context.project.id, interaction)?;
    println!("{}", interaction.confirmation());

    Ok(())
}

pub fn status(debug: bool) -> Result<()> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    let state = repository::ensure_pet_state(&connection, &context.project.id)?;
    let latest_event = repository::latest_event(&connection, &context.project.id)?;
    let dirty_count = git::dirty_count(&context.project.root_path)?;

    println!("project: {}", context.project.display_name());
    println!("root: {}", context.project.root_path.display());
    println!("mood: {}", state.mood);
    println!("bond: {}", state.bond);
    println!(
        "last test: {}",
        state.last_test_status.as_deref().unwrap_or("unknown")
    );
    println!(
        "dirty files: {}",
        dirty_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    );

    if let Some(event) = latest_event {
        println!("last event: {} at {}", event.kind, event.created_at);
    } else {
        println!("last event: none");
    }

    if let Some(custom_image) = &state.custom_image {
        println!("image pet: {}", custom_image.path.display());
        println!(
            "image options: width={}, height-scale={}, charset={}, invert={}",
            custom_image.width,
            custom_image.height_scale,
            custom_image.charset,
            custom_image.invert
        );
    } else {
        println!("image pet: built-in");
    }

    if debug {
        println!("project id: {}", context.project.id);
        println!("database: {}", context.paths.db_path.display());
        if let Some(remote) = &context.project.git_remote_url {
            println!("git remote: {remote}");
        }
    }

    Ok(())
}

pub fn image(action: ImageAction) -> Result<()> {
    match action {
        ImageAction::Set(ImageSetRequest {
            path,
            width,
            height_scale,
            charset,
            invert,
        }) => {
            let path = custom_image::normalized_image_path(path);
            let options = CustomImageOptions {
                width,
                height_scale,
                charset,
                invert,
            };
            let rendered = custom_image::render_path(&path, &options)?;
            let config = CustomImageConfig {
                path,
                width,
                height_scale,
                charset: charset.name().to_string(),
                invert,
            };
            let context = AppContext::load()?;
            let connection = context.open_database()?;

            repository::ensure_project(&connection, &context.project)?;
            repository::ensure_pet_state(&connection, &context.project.id)?;
            repository::set_custom_image(&connection, &context.project.id, &config)?;
            println!(
                "tty-pet: image pet set to {}x{} ASCII.",
                rendered.width, rendered.height
            );
        }
        ImageAction::Clear => {
            let context = AppContext::load()?;
            let connection = context.open_database()?;

            repository::ensure_project(&connection, &context.project)?;
            repository::ensure_pet_state(&connection, &context.project.id)?;
            repository::clear_custom_image(&connection, &context.project.id)?;
            println!("tty-pet: image pet cleared.");
        }
        ImageAction::Status => {
            let context = AppContext::load()?;
            let connection = context.open_database()?;

            repository::ensure_project(&connection, &context.project)?;
            let state = repository::ensure_pet_state(&connection, &context.project.id)?;
            if let Some(custom_image) = state.custom_image {
                println!("image pet: {}", custom_image.path.display());
                println!("width: {}", custom_image.width);
                println!("height-scale: {}", custom_image.height_scale);
                println!("charset: {}", custom_image.charset);
                println!("invert: {}", custom_image.invert);
            } else {
                println!("image pet: built-in");
            }
        }
    }

    Ok(())
}

fn mark_test(kind: &str, test_status: Option<&str>, bond_delta: i64) -> Result<()> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    repository::ensure_pet_state(&connection, &context.project.id)?;
    repository::record_event(
        &connection,
        &context.project.id,
        kind,
        test_status,
        bond_delta,
    )?;

    Ok(())
}
