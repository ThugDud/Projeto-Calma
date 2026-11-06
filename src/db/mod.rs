use crate::errors::{AppError, AppResult};
use crate::models::event::Event;
use crate::models::project::Project;
use crate::models::task::{Task, TaskPriority, TaskStatus};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

pub const SCHEMA_VERSION: i32 = 3;

/// Inicializa o banco de dados SQLite no caminho especificado e aplica migrações.
pub fn init(db_path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::Database(format!("Não foi possível abrir o banco SQLite: {}", e)))?;

    setup_schema(&conn)?;
    Ok(conn)
}

/// Inicializa um banco de dados SQLite em memória para testes.
pub fn init_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory().map_err(|e| {
        AppError::Database(format!("Não foi possível abrir banco em memória: {}", e))
    })?;

    setup_schema(&conn)?;
    Ok(conn)
}

/// Aplica as migrações de esquema pendentes, de forma incremental e idempotente.
pub fn setup_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| AppError::Database(format!("Erro ao ativar chaves estrangeiras: {}", e)))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
             version INTEGER PRIMARY KEY,
             applied_at TEXT NOT NULL DEFAULT (datetime('now'))
         );",
    )
    .map_err(|e| AppError::Database(format!("Erro ao criar tabela de migrações: {}", e)))?;

    let current = current_schema_version(conn)?;

    if current < 1 {
        migrate_v1(conn)?;
    }
    if current < 2 {
        migrate_v2(conn)?;
    }
    if current < 3 {
        migrate_v3(conn)?;
    }

    Ok(())
}

fn current_schema_version(conn: &Connection) -> AppResult<i32> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
    .map_err(|e| AppError::Database(format!("Erro ao verificar versão do esquema: {}", e)))
}

fn record_migration(conn: &Connection, version: i32) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
        params![version],
    )
    .map_err(|e| AppError::Database(format!("Erro ao registrar migração {}: {}", version, e)))?;
    Ok(())
}

/// v1: tabelas base.
fn migrate_v1(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT NOT NULL,
             parent_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
             created_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS tasks (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             title TEXT NOT NULL,
             parent_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
             project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
             created_at TEXT NOT NULL,
             due_date TEXT,
             status INTEGER NOT NULL DEFAULT 0,
             priority INTEGER NOT NULL DEFAULT 1
         );
         CREATE TABLE IF NOT EXISTS events (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             title TEXT NOT NULL,
             date TEXT NOT NULL,
             created_at TEXT NOT NULL
         );",
    )
    .map_err(|e| AppError::Database(format!("Erro na migração v1: {}", e)))?;

    record_migration(conn, 1)
}

/// v2: colunas de descrição, agendamento, categorização e ordenação.
fn migrate_v2(conn: &Connection) -> AppResult<()> {
    add_column_if_missing(conn, "projects", "description", "TEXT")?;
    add_column_if_missing(conn, "projects", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
    add_column_if_missing(conn, "projects", "color", "TEXT")?;

    add_column_if_missing(conn, "tasks", "description", "TEXT")?;
    add_column_if_missing(conn, "tasks", "scheduled_date", "TEXT")?;
    add_column_if_missing(conn, "tasks", "scheduled_time", "TEXT")?;
    add_column_if_missing(conn, "tasks", "category", "TEXT")?;
    add_column_if_missing(conn, "tasks", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
    add_column_if_missing(conn, "tasks", "completed_at", "TEXT")?;

    add_column_if_missing(conn, "events", "description", "TEXT")?;
    add_column_if_missing(conn, "events", "start_time", "TEXT")?;
    add_column_if_missing(conn, "events", "end_time", "TEXT")?;
    add_column_if_missing(conn, "events", "category", "TEXT")?;
    add_column_if_missing(conn, "events", "location", "TEXT")?;

    record_migration(conn, 2)
}

/// v3: índices para consultas por data, projeto e hierarquia.
fn migrate_v3(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
         CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
         CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks(parent_id);
         CREATE INDEX IF NOT EXISTS idx_events_date ON events(date);
         CREATE INDEX IF NOT EXISTS idx_projects_parent_id ON projects(parent_id);",
    )
    .map_err(|e| AppError::Database(format!("Erro na migração v3: {}", e)))?;

    record_migration(conn, 3)
}

/// Adiciona uma coluna apenas se ela ainda não existir (migração segura).
fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> AppResult<()> {
    let exists = {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| {
                AppError::Database(format!("Erro ao inspecionar tabela {}: {}", table, e))
            })?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(format!("Erro ao ler colunas de {}: {}", table, e)))?;

        let mut found = false;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == column {
                found = true;
                break;
            }
        }
        found
    };

    if !exists {
        conn.execute_batch(&format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            table, column, definition
        ))
        .map_err(|e| {
            AppError::Database(format!(
                "Erro ao adicionar coluna {}.{}: {}",
                table, column, e
            ))
        })?;
    }

    Ok(())
}

// ==========================================
// TAREFAS (CRUD & OPERAÇÕES)
// ==========================================

pub fn insert_task(conn: &Connection, task: &Task) -> AppResult<i64> {
    if let Some(parent_id) = task.parent_id {
        if task.id != 0 && (task.id == parent_id || is_ancestor_task(conn, task.id, parent_id)?) {
            return Err(AppError::Business(
                "Ciclo na hierarquia de tarefas não permitido".into(),
            ));
        }
    }

    conn.execute(
        "INSERT INTO tasks (title, description, parent_id, project_id, created_at,
                            due_date, scheduled_date, scheduled_time, status, priority,
                            category, sort_order, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            task.title,
            task.description,
            task.parent_id,
            task.project_id,
            task.created_at.to_string(),
            task.due_date.map(|d| d.to_string()),
            task.scheduled_date.map(|d| d.to_string()),
            task.scheduled_time.map(|t| t.to_string()),
            task.status as i32,
            task.priority as i32,
            task.category,
            task.sort_order,
            task.completed_at.map(|d| d.to_string()),
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao inserir tarefa: {}", e)))?;

    Ok(conn.last_insert_rowid())
}

pub fn list_tasks(conn: &Connection) -> AppResult<Vec<Task>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, parent_id, project_id, created_at,
                    due_date, scheduled_date, scheduled_time, status, priority,
                    category, sort_order, completed_at
             FROM tasks
             ORDER BY sort_order ASC, id ASC",
        )
        .map_err(|e| AppError::Database(format!("Erro ao preparar consulta: {}", e)))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| AppError::Database(format!("Erro ao executar consulta: {}", e)))?;

    let mut tasks = Vec::new();
    while let Some(row) = rows.next()? {
        let created_at_str: String = row.get(5)?;
        let created_at = parse_datetime(&created_at_str)?;

        let due_date_str: Option<String> = row.get(6)?;
        let due_date = due_date_str
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        let scheduled_date_str: Option<String> = row.get(7)?;
        let scheduled_date = scheduled_date_str
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        let scheduled_time_str: Option<String> = row.get(8)?;
        let scheduled_time = scheduled_time_str
            .as_deref()
            .and_then(|s| s.parse::<NaiveTime>().ok());

        let status_val: i32 = row.get(9)?;
        let status = match status_val {
            1 => TaskStatus::EmAndamento,
            2 => TaskStatus::Concluida,
            _ => TaskStatus::Pendente,
        };

        let priority_val: i32 = row.get(10)?;
        let priority = match priority_val {
            0 => TaskPriority::Baixa,
            2 => TaskPriority::Alta,
            3 => TaskPriority::Urgente,
            _ => TaskPriority::Normal,
        };

        let completed_at_str: Option<String> = row.get(13)?;
        let completed_at = completed_at_str
            .as_deref()
            .and_then(|s| parse_datetime(s).ok());

        tasks.push(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            parent_id: row.get(3)?,
            project_id: row.get(4)?,
            created_at,
            due_date,
            scheduled_date,
            scheduled_time,
            status,
            priority,
            category: row.get(11)?,
            sort_order: row.get(12)?,
            completed_at,
        });
    }

    Ok(tasks)
}

pub fn update_task(conn: &Connection, task: &Task) -> AppResult<()> {
    if let Some(parent_id) = task.parent_id {
        if task.id == parent_id || is_ancestor_task(conn, task.id, parent_id)? {
            return Err(AppError::Business(
                "Uma tarefa não pode ser ancestral ou filha de si mesma".into(),
            ));
        }
    }

    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, parent_id = ?3,
                          project_id = ?4, due_date = ?5, scheduled_date = ?6,
                          scheduled_time = ?7, status = ?8, priority = ?9,
                          category = ?10, sort_order = ?11, completed_at = ?12
         WHERE id = ?13",
        params![
            task.title,
            task.description,
            task.parent_id,
            task.project_id,
            task.due_date.map(|d| d.to_string()),
            task.scheduled_date.map(|d| d.to_string()),
            task.scheduled_time.map(|t| t.to_string()),
            task.status as i32,
            task.priority as i32,
            task.category,
            task.sort_order,
            task.completed_at.map(|d| d.to_string()),
            task.id,
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao atualizar tarefa: {}", e)))?;
    Ok(())
}

pub fn delete_task(conn: &Connection, task_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
        .map_err(|e| AppError::Database(format!("Erro ao excluir tarefa: {}", e)))?;
    Ok(())
}

pub fn mark_task_completed(conn: &Connection, task_id: i64, completed: bool) -> AppResult<()> {
    let now = chrono::Local::now().naive_local();
    if completed {
        conn.execute(
            "UPDATE tasks SET status = ?1, completed_at = ?2 WHERE id = ?3",
            params![TaskStatus::Concluida as i32, now.to_string(), task_id],
        )
        .map_err(|e| AppError::Database(format!("Erro ao concluir tarefa: {}", e)))?;
    } else {
        conn.execute(
            "UPDATE tasks SET status = ?1, completed_at = NULL WHERE id = ?2",
            params![TaskStatus::Pendente as i32, task_id],
        )
        .map_err(|e| AppError::Database(format!("Erro ao reabrir tarefa: {}", e)))?;
    }
    Ok(())
}

/// Verifica se `potential_ancestor_id` é pai, avô, etc., de `candidate_parent_id`.
pub fn is_ancestor_task(
    conn: &Connection,
    potential_ancestor_id: i64,
    mut current_parent_id: i64,
) -> AppResult<bool> {
    let mut visited = std::collections::HashSet::new();
    visited.insert(current_parent_id);

    loop {
        if current_parent_id == potential_ancestor_id {
            return Ok(true);
        }
        let parent: Option<i64> = conn
            .query_row(
                "SELECT parent_id FROM tasks WHERE id = ?1",
                params![current_parent_id],
                |row| row.get(0),
            )
            .optional()?;

        match parent {
            Some(pid) => {
                if !visited.insert(pid) {
                    // Já existe ciclo
                    return Ok(true);
                }
                current_parent_id = pid;
            }
            None => return Ok(false),
        }
    }
}

// ==========================================
// PROJETOS (CRUD)
// ==========================================

pub fn insert_project(conn: &Connection, project: &Project) -> AppResult<i64> {
    if let Some(parent_id) = project.parent_id {
        if project.id != 0
            && (project.id == parent_id || is_ancestor_project(conn, project.id, parent_id)?)
        {
            return Err(AppError::Business(
                "Ciclo na hierarquia de projetos não permitido".into(),
            ));
        }
    }

    let now = chrono::Local::now().naive_local();
    conn.execute(
        "INSERT INTO projects (name, parent_id, description, sort_order, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            project.name,
            project.parent_id,
            project.description,
            project.sort_order,
            project.color,
            now.to_string(),
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao inserir projeto: {}", e)))?;

    Ok(conn.last_insert_rowid())
}

pub fn list_projects(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, description, sort_order, color FROM projects ORDER BY sort_order ASC, id ASC")
        .map_err(|e| AppError::Database(format!("Erro ao preparar consulta de projetos: {}", e)))?;

    let mut rows = stmt.query([])?;
    let mut projects = Vec::new();
    while let Some(row) = rows.next()? {
        projects.push(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            description: row.get(3)?,
            sort_order: row.get(4)?,
            color: row.get(5)?,
        });
    }
    Ok(projects)
}

pub fn update_project(conn: &Connection, project: &Project) -> AppResult<()> {
    if let Some(parent_id) = project.parent_id {
        if project.id == parent_id || is_ancestor_project(conn, project.id, parent_id)? {
            return Err(AppError::Business(
                "Um projeto não pode ser ancestral ou filho de si mesmo".into(),
            ));
        }
    }

    conn.execute(
        "UPDATE projects SET name = ?1, parent_id = ?2, description = ?3, sort_order = ?4, color = ?5
         WHERE id = ?6",
        params![
            project.name,
            project.parent_id,
            project.description,
            project.sort_order,
            project.color,
            project.id,
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao atualizar projeto: {}", e)))?;
    Ok(())
}

pub fn delete_project(conn: &Connection, project_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])
        .map_err(|e| AppError::Database(format!("Erro ao excluir projeto: {}", e)))?;
    Ok(())
}

/// Verifica se `potential_ancestor_id` é pai, avô, etc. de `candidate_parent_id`.
pub fn is_ancestor_project(
    conn: &Connection,
    potential_ancestor_id: i64,
    mut current_parent_id: i64,
) -> AppResult<bool> {
    let mut visited = std::collections::HashSet::new();
    visited.insert(current_parent_id);

    loop {
        if current_parent_id == potential_ancestor_id {
            return Ok(true);
        }
        let parent: Option<i64> = conn
            .query_row(
                "SELECT parent_id FROM projects WHERE id = ?1",
                params![current_parent_id],
                |row| row.get(0),
            )
            .optional()?;

        match parent {
            Some(pid) => {
                if !visited.insert(pid) {
                    return Ok(true);
                }
                current_parent_id = pid;
            }
            None => return Ok(false),
        }
    }
}

// ==========================================
// EVENTOS (CRUD)
// ==========================================

pub fn insert_event(conn: &Connection, event: &Event) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO events (title, description, date, start_time, end_time, category, location, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event.title,
            event.description,
            event.date.to_string(),
            event.start_time.map(|t| t.to_string()),
            event.end_time.map(|t| t.to_string()),
            event.category,
            event.location,
            event.created_at.to_string(),
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao inserir evento: {}", e)))?;

    Ok(conn.last_insert_rowid())
}

pub fn list_events(conn: &Connection) -> AppResult<Vec<Event>> {
    let mut stmt = conn
        .prepare("SELECT id, title, description, date, start_time, end_time, category, location, created_at FROM events ORDER BY date ASC, start_time IS NULL, start_time ASC")
        .map_err(|e| AppError::Database(format!("Erro ao preparar consulta de eventos: {}", e)))?;

    let mut rows = stmt.query([])?;
    let mut events = Vec::new();
    while let Some(row) = rows.next()? {
        let date_str: String = row.get(3)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| AppError::Database(format!("Data inválida em evento: {}", e)))?;

        let start_time: Option<String> = row.get(4)?;
        let start_time = start_time
            .as_deref()
            .and_then(|s| s.parse::<NaiveTime>().ok());

        let end_time: Option<String> = row.get(5)?;
        let end_time = end_time
            .as_deref()
            .and_then(|s| s.parse::<NaiveTime>().ok());

        let created_at_str: String = row.get(8)?;
        let created_at = parse_datetime(&created_at_str)?;

        events.push(Event {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            date,
            start_time,
            end_time,
            category: row.get(6)?,
            location: row.get(7)?,
            created_at,
        });
    }
    Ok(events)
}

pub fn update_event(conn: &Connection, event: &Event) -> AppResult<()> {
    conn.execute(
        "UPDATE events SET title = ?1, description = ?2, date = ?3, start_time = ?4,
                          end_time = ?5, category = ?6, location = ?7
         WHERE id = ?8",
        params![
            event.title,
            event.description,
            event.date.to_string(),
            event.start_time.map(|t| t.to_string()),
            event.end_time.map(|t| t.to_string()),
            event.category,
            event.location,
            event.id,
        ],
    )
    .map_err(|e| AppError::Database(format!("Erro ao atualizar evento: {}", e)))?;
    Ok(())
}

pub fn delete_event(conn: &Connection, event_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM events WHERE id = ?1", params![event_id])
        .map_err(|e| AppError::Database(format!("Erro ao excluir evento: {}", e)))?;
    Ok(())
}

fn parse_datetime(s: &str) -> AppResult<NaiveDateTime> {
    s.parse::<NaiveDateTime>()
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f"))
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .map_err(|e| AppError::Database(format!("Erro ao decodificar data/hora '{}': {}", s, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_init_and_task_crud() {
        let conn = init_memory().expect("Banco em memória deve inicializar");

        let mut task = Task::new("Estudar Álgebra Linear".into());
        task.priority = TaskPriority::Alta;
        task.category = Some("Faculdade".into());

        let id = insert_task(&conn, &task).expect("Inserção deve ter sucesso");
        assert!(id > 0);

        let tasks = list_tasks(&conn).expect("Listagem de tarefas");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Estudar Álgebra Linear");
        assert_eq!(tasks[0].status, TaskStatus::Pendente);

        // Concluir
        mark_task_completed(&conn, id, true).expect("Concluir tarefa");
        let tasks_after = list_tasks(&conn).unwrap();
        assert_eq!(tasks_after[0].status, TaskStatus::Concluida);
        assert!(tasks_after[0].completed_at.is_some());

        // Reabrir
        mark_task_completed(&conn, id, false).expect("Reabrir tarefa");
        let tasks_reopened = list_tasks(&conn).unwrap();
        assert_eq!(tasks_reopened[0].status, TaskStatus::Pendente);

        // Excluir
        delete_task(&conn, id).expect("Excluir tarefa");
        let tasks_empty = list_tasks(&conn).unwrap();
        assert_eq!(tasks_empty.len(), 0);
    }

    #[test]
    fn test_prevent_cycle_in_hierarchy() {
        let conn = init_memory().unwrap();

        let mut t1 = Task::new("Tarefa Pai".into());
        let id1 = insert_task(&conn, &t1).unwrap();
        t1.id = id1;

        let mut t2 = Task::new("Tarefa Filha".into());
        t2.parent_id = Some(id1);
        let id2 = insert_task(&conn, &t2).unwrap();
        t2.id = id2;

        // Tentar fazer t1 ter t2 como pai (ciclo direto)
        t1.parent_id = Some(id2);
        let result = update_task(&conn, &t1);
        assert!(result.is_err(), "Ciclo direto deve ser bloqueado");

        // Tentar fazer t1 ser pai de si mesma
        t1.parent_id = Some(id1);
        let self_cycle = update_task(&conn, &t1);
        assert!(self_cycle.is_err(), "Auto-referência deve ser bloqueada");
    }

    #[test]
    fn test_projects_crud() {
        let conn = init_memory().unwrap();

        let proj = Project::new("Faculdade".into());
        let pid = insert_project(&conn, &proj).expect("Inserir projeto");
        assert!(pid > 0);

        let projs = list_projects(&conn).expect("Listar projetos");
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].name, "Faculdade");

        // Atualizar
        let mut proj_updated = projs[0].clone();
        proj_updated.name = "Faculdade de Engenharia".into();
        update_project(&conn, &proj_updated).expect("Atualizar projeto");
        let projs_after = list_projects(&conn).unwrap();
        assert_eq!(projs_after[0].name, "Faculdade de Engenharia");

        delete_project(&conn, pid).expect("Excluir projeto");
        let projs_empty = list_projects(&conn).unwrap();
        assert_eq!(projs_empty.len(), 0);
    }

    #[test]
    fn test_prevent_cycle_in_projects() {
        let conn = init_memory().unwrap();

        let mut parent = Project::new("Faculdade".into());
        let parent_id = insert_project(&conn, &parent).unwrap();
        parent.id = parent_id;

        let mut child = Project::new("Cálculo".into());
        child.parent_id = Some(parent_id);
        let child_id = insert_project(&conn, &child).unwrap();

        // Fazer o pai virar filho do próprio filho deve falhar.
        parent.parent_id = Some(child_id);
        assert!(
            update_project(&conn, &parent).is_err(),
            "Ciclo indireto deve ser bloqueado"
        );

        // Um projeto não pode ser pai de si mesmo.
        parent.parent_id = Some(parent_id);
        assert!(
            update_project(&conn, &parent).is_err(),
            "Auto-referência deve ser bloqueada"
        );
    }

    #[test]
    fn test_event_crud() {
        let conn = init_memory().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();

        let mut event = Event::new("Reunião".into(), date);
        event.start_time = NaiveTime::from_hms_opt(14, 30, 0);
        event.location = Some("Sala 2".into());

        let id = insert_event(&conn, &event).expect("Inserir evento");
        assert!(id > 0);

        let events = list_events(&conn).expect("Listar eventos");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Reunião");
        assert_eq!(events[0].start_time, NaiveTime::from_hms_opt(14, 30, 0));

        let mut updated = events[0].clone();
        updated.title = "Reunião de equipe".into();
        updated.start_time = Some(NaiveTime::from_hms_opt(15, 0, 0).unwrap());
        update_event(&conn, &updated).expect("Atualizar evento");

        let events = list_events(&conn).unwrap();
        assert_eq!(events[0].title, "Reunião de equipe");
        assert_eq!(events[0].start_time, NaiveTime::from_hms_opt(15, 0, 0));

        delete_event(&conn, id).expect("Excluir evento");
        assert!(list_events(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_schema_migrations_are_idempotent() {
        let conn = init_memory().unwrap();

        // Rodar novamente não deve falhar nem duplicar versões.
        setup_schema(&conn).expect("Migrações idempotentes");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, SCHEMA_VERSION);
    }
}
