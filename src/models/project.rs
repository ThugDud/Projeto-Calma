use serde::{Deserialize, Serialize};

/// Representa um projeto ou grupo de tarefas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub description: Option<String>,
    pub sort_order: i32,
    pub color: Option<String>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Project {
            id: 0,
            name,
            parent_id: None,
            description: None,
            sort_order: 0,
            color: None,
        }
    }
}
