/// TODO - Write more tests mainly testing egui related functionality (more research needed)
#[cfg(test)]
mod app_tests {
    use crate::IronCoderApp;

    #[test]
    fn test_initialization() {
        let app : IronCoderApp = IronCoderApp::default();
        assert_ne!(app.boards.len(), 0);
    }
    #[test]
    fn test_get_boards() {
        let app : IronCoderApp = IronCoderApp::default();
        let boards = app.get_boards();
        assert_ne!(boards.len(), 0);
        assert_eq!(boards, app.boards);
    }
}