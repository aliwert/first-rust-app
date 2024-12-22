use crate::model::todo::Todo;
use crate::model::todo::TodoState;
use crate::repository::ddb::DDBRepository;
use actix_web::{
    get, 
    post, 
    put,
    error::ResponseError,
    web::Path,
    web::Json,
    web::Data,
    HttpResponse,
    http::{header::ContentType, StatusCode}
};
use serde::{Serialize, Deserialize};
use derive_more::{Display};

#[derive(Deserialize, Serialize)]
pub struct TodoIdentifier {
    todo_global_id: String,
}

#[derive(Deserialize)]
pub struct TodoCompletionRequest {
    result_file: String
}

#[derive(Deserialize)]
pub struct SubmitTodoRequest {
    user_id: String,
    todo_type: String,
    source_file: String
}

#[derive(Debug, Display)]
pub enum TodoError {
    TodoNotFound,
    TodoUpdateFailure,
    TodoCreationFailure,
    BadTodoRequest
}

impl ResponseError for TodoError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TodoError::TodoNotFound => StatusCode::NOT_FOUND,
            TodoError::TodoUpdateFailure => StatusCode::FAILED_DEPENDENCY,
            TodoError::TodoCreationFailure => StatusCode::FAILED_DEPENDENCY,
            TodoError::BadTodoRequest => StatusCode::BAD_REQUEST
        }
    }
}

#[get("/todo/{todo_global_id}")]
pub async fn get_todo(
    ddb_repo: Data<DDBRepository>, 
    todo_identifier: Path<TodoIdentifier>
) -> Result<Json<Todo>, TodoError> {
    let tsk = ddb_repo.get_todo(
        todo_identifier.into_inner().todo_global_id
    ).await;

    match tsk {
        Some(tsk) => Ok(Json(tsk)),
        None => Err(TodoError::TodoNotFound)
    }
}

#[post("/todo")]
pub async fn submit_todo(
    ddb_repo: Data<DDBRepository>,
    request: Json<SubmitTodoRequest>
) -> Result<Json<TodoIdentifier>, TodoError> {
    let todo = Todo::new (
        request.user_id.clone(),
        request.todo_type.clone(),
        request.source_file.clone(),
    );

    let todo_identifier = todo.get_global_id();
    match ddb_repo.put_todo(todo).await {
        Ok(()) => Ok(Json(TodoIdentifier { todo_global_id: todo_identifier })),
        Err(_) => Err(TodoError::TodoCreationFailure)
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>, 
    todo_global_id: String,
    new_state: TodoState,
    result_file: Option<String>
) -> Result<Json<TodoIdentifier>, TodoError> {
    let mut todo = match ddb_repo.get_todo(
        todo_global_id
    ).await {
        Some(todo) => todo,
        None => return Err(TodoError::TodoNotFound)
    };

    if !todo.can_transition_to(&new_state) {
        return Err(TodoError::BadTodoRequest);
    };
    
    todo.state = new_state;
    todo.result_file = result_file;

    let todo_identifier = todo.get_global_id();
    match ddb_repo.put_todo(todo).await {
        Ok(()) => Ok(Json(TodoIdentifier { todo_global_id: todo_identifier })),
        Err(_) => Err(TodoError::TodoUpdateFailure)
    }
}

#[put("/todo/{todo_global_id}/start")]
pub async fn start_todo(
    ddb_repo: Data<DDBRepository>, 
    todo_identifier: Path<TodoIdentifier>
) -> Result<Json<TodoIdentifier>, TodoError> {
    state_transition(
        ddb_repo, 
        todo_identifier.into_inner().todo_global_id, 
        TodoState::InProgress, 
        None
    ).await
}

#[put("/todo/{todo_global_id}/pause")]
pub async fn pause_todo(
    ddb_repo: Data<DDBRepository>, 
    todo_identifier: Path<TodoIdentifier>
) -> Result<Json<TodoIdentifier>, TodoError> {
    state_transition(
        ddb_repo, 
        todo_identifier.into_inner().todo_global_id, 
        TodoState::Paused, 
        None
    ).await
}

#[put("/todo/{todo_global_id}/fail")]
pub async fn fail_todo(
    ddb_repo: Data<DDBRepository>, 
    todo_identifier: Path<TodoIdentifier>
) -> Result<Json<TodoIdentifier>, TodoError> {
    state_transition(
        ddb_repo, 
        todo_identifier.into_inner().todo_global_id, 
        TodoState::Failed, 
        None
    ).await
}

#[put("/todo/{todo_global_id}/complete")]
pub async fn complete_todo(
    ddb_repo: Data<DDBRepository>, 
    todo_identifier: Path<TodoIdentifier>,
    completion_request: Json<TodoCompletionRequest>
) -> Result<Json<TodoIdentifier>, TodoError> {
    state_transition(
        ddb_repo, 
        todo_identifier.into_inner().todo_global_id, 
        TodoState::Completed, 
        Some(completion_request.result_file.clone())
    ).await
}