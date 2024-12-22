use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_config::Config;
use crate::model::todo::{Todo, TodoState};
use log::error;
use std::str::FromStr;
use std::collections::HashMap;

pub struct DDBRepository {
    client: Client,
    table_name: String
}

pub struct DDBError;

fn required_item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<String, DDBError> {
    match item_value(key, item) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(DDBError),
        Err(DDBError) => Err(DDBError)
    }
}

fn item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<Option<String>, DDBError> {
    match item.get(key) {
        Some(value) => match value.as_s() {
            Ok(val) => Ok(Some(val.clone())),
            Err(_) => Err(DDBError)
        },
        None => Ok(None)
    }
}

fn item_to_todo(item: &HashMap<String, AttributeValue>) -> Result<Todo, DDBError> {
    let state: TodoState = match TodoState::from_str(required_item_value("state", item)?.as_str()) {
        Ok(value) => value,
        Err(_) => return Err(DDBError)
    };

    let result_file = item_value("result_file", item)?;

    Ok(Todo {
        user_uuid: required_item_value("pK", item)?,
        todo_uuid: required_item_value("sK", item)?,
        todo_type: required_item_value("todo_type", item)?,
        state,
        source_file: required_item_value("source_file", item)?,
        result_file
    })
}

impl DDBRepository {
    pub fn init(table_name: String, config: Config) -> DDBRepository {
        let client = Client::new(&config);
        DDBRepository {
            table_name,
            client
        }
    }

    pub async fn put_todo(&self, todo: Todo) -> Result<(), DDBError> {
        let mut request = self.client.put_item()
            .table_name(&self.table_name)
            .item("pK", AttributeValue::S(String::from(todo.user_uuid)))
            .item("sK", AttributeValue::S(String::from(todo.todo_uuid)))
            .item("todo_type", AttributeValue::S(String::from(todo.todo_type)))
            .item("state", AttributeValue::S(todo.state.to_string()))
            .item("source_file", AttributeValue::S(String::from(todo.source_file)));
        
        if let Some(result_file) = todo.result_file {
            request = request.item("result_file", AttributeValue::S(String::from(result_file)));
        }

        match request.send().await {
            Ok(_) => Ok(()),
            Err(_) => Err(DDBError)
        }
    }

    pub async fn get_todo(&self, todo_id: String) -> Option<Todo> {
        let tokens:Vec<String> = todo_id
            .split("_")
            .map(|x| String::from(x))
            .collect();
        let user_uuid = AttributeValue::S(tokens[0].clone());
        let todo_uuid = AttributeValue::S(tokens[1].clone());
        
        let res = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pK = :user_id and #sK = :todo_uuid")
            .expression_attribute_names("#pK", "pK")
            .expression_attribute_names("#sK", "sK")
            .expression_attribute_values(":user_id", user_uuid)
            .expression_attribute_values(":todo_uuid", todo_uuid)
            .send()
            .await;

        return match res {
            Ok(output) => {
                match output.items {
                    Some(items) => {
                        let item = &items.first()?;
                        error!("{:?}", &item);
                        match item_to_todo(item) {
                            Ok(todo) => Some(todo),
                            Err(_) => None
                        }
                    },
                    None => {
                        None
                    }
                }
            },
            Err(error) => {
                error!("{:?}", error);
                None
            }
        }
    }
}