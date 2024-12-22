use serde::Serialize;
use uuid::Uuid;
use strum_macros::{EnumString, Display};

#[derive(Serialize, EnumString, Display, Eq, PartialEq)]
pub enum TodoState {
    NotStarted,
    InProgress,
    Completed,
    Paused,
    Failed
}

#[derive(Serialize)]
pub struct Todo {
    pub user_uuid: String,
    pub todo_uuid: String,
    pub todo_type: String,
    pub state: TodoState,
    pub source_file: String,
    pub result_file: Option<String>
}

impl Todo {
    pub fn new(user_uuid: String, todo_type: String, source_file: String) -> Todo {
        Todo {
            user_uuid,
            todo_uuid: Uuid::new_v4().to_string(),
            todo_type,
            state: TodoState::NotStarted,
            source_file,
            result_file: None
        }
    }

    pub fn get_global_id(&self) -> String {
        return format!("{}_{}", self.user_uuid, self.todo_uuid);
    }

    pub fn can_transition_to(&self, state: &TodoState) -> bool {
        self.state != *state
    }
}