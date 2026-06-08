use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::agent_presentation;
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

#[derive(Debug, Clone, Copy)]
pub struct StatusOptions {
    pub debug: bool,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct StatusReport {
    project: StatusProject,
    state: StatusState,
    pet: StatusPet,
    debug: StatusDebug,
}

#[derive(Debug, Serialize)]
struct StatusProject {
    id: String,
    name: String,
    root_path: String,
    git_remote_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatusState {
    mood: String,
    bond: i64,
    last_test_status: Option<String>,
    dirty_files: Option<usize>,
    last_event: Option<StatusEvent>,
}

#[derive(Debug, Serialize)]
struct StatusEvent {
    kind: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct StatusPet {
    image: StatusImage,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum StatusImage {
    BuiltIn,
    Custom {
        path: String,
        width: u32,
        height_scale: f32,
        charset: String,
        invert: bool,
    },
}

#[derive(Debug, Serialize)]
struct StatusDebug {
    database_path: String,
}

struct CurrentStatus {
    context: AppContext,
    report: StatusReport,
    latest_event: Option<crate::db::models::ProjectEvent>,
    dirty_count: Option<usize>,
    state: crate::db::models::PetState,
}

impl AppContext {
    pub fn load() -> Result<Self> {
        let paths = config::resolve_paths()?;
        let project = match std::env::var_os("TTY_PET_PROJECT_DIR") {
            Some(path) => identity::resolve_project(path.into())?,
            None => identity::resolve_current_project()?,
        };

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

pub fn record_agent_event(kind: &str) -> Result<String> {
    match kind {
        "pass" => {
            mark_test("test_pass", Some("pass"), 1)?;
        }
        "fail" => {
            mark_test("test_fail", Some("fail"), 0)?;
        }
        "poke" => {
            record_interaction(Interaction::Poke)?;
        }
        "treat" => {
            record_interaction(Interaction::Treat)?;
        }
        "call" => {
            record_interaction(Interaction::Call)?;
        }
        "nap" => {
            record_interaction(Interaction::Nap)?;
        }
        _ => anyhow::bail!("unsupported tty-pet event kind: {kind}"),
    };

    let status: serde_json::Value = serde_json::from_str(&status_json_string()?)?;
    Ok(serde_json::to_string_pretty(
        &agent_presentation::event_payload(kind, &status),
    )?)
}

pub fn status(options: StatusOptions) -> Result<()> {
    let current = current_status()?;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&current.report)?);
        return Ok(());
    }

    print_status_text(
        &current.context,
        &current.state,
        current.latest_event.as_ref(),
        current.dirty_count,
        options.debug,
    );

    Ok(())
}

pub fn status_json_string() -> Result<String> {
    let current = current_status()?;

    Ok(serde_json::to_string_pretty(&current.report)?)
}

pub fn agent_status_json_string() -> Result<String> {
    let status: serde_json::Value = serde_json::from_str(&status_json_string()?)?;

    Ok(serde_json::to_string_pretty(
        &agent_presentation::status_payload(&status),
    )?)
}

fn current_status() -> Result<CurrentStatus> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    let state = repository::ensure_pet_state(&connection, &context.project.id)?;
    let latest_event = repository::latest_event(&connection, &context.project.id)?;
    let dirty_count = git::dirty_count(&context.project.root_path)?;
    let report = status_report(&context, &state, latest_event.as_ref(), dirty_count);

    Ok(CurrentStatus {
        context,
        report,
        latest_event,
        dirty_count,
        state,
    })
}

fn print_status_text(
    context: &AppContext,
    state: &crate::db::models::PetState,
    latest_event: Option<&crate::db::models::ProjectEvent>,
    dirty_count: Option<usize>,
    debug: bool,
) {
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
}

fn status_report(
    context: &AppContext,
    state: &crate::db::models::PetState,
    latest_event: Option<&crate::db::models::ProjectEvent>,
    dirty_count: Option<usize>,
) -> StatusReport {
    StatusReport {
        project: StatusProject {
            id: context.project.id.clone(),
            name: context.project.display_name(),
            root_path: context.project.root_path.display().to_string(),
            git_remote_url: context.project.git_remote_url.clone(),
        },
        state: StatusState {
            mood: state.mood.clone(),
            bond: state.bond,
            last_test_status: state.last_test_status.clone(),
            dirty_files: dirty_count,
            last_event: latest_event.map(|event| StatusEvent {
                kind: event.kind.clone(),
                created_at: event.created_at.clone(),
            }),
        },
        pet: StatusPet {
            image: state
                .custom_image
                .as_ref()
                .map_or(StatusImage::BuiltIn, |image| StatusImage::Custom {
                    path: image.path.display().to_string(),
                    width: image.width,
                    height_scale: image.height_scale,
                    charset: image.charset.clone(),
                    invert: image.invert,
                }),
        },
        debug: StatusDebug {
            database_path: context.paths.db_path.display().to_string(),
        },
    }
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

fn record_interaction(interaction: Interaction) -> Result<()> {
    let context = AppContext::load()?;
    let connection = context.open_database()?;

    repository::ensure_project(&connection, &context.project)?;
    repository::ensure_pet_state(&connection, &context.project.id)?;
    interactions::record(&connection, &context.project.id, interaction)?;

    Ok(())
}
