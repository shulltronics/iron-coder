#[cfg(test)]
mod project_tests {
    use crate::project::*;

    #[test]
    pub fn test_save_as() {
        let mut project: Project = Project{name : "test_project".to_string(), location: None, system: Default::default(), code_editor: Default::default(), terminal_buffer: "".to_string(), receiver: None, current_view: Default::default(), known_boards: vec![], repo: Repository };
        project.save_as(true).expect("Project Failed to Save!");
    }
}