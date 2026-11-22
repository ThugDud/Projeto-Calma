//! Exportação e importação do estado persistido do Calma.
//!
//! O formato escolhido é JSON versionado. Ele preserva a hierarquia de
//! projetos e tarefas, a recorrência, as conclusões por ocorrência e mantém
//! datas/horas legíveis, sendo fácil de inspecionar ou versionar fora do
//! aplicativo. A importação sempre acontece dentro de uma transação SQLite:
//! ou todo o arquivo é aplicado, ou nada é alterado.

use crate::db;
use crate::errors::{AppError, AppResult};
use crate::models::event::Event;
use crate::models::project::Project;
use crate::models::task::Task;
use chrono::NaiveDateTime;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Identificador do formato de backup do Calma.
pub const BACKUP_FORMAT: &str = "calma-backup";
/// Versão atual do formato de backup.
pub const BACKUP_VERSION: u32 = 1;

/// Conteúdo completo de um arquivo de backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub format: String,
    pub version: u32,
    pub exported_at: NaiveDateTime,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub events: Vec<Event>,
    /// Conclusões de ocorrências de tarefas recorrentes.
    #[serde(default)]
    pub task_completions: Vec<TaskCompletion>,
}

/// Conclusão de uma ocorrência diária/semanal de uma tarefa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCompletion {
    pub task_id: i64,
    pub date: chrono::NaiveDate,
}

/// Modo de aplicação de uma importação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Mantém os dados atuais e só adiciona itens que ainda não existem.
    Add,
    /// Remove todos os projetos, tarefas e eventos atuais antes de importar.
    Overwrite,
}

impl ImportMode {
    pub fn label(self) -> &'static str {
        match self {
            ImportMode::Add => "Importar e adicionar",
            ImportMode::Overwrite => "Importar e sobrescrever",
        }
    }
}

/// Resumo de uma exportação concluída.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSummary {
    pub path: PathBuf,
    pub projects: usize,
    pub tasks: usize,
    pub events: usize,
}

/// Resumo de uma importação concluída.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub projects: usize,
    pub tasks: usize,
    pub events: usize,
    pub duplicates_skipped: usize,
}

/// Caminho padrão sugerido para exportação.
pub fn default_export_path() -> PathBuf {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(format!("calma-backup-{}.json", stamp))
}

/// Expande `~` e `~/...` no caminho informado pelo usuário.
pub fn resolve_user_path(input: &str) -> AppResult<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Backup(
            "Informe o caminho de um arquivo JSON.".to_string(),
        ));
    }

    if trimmed == "~" {
        return Ok(std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")));
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        let base = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return Ok(base.join(rest));
    }

    Ok(PathBuf::from(trimmed))
}

/// Exporta todos os projetos, tarefas e eventos para um arquivo JSON.
pub fn export_to_file(path: &Path, conn: &Connection) -> AppResult<ExportSummary> {
    let projects = db::list_projects(conn)?;
    let tasks = db::list_tasks(conn)?;
    let events = db::list_events(conn)?;
    let task_completions = db::list_task_completions(conn)?
        .into_iter()
        .map(|(task_id, date)| TaskCompletion { task_id, date })
        .collect();

    let data = BackupData {
        format: BACKUP_FORMAT.to_string(),
        version: BACKUP_VERSION,
        exported_at: chrono::Local::now().naive_local(),
        projects,
        tasks,
        events,
        task_completions,
    };

    let json = serde_json::to_string_pretty(&data)
        .map_err(|err| AppError::Backup(format!("Não foi possível gerar o JSON: {}", err)))?;
    write_backup(path, &json)?;

    Ok(ExportSummary {
        path: path.to_path_buf(),
        projects: data.projects.len(),
        tasks: data.tasks.len(),
        events: data.events.len(),
    })
}

/// Importa um arquivo JSON aplicando o modo escolhido.
///
/// A leitura, a validação e a escrita acontecem antes/dentro de uma única
/// transação. Em caso de erro, nenhuma alteração parcial permanece no banco.
pub fn import_from_file(
    path: &Path,
    conn: &Connection,
    mode: ImportMode,
) -> AppResult<ImportSummary> {
    let contents = fs::read_to_string(path).map_err(|err| {
        AppError::Backup(format!("Não foi possível ler {}: {}", path.display(), err))
    })?;

    let data: BackupData = serde_json::from_str(&contents)
        .map_err(|err| AppError::Backup(format!("JSON inválido em {}: {}", path.display(), err)))?;

    validate_backup(&data)?;

    match mode {
        ImportMode::Overwrite => apply_overwrite(conn, &data),
        ImportMode::Add => apply_add(conn, &data),
    }
}

fn write_backup(path: &Path, contents: &str) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                AppError::Backup(format!(
                    "Não foi possível criar o diretório {}: {}",
                    parent.display(),
                    err
                ))
            })?;
        }
    }

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, contents).map_err(|err| {
        AppError::Backup(format!(
            "Não foi possível escrever {}: {}",
            temp_path.display(),
            err
        ))
    })?;

    fs::rename(&temp_path, path).map_err(|err| {
        let _ = fs::remove_file(&temp_path);
        AppError::Backup(format!(
            "Não foi possível salvar {}: {}",
            path.display(),
            err
        ))
    })
}

fn validate_backup(data: &BackupData) -> AppResult<()> {
    if data.format != BACKUP_FORMAT {
        return Err(AppError::Backup(format!(
            "Formato não reconhecido: esperado \"{}\", encontrado \"{}\".",
            BACKUP_FORMAT, data.format
        )));
    }

    if data.version == 0 || data.version > BACKUP_VERSION {
        return Err(AppError::Backup(format!(
            "Versão de backup {} não suportada (máxima: {}).",
            data.version, BACKUP_VERSION
        )));
    }

    let mut project_ids = HashSet::new();
    for project in &data.projects {
        if project.id <= 0 {
            return Err(AppError::Backup(format!(
                "Projeto com id inválido: {}.",
                project.id
            )));
        }
        if !project_ids.insert(project.id) {
            return Err(AppError::Backup(format!(
                "Id de projeto duplicado no arquivo: {}.",
                project.id
            )));
        }
    }

    for project in &data.projects {
        if let Some(parent_id) = project.parent_id {
            if !project_ids.contains(&parent_id) {
                return Err(AppError::Backup(format!(
                    "Projeto \"{}\" referencia um projeto pai inexistente ({}).",
                    project.name, parent_id
                )));
            }
        }
    }

    let mut task_ids = HashSet::new();
    for task in &data.tasks {
        if task.id <= 0 {
            return Err(AppError::Backup(format!(
                "Tarefa com id inválido: {}.",
                task.id
            )));
        }
        if !task_ids.insert(task.id) {
            return Err(AppError::Backup(format!(
                "Id de tarefa duplicado no arquivo: {}.",
                task.id
            )));
        }
    }

    for task in &data.tasks {
        if let Some(parent_id) = task.parent_id {
            if !task_ids.contains(&parent_id) {
                return Err(AppError::Backup(format!(
                    "Tarefa \"{}\" referencia uma tarefa pai inexistente ({}).",
                    task.title, parent_id
                )));
            }
        }
        if let Some(project_id) = task.project_id {
            if !project_ids.contains(&project_id) {
                return Err(AppError::Backup(format!(
                    "Tarefa \"{}\" referencia um projeto inexistente ({}).",
                    task.title, project_id
                )));
            }
        }
    }

    let mut completion_keys = HashSet::new();
    for completion in &data.task_completions {
        if !task_ids.contains(&completion.task_id) {
            return Err(AppError::Backup(format!(
                "Conclusão referencia uma tarefa inexistente ({}).",
                completion.task_id
            )));
        }
        if !completion_keys.insert((completion.task_id, completion.date)) {
            return Err(AppError::Backup(format!(
                "Conclusão duplicada no arquivo para a tarefa {} em {}.",
                completion.task_id, completion.date
            )));
        }
    }

    ordered_projects(data)?;
    ordered_tasks(data)?;

    Ok(())
}

fn apply_overwrite(conn: &Connection, data: &BackupData) -> AppResult<ImportSummary> {
    let tx = conn.unchecked_transaction().map_err(|err| {
        AppError::Database(format!(
            "Não foi possível iniciar a transação de importação: {}",
            err
        ))
    })?;

    tx.execute_batch(
        "DELETE FROM task_completions; DELETE FROM tasks; DELETE FROM events; DELETE FROM projects;",
    )
    .map_err(|err| {
        AppError::Database(format!("Não foi possível limpar os dados atuais: {}", err))
    })?;

    let mut summary = ImportSummary::default();
    let mut project_map = HashMap::new();
    for project in ordered_projects(data)? {
        let mut new_project = project.clone();
        new_project.id = 0;
        new_project.parent_id = project.parent_id.map(|id| project_map[&id]);
        let new_id = db::insert_project(&tx, &new_project)?;
        project_map.insert(project.id, new_id);
        summary.projects += 1;
    }

    let mut task_map = HashMap::new();
    for task in ordered_tasks(data)? {
        let mut new_task = task.clone();
        new_task.id = 0;
        new_task.parent_id = task.parent_id.map(|id| task_map[&id]);
        new_task.project_id = task.project_id.map(|id| project_map[&id]);
        let new_id = db::insert_task(&tx, &new_task)?;
        task_map.insert(task.id, new_id);
        summary.tasks += 1;
    }

    for event in &data.events {
        let mut new_event = event.clone();
        new_event.id = 0;
        db::insert_event(&tx, &new_event)?;
        summary.events += 1;
    }

    for completion in &data.task_completions {
        if let Some(&new_task_id) = task_map.get(&completion.task_id) {
            db::set_task_completed_on(&tx, new_task_id, completion.date, true)?;
        }
    }

    tx.commit().map_err(|err| {
        AppError::Database(format!("Não foi possível concluir a importação: {}", err))
    })?;

    Ok(summary)
}

fn apply_add(conn: &Connection, data: &BackupData) -> AppResult<ImportSummary> {
    let tx = conn.unchecked_transaction().map_err(|err| {
        AppError::Database(format!(
            "Não foi possível iniciar a transação de importação: {}",
            err
        ))
    })?;

    let existing_projects = db::list_projects(&tx)?;
    let mut project_keys: HashMap<String, i64> = existing_projects
        .iter()
        .map(|project| {
            (
                project_key(
                    &project.name,
                    project.parent_id,
                    project.description.as_deref(),
                    project.color.as_deref(),
                ),
                project.id,
            )
        })
        .collect();

    let mut summary = ImportSummary::default();
    let mut project_map = HashMap::new();
    for project in ordered_projects(data)? {
        let parent_id = project.parent_id.map(|id| project_map[&id]);
        let key = project_key(
            &project.name,
            parent_id,
            project.description.as_deref(),
            project.color.as_deref(),
        );

        if let Some(&existing_id) = project_keys.get(&key) {
            project_map.insert(project.id, existing_id);
            summary.duplicates_skipped += 1;
            continue;
        }

        let mut new_project = project.clone();
        new_project.id = 0;
        new_project.parent_id = parent_id;
        let new_id = db::insert_project(&tx, &new_project)?;
        project_keys.insert(key, new_id);
        project_map.insert(project.id, new_id);
        summary.projects += 1;
    }

    let existing_tasks = db::list_tasks(&tx)?;
    let mut task_keys: HashMap<String, i64> = existing_tasks
        .iter()
        .map(|task| (task_key(task, task.parent_id, task.project_id), task.id))
        .collect();

    let mut task_map = HashMap::new();
    for task in ordered_tasks(data)? {
        let parent_id = task.parent_id.map(|id| task_map[&id]);
        let project_id = task.project_id.map(|id| project_map[&id]);
        let key = task_key(task, parent_id, project_id);

        if let Some(&existing_id) = task_keys.get(&key) {
            task_map.insert(task.id, existing_id);
            summary.duplicates_skipped += 1;
            continue;
        }

        let mut new_task = task.clone();
        new_task.id = 0;
        new_task.parent_id = parent_id;
        new_task.project_id = project_id;
        let new_id = db::insert_task(&tx, &new_task)?;
        task_keys.insert(key, new_id);
        task_map.insert(task.id, new_id);
        summary.tasks += 1;
    }

    let existing_events = db::list_events(&tx)?;
    let mut event_keys: HashSet<String> = existing_events.iter().map(event_key).collect();
    for event in &data.events {
        let key = event_key(event);
        if !event_keys.insert(key) {
            summary.duplicates_skipped += 1;
            continue;
        }

        let mut new_event = event.clone();
        new_event.id = 0;
        db::insert_event(&tx, &new_event)?;
        summary.events += 1;
    }

    for completion in &data.task_completions {
        if let Some(&task_id) = task_map.get(&completion.task_id) {
            db::set_task_completed_on(&tx, task_id, completion.date, true)?;
        }
    }

    tx.commit().map_err(|err| {
        AppError::Database(format!("Não foi possível concluir a importação: {}", err))
    })?;

    Ok(summary)
}

fn ordered_projects(data: &BackupData) -> AppResult<Vec<&Project>> {
    let mut remaining: Vec<&Project> = data.projects.iter().collect();
    let mut resolved = HashSet::new();
    let mut ordered = Vec::with_capacity(remaining.len());

    while !remaining.is_empty() {
        let mut next = Vec::new();
        let mut progressed = false;

        for project in remaining {
            let parent_pending = project
                .parent_id
                .is_some_and(|parent_id| !resolved.contains(&parent_id));

            if parent_pending {
                next.push(project);
            } else {
                resolved.insert(project.id);
                ordered.push(project);
                progressed = true;
            }
        }

        if !progressed {
            return Err(AppError::Backup(
                "Ciclo detectado na hierarquia de projetos.".to_string(),
            ));
        }
        remaining = next;
    }

    Ok(ordered)
}

fn ordered_tasks(data: &BackupData) -> AppResult<Vec<&Task>> {
    let mut remaining: Vec<&Task> = data.tasks.iter().collect();
    let mut resolved = HashSet::new();
    let mut ordered = Vec::with_capacity(remaining.len());

    while !remaining.is_empty() {
        let mut next = Vec::new();
        let mut progressed = false;

        for task in remaining {
            let parent_pending = task
                .parent_id
                .is_some_and(|parent_id| !resolved.contains(&parent_id));

            if parent_pending {
                next.push(task);
            } else {
                resolved.insert(task.id);
                ordered.push(task);
                progressed = true;
            }
        }

        if !progressed {
            return Err(AppError::Backup(
                "Ciclo detectado na hierarquia de tarefas.".to_string(),
            ));
        }
        remaining = next;
    }

    Ok(ordered)
}

fn project_key(
    name: &str,
    parent_id: Option<i64>,
    description: Option<&str>,
    color: Option<&str>,
) -> String {
    serde_json::to_string(&(
        normalized(name),
        parent_id,
        normalized_opt(description),
        normalized_opt(color),
    ))
    .unwrap_or_default()
}

fn task_key(task: &Task, parent_id: Option<i64>, project_id: Option<i64>) -> String {
    serde_json::to_string(&(
        normalized(&task.title),
        parent_id,
        project_id,
        task.due_date,
        task.scheduled_date,
        task.scheduled_time,
        task.recurrence,
        task.recurrence_days,
        task.status,
        task.priority,
        normalized_opt(task.category.as_deref()),
        normalized_opt(task.description.as_deref()),
    ))
    .unwrap_or_default()
}

fn event_key(event: &Event) -> String {
    serde_json::to_string(&(
        normalized(&event.title),
        event.date,
        event.start_time,
        event.end_time,
        normalized_opt(event.category.as_deref()),
        normalized_opt(event.location.as_deref()),
        normalized_opt(event.description.as_deref()),
    ))
    .unwrap_or_default()
}

fn normalized(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalized_opt(value: Option<&str>) -> Option<String> {
    value.map(normalized).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::task::{TaskPriority, TaskRecurrence, TaskStatus};
    use chrono::{NaiveDate, NaiveTime};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio do sistema")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "calma-portability-{}-{}-{}.json",
            label,
            std::process::id(),
            nanos
        ))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn seed(conn: &Connection) {
        let mut root = Project::new("Trabalho".into());
        root.description = Some("Projetos profissionais".into());
        let root_id = db::insert_project(conn, &root).expect("projeto raiz");

        let mut child = Project::new("Calma".into());
        child.parent_id = Some(root_id);
        child.color = Some("#8fbcbb".into());
        let child_id = db::insert_project(conn, &child).expect("subprojeto");

        let mut parent_task = Task::new("Publicar versão".into());
        parent_task.project_id = Some(child_id);
        parent_task.due_date = Some(NaiveDate::from_ymd_opt(2026, 11, 20).unwrap());
        parent_task.priority = TaskPriority::Alta;
        parent_task.category = Some("Trabalho".into());
        let parent_task_id = db::insert_task(conn, &parent_task).expect("tarefa pai");

        let mut child_task = Task::new("Revisar README".into());
        child_task.parent_id = Some(parent_task_id);
        child_task.project_id = Some(child_id);
        child_task.status = TaskStatus::EmAndamento;
        db::insert_task(conn, &child_task).expect("subtarefa");

        let mut event = Event::new(
            "Reunião".into(),
            NaiveDate::from_ymd_opt(2026, 11, 15).unwrap(),
        );
        event.start_time = Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        event.description = Some("Revisão de roadmap".into());
        event.location = Some("Sala 1".into());
        db::insert_event(conn, &event).expect("evento");
    }

    fn write_backup_data(path: &Path, data: &BackupData) {
        let json = serde_json::to_string_pretty(data).expect("json de teste");
        fs::write(path, json).expect("gravar backup de teste");
    }

    fn empty_backup() -> BackupData {
        BackupData {
            format: BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION,
            exported_at: chrono::Local::now().naive_local(),
            projects: Vec::new(),
            tasks: Vec::new(),
            events: Vec::new(),
            task_completions: Vec::new(),
        }
    }

    #[test]
    fn export_and_import_overwrite_preserves_hierarchy() {
        let source = db::init_memory().expect("banco de origem");
        seed(&source);
        let path = temp_path("roundtrip");

        let summary = export_to_file(&path, &source).expect("exportar");
        assert_eq!(summary.projects, 2);
        assert_eq!(summary.tasks, 2);
        assert_eq!(summary.events, 1);

        let raw = fs::read_to_string(&path).expect("ler backup");
        let data: BackupData = serde_json::from_str(&raw).expect("json válido");
        assert_eq!(data.format, BACKUP_FORMAT);
        assert_eq!(data.version, BACKUP_VERSION);

        let target = db::init_memory().expect("banco de destino");
        let imported = import_from_file(&path, &target, ImportMode::Overwrite).expect("importar");
        assert_eq!(imported.projects, 2);
        assert_eq!(imported.tasks, 2);
        assert_eq!(imported.events, 1);
        assert_eq!(imported.duplicates_skipped, 0);

        let projects = db::list_projects(&target).expect("listar projetos");
        let root = projects
            .iter()
            .find(|project| project.name == "Trabalho")
            .expect("projeto raiz importado");
        let child = projects
            .iter()
            .find(|project| project.name == "Calma")
            .expect("subprojeto importado");
        assert_eq!(child.parent_id, Some(root.id));
        assert_eq!(child.color.as_deref(), Some("#8fbcbb"));

        let tasks = db::list_tasks(&target).expect("listar tarefas");
        let parent = tasks
            .iter()
            .find(|task| task.title == "Publicar versão")
            .expect("tarefa pai importada");
        let sub = tasks
            .iter()
            .find(|task| task.title == "Revisar README")
            .expect("subtarefa importada");
        assert_eq!(parent.project_id, Some(child.id));
        assert_eq!(parent.priority, TaskPriority::Alta);
        assert_eq!(sub.parent_id, Some(parent.id));
        assert_eq!(sub.project_id, Some(child.id));
        assert_eq!(sub.status, TaskStatus::EmAndamento);

        let events = db::list_events(&target).expect("listar eventos");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Reunião");
        assert_eq!(events[0].location.as_deref(), Some("Sala 1"));

        cleanup(&path);
    }

    #[test]
    fn overwrite_removes_existing_data() {
        let source = db::init_memory().expect("banco de origem");
        seed(&source);
        let path = temp_path("overwrite-remove");
        export_to_file(&path, &source).expect("exportar");

        let target = db::init_memory().expect("banco de destino");
        db::insert_project(&target, &Project::new("Antigo".into())).expect("projeto antigo");
        db::insert_task(&target, &Task::new("Tarefa antiga".into())).expect("tarefa antiga");
        db::insert_event(
            &target,
            &Event::new(
                "Evento antigo".into(),
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ),
        )
        .expect("evento antigo");

        import_from_file(&path, &target, ImportMode::Overwrite).expect("importar");

        let projects = db::list_projects(&target).expect("listar projetos");
        assert_eq!(projects.len(), 2);
        assert!(projects.iter().all(|project| project.name != "Antigo"));
        let tasks = db::list_tasks(&target).expect("listar tarefas");
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|task| task.title != "Tarefa antiga"));
        let events = db::list_events(&target).expect("listar eventos");
        assert_eq!(events.len(), 1);
        assert_ne!(events[0].title, "Evento antigo");

        cleanup(&path);
    }

    #[test]
    fn add_import_skips_every_duplicate() {
        let conn = db::init_memory().expect("banco");
        seed(&conn);
        let path = temp_path("add-dedup");
        export_to_file(&path, &conn).expect("exportar");

        let summary = import_from_file(&path, &conn, ImportMode::Add).expect("importar");
        assert_eq!(summary.projects, 0);
        assert_eq!(summary.tasks, 0);
        assert_eq!(summary.events, 0);
        assert_eq!(summary.duplicates_skipped, 5);

        assert_eq!(db::list_projects(&conn).expect("projetos").len(), 2);
        assert_eq!(db::list_tasks(&conn).expect("tarefas").len(), 2);
        assert_eq!(db::list_events(&conn).expect("eventos").len(), 1);

        cleanup(&path);
    }

    #[test]
    fn add_import_inserts_only_new_items() {
        let source = db::init_memory().expect("banco de origem");
        seed(&source);
        let path = temp_path("add-partial");
        export_to_file(&path, &source).expect("exportar");

        let target = db::init_memory().expect("banco de destino");
        let mut duplicate_event = Event::new(
            "Reunião".into(),
            NaiveDate::from_ymd_opt(2026, 11, 15).unwrap(),
        );
        duplicate_event.start_time = Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        duplicate_event.description = Some("Revisão de roadmap".into());
        duplicate_event.location = Some("Sala 1".into());
        db::insert_event(&target, &duplicate_event).expect("evento duplicado");

        let summary = import_from_file(&path, &target, ImportMode::Add).expect("importar");
        assert_eq!(summary.projects, 2);
        assert_eq!(summary.tasks, 2);
        assert_eq!(summary.events, 0);
        assert_eq!(summary.duplicates_skipped, 1);
        assert_eq!(db::list_events(&target).expect("eventos").len(), 1);

        cleanup(&path);
    }

    #[test]
    fn invalid_json_is_rejected() {
        let path = temp_path("invalid-json");
        fs::write(&path, "{ isso não é json").expect("gravar arquivo inválido");
        let conn = db::init_memory().expect("banco");
        let result = import_from_file(&path, &conn, ImportMode::Add);
        assert!(matches!(result, Err(AppError::Backup(_))));
        cleanup(&path);
    }

    #[test]
    fn invalid_format_and_version_are_rejected() {
        let path = temp_path("invalid-format");
        let conn = db::init_memory().expect("banco");

        let mut data = empty_backup();
        data.format = "outro-formato".into();
        write_backup_data(&path, &data);
        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));

        let mut data = empty_backup();
        data.version = BACKUP_VERSION + 1;
        write_backup_data(&path, &data);
        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));

        let mut data = empty_backup();
        data.version = 0;
        write_backup_data(&path, &data);
        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));

        cleanup(&path);
    }

    #[test]
    fn invalid_references_abort_without_changing_data() {
        let path = temp_path("invalid-reference");
        let conn = db::init_memory().expect("banco");
        db::insert_task(&conn, &Task::new("Existente".into())).expect("tarefa existente");

        let mut task = Task::new("Órfã".into());
        task.id = 1;
        task.project_id = Some(99);
        let mut data = empty_backup();
        data.tasks.push(task);
        write_backup_data(&path, &data);

        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Overwrite),
            Err(AppError::Backup(_))
        ));
        let tasks = db::list_tasks(&conn).expect("listar tarefas");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Existente");

        cleanup(&path);
    }

    #[test]
    fn project_and_task_cycles_are_rejected() {
        let path = temp_path("cycles");
        let conn = db::init_memory().expect("banco");

        let mut first = Project::new("A".into());
        first.id = 1;
        first.parent_id = Some(2);
        let mut second = Project::new("B".into());
        second.id = 2;
        second.parent_id = Some(1);
        let mut data = empty_backup();
        data.projects = vec![first, second];
        write_backup_data(&path, &data);
        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));

        let mut task_a = Task::new("A".into());
        task_a.id = 10;
        task_a.parent_id = Some(11);
        let mut task_b = Task::new("B".into());
        task_b.id = 11;
        task_b.parent_id = Some(10);
        let mut data = empty_backup();
        data.tasks = vec![task_a, task_b];
        write_backup_data(&path, &data);
        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));

        cleanup(&path);
    }

    #[test]
    fn export_and_import_preserves_recurrence_and_completions() {
        let source = db::init_memory().expect("banco de origem");
        let mut recurring = Task::new("Academia".into());
        recurring.recurrence = TaskRecurrence::Weekly;
        recurring.recurrence_days = 0b001_0101;
        recurring.scheduled_time = Some(NaiveTime::from_hms_opt(7, 30, 0).unwrap());
        let task_id = db::insert_task(&source, &recurring).expect("tarefa recorrente");

        let day = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        db::set_task_completed_on(&source, task_id, day, true).expect("conclusão");

        let path = temp_path("recurrence");
        export_to_file(&path, &source).expect("exportar");

        let raw = fs::read_to_string(&path).expect("ler backup");
        let data: BackupData = serde_json::from_str(&raw).expect("json válido");
        assert_eq!(data.task_completions.len(), 1);
        assert_eq!(data.task_completions[0].date, day);

        let target = db::init_memory().expect("banco de destino");
        import_from_file(&path, &target, ImportMode::Overwrite).expect("importar");

        let tasks = db::list_tasks(&target).expect("listar tarefas");
        let imported = tasks
            .iter()
            .find(|task| task.title == "Academia")
            .expect("tarefa recorrente importada");
        assert_eq!(imported.recurrence, TaskRecurrence::Weekly);
        assert_eq!(imported.recurrence_days, 0b001_0101);
        assert_eq!(
            imported.scheduled_time,
            Some(NaiveTime::from_hms_opt(7, 30, 0).unwrap())
        );

        let completions = db::list_task_completions(&target).expect("listar conclusões");
        assert_eq!(completions, vec![(imported.id, day)]);

        cleanup(&path);
    }

    #[test]
    fn add_import_skips_duplicate_recurring_tasks_and_keeps_completion() {
        let conn = db::init_memory().expect("banco");
        let mut recurring = Task::new("Corrida".into());
        recurring.recurrence = TaskRecurrence::Daily;
        recurring.scheduled_time = Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap());
        let task_id = db::insert_task(&conn, &recurring).expect("tarefa recorrente");
        let day = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        db::set_task_completed_on(&conn, task_id, day, true).expect("conclusão");

        let path = temp_path("add-recurrence");
        export_to_file(&path, &conn).expect("exportar");

        let summary = import_from_file(&path, &conn, ImportMode::Add).expect("importar");
        assert_eq!(summary.tasks, 0);
        assert_eq!(summary.duplicates_skipped, 1);
        assert_eq!(db::list_tasks(&conn).expect("tarefas").len(), 1);
        assert_eq!(
            db::list_task_completions(&conn).expect("conclusões"),
            vec![(task_id, day)]
        );

        cleanup(&path);
    }

    #[test]
    fn completion_referencing_missing_task_is_rejected() {
        let path = temp_path("missing-completion-task");
        let conn = db::init_memory().expect("banco");
        let mut data = empty_backup();
        data.task_completions.push(TaskCompletion {
            task_id: 42,
            date: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
        });
        write_backup_data(&path, &data);

        assert!(matches!(
            import_from_file(&path, &conn, ImportMode::Add),
            Err(AppError::Backup(_))
        ));
        cleanup(&path);
    }

    #[test]
    fn old_backup_without_completions_field_is_supported() {
        let path = temp_path("old-format");
        let conn = db::init_memory().expect("banco");
        let data = empty_backup();
        let mut value = serde_json::to_value(&data).expect("json de teste");
        value
            .as_object_mut()
            .expect("objeto json")
            .remove("task_completions");
        fs::write(&path, serde_json::to_string(&value).expect("serializar")).expect("gravar");

        let summary = import_from_file(&path, &conn, ImportMode::Add).expect("importar");
        assert_eq!(summary, ImportSummary::default());
        cleanup(&path);
    }

    #[test]
    fn resolve_user_path_expands_home_and_rejects_empty() {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(
            resolve_user_path("~/backup.json").expect("caminho válido"),
            home.join("backup.json")
        );
        assert!(matches!(resolve_user_path("   "), Err(AppError::Backup(_))));
    }
}
