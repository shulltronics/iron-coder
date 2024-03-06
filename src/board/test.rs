#[cfg(test)]
mod board_tests {
    use std::collections::HashSet;
    use std::path::Path;
    use crate::board;

    #[test]
    pub fn test_get_boards() {
        let mut board_names: HashSet<&str> = HashSet::from(["Feather nRF52832", "Feather RP2040", "OLED Featherwing (128x64)", "PropMaker Featherwing", "PiTFT 3.2 with Capacitive Touch Screen", "MicroMod ESP32 Processor"]);
        let boards = board::get_boards(Path::new("./iron-coder-boards"));
        for board in boards {
            assert!(board_names.contains(board.get_name()));
            board_names.remove(board.get_name());
        }
        assert!(board_names.is_empty());
    }
}