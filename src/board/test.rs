use std::process::Command;

fn cli_cmd(str: &str) {
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["/C", &str])
            .output()
            .expect("failed to execute process")
    } else {
        return;
    };
    let str = String::from_utf8(output.stdout).expect("Returned output");
    print!("{}", str);
}

#[cfg(test)]
mod board_tests {
    use std::collections::HashSet;
    use std::path::Path;
    use eframe::glow::FALSE;
    use egui::TextBuffer;
    use crate::board;
    use crate::board::get_boards;
    use crate::board::test::cli_cmd;

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
    #[test]
    pub fn test_board_info() {
        // Ensure boards have crates associated with them.
        let mut boards = board::get_boards(Path::new("./iron-coder-boards"));
        for board in boards {
            assert!(!board.related_crates().unwrap().is_empty());
        }
    }
    #[test]
    pub fn test_board_crates() {
        // Ensure crates don't have any errors.
        let mut boards = board::get_boards(Path::new("./iron-coder-boards"));
        let mut cmd = "cargo check --features".to_owned();
        for board in boards {
            for crates in board.related_crates().unwrap() {
                cmd.push_str(crates.as_str());
                cli_cmd(cmd.as_mut_str());
            }
        }
    }
}
