#[cfg(test)]
mod project_tests {
    use egui::Context;
    use crate::board::BoardStandards::Arduino;
    use crate::board::{BoardTomlInfo, PinoutTomlInfo};
    use crate::board::BoardType::Main;
    use crate::board::pinout::InterfaceDirection::Bidirectional;
    use crate::board::pinout::InterfaceType::{GPIO, UART};
    use crate::project::*;

    #[test]
    pub fn test_save_as() {
        let mut project: Project = Project{name : "test_project".to_string(), location: None, system: Default::default(), code_editor: Default::default(), terminal_buffer: "".to_string(), receiver: None, current_view: Default::default(), known_boards: vec![], repo: None };
        project.save_as(true).expect("Project Failed to Save!");
    }
    #[test]
    pub fn save_new_board_base_case() {
        let mut project: Project = Project{name : "test_toml_project".to_string(), location: None, system: Default::default(), code_editor: Default::default(), terminal_buffer: "".to_string(), receiver: None, current_view: Default::default(), known_boards: vec![], repo: None };
        let pin1 = PinoutTomlInfo {
            pins: vec![String::from("rx"), String::from("tx")],
            iface_type: UART,
            direction: Bidirectional,
        };
        let pin2 = PinoutTomlInfo {
            pins: vec![String::from("d0"), String::from("d1")],
            iface_type: GPIO,
            direction: Bidirectional,
        };
        let mut board_toml = BoardTomlInfo{
            name: String::from("ArduinoOne"),
            manufacturer: "UnitTesting".to_string(),
            board_type: Main,
            standard: Arduino.to_string(),
            cpu: "Cortex".to_string(),
            ram: 32,
            flash: 16,
            required_crates: vec![String::from("embedded-hal"), String::from("ravedude"), String::from("panic-halt")],
            related_crates: vec![String::from("smart-leds")],
            bsp: "".to_string(),
            pinouts: vec![pin1, pin2]
        };


        let mut ctx = egui::Context::default();
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        ctx.data_mut(|data| {
            data.insert_temp(board_toml_info_id, board_toml);
        });

        let svg_string = match fs::read_to_string( PathBuf::from("./iron-coder-boards/Arduino/Arduino_Uno/arduino_uno.svg")) {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        ctx.data_mut(|data| {
            data.insert_temp(new_board_svg_string_id, svg_string);
        });

        Project::save_new_board_info(&mut project, &mut ctx);

        let toml_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/ArduinoOne/arduinoone.toml") {
            Ok(string) => string,
            Err(e) => String::new(),
        };
        let svg_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/ArduinoOne/arduinoone.svg") {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        assert!(!toml_string.is_empty());
        assert!(!svg_string.is_empty());

        assert_eq!(toml_string, "name = \"ArduinoOne\"\n\
        manufacturer = \"UnitTesting\"\n\
        board_type = \"Main\"\n\
        standard = \"Arduino\"\n\
        cpu = \"Cortex\"\n\
        ram = 32\n\
        flash = 16\n\
        required_crates = [\"embedded-hal\", \"ravedude\", \"panic-halt\"]\n\
        related_crates = [\"smart-leds\"]\n\n\

        bsp = \"iron-coder-ArduinoOne-bsp\"\n\n\

        [[pinout]]\n\
        pins = [\"rx\", \"tx\"]\n\
        interface = { iface_type = \"UART\", direction = \"Bidirectional\" }\n\n\

        [[pinout]]\n\
        pins = [\"d0\", \"d1\"]\n\
        interface = { iface_type = \"GPIO\", direction = \"Bidirectional\" }\n\n");

    }
    #[test]
    pub fn save_new_board_contains_spaces() {
        let mut project: Project = Project{name : "test_toml_project".to_string(), location: None, system: Default::default(), code_editor: Default::default(), terminal_buffer: "".to_string(), receiver: None, current_view: Default::default(), known_boards: vec![], repo: None };
        let pin1 = PinoutTomlInfo {
            pins: vec![String::from("rx"), String::from("tx")],
            iface_type: UART,
            direction: Bidirectional,
        };
        let pin2 = PinoutTomlInfo {
            pins: vec![String::from("d0"), String::from("d1")],
            iface_type: GPIO,
            direction: Bidirectional,
        };
        let mut board_toml = BoardTomlInfo{
            name: String::from("Arduino Dos"),
            manufacturer: "UnitTesting".to_string(),
            board_type: Main,
            standard: Arduino.to_string(),
            cpu: "Cortex".to_string(),
            ram: 32,
            flash: 16,
            required_crates: vec![String::from("embedded-hal"), String::from("ravedude"), String::from("panic-halt")],
            related_crates: vec![String::from("smart-leds")],
            bsp: "".to_string(),
            pinouts: vec![pin1, pin2]
        };


        let mut ctx = egui::Context::default();
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        ctx.data_mut(|data| {
            data.insert_temp(board_toml_info_id, board_toml);
        });

        let svg_string = match fs::read_to_string( PathBuf::from("./iron-coder-boards/Arduino/Arduino_Uno/arduino_uno.svg")) {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        ctx.data_mut(|data| {
            data.insert_temp(new_board_svg_string_id, svg_string);
        });


        Project::save_new_board_info(&mut project, &mut ctx);

        let toml_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/Arduino_Dos/arduino_dos.toml") {
            Ok(string) => string,
            Err(e) => String::new(),
        };
        let svg_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/Arduino_Dos/arduino_dos.svg") {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        assert!(!toml_string.is_empty());
        assert!(!svg_string.is_empty());

        assert_eq!(toml_string, "name = \"Arduino Dos\"\n\
        manufacturer = \"UnitTesting\"\n\
        board_type = \"Main\"\n\
        standard = \"Arduino\"\n\
        cpu = \"Cortex\"\n\
        ram = 32\n\
        flash = 16\n\
        required_crates = [\"embedded-hal\", \"ravedude\", \"panic-halt\"]\n\
        related_crates = [\"smart-leds\"]\n\n\

        bsp = \"iron-coder-Arduino-Dos-bsp\"\n\n\

        [[pinout]]\n\
        pins = [\"rx\", \"tx\"]\n\
        interface = { iface_type = \"UART\", direction = \"Bidirectional\" }\n\n\

        [[pinout]]\n\
        pins = [\"d0\", \"d1\"]\n\
        interface = { iface_type = \"GPIO\", direction = \"Bidirectional\" }\n\n");

    }
    #[test]
    pub fn save_new_board_invalid_svg_path() {
        let mut project: Project = Project{name : "test_toml_project".to_string(), location: None, system: Default::default(), code_editor: Default::default(), terminal_buffer: "".to_string(), receiver: None, current_view: Default::default(), known_boards: vec![], repo: None };
        let pin1 = PinoutTomlInfo {
            pins: vec![String::from("rx"), String::from("tx")],
            iface_type: UART,
            direction: Bidirectional,
        };
        let pin2 = PinoutTomlInfo {
            pins: vec![String::from("d0"), String::from("d1")],
            iface_type: GPIO,
            direction: Bidirectional,
        };
        let mut board_toml = BoardTomlInfo{
            name: String::from("ArduinoOne"),
            manufacturer: "UnitTesting".to_string(),
            board_type: Main,
            standard: Arduino.to_string(),
            cpu: "Cortex".to_string(),
            ram: 32,
            flash: 16,
            required_crates: vec![String::from("embedded-hal"), String::from("ravedude"), String::from("panic-halt")],
            related_crates: vec![String::from("smart-leds")],
            bsp: "".to_string(),
            pinouts: vec![pin1, pin2]
        };


        let mut ctx = egui::Context::default();
        let board_toml_info_id = egui::Id::new("board_toml_info");
        let new_board_svg_string_id = egui::Id::new("new_board_svg_string");
        let save_failure_id = egui::Id::new("save_board_FAILED");
        ctx.data_mut(|data| {
            data.insert_temp(board_toml_info_id, board_toml);
        });

        let svg_string = match fs::read_to_string( PathBuf::from("./iron-coder-boards/Arduino/Arduino_Uno/arduino_uno.svg")) {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        ctx.data_mut(|data| {
            data.insert_temp(new_board_svg_string_id, svg_string);
        });


        Project::save_new_board_info(&mut project, &mut ctx);

        let toml_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/ArduinoOne/arduinoone.toml") {
            Ok(string) => string,
            Err(e) => String::new(),
        };
        let svg_string = match fs::read_to_string("./iron-coder-boards/UnitTesting/ArduinoOne/arduinoone.svg") {
            Ok(string) => string,
            Err(e) => String::new(),
        };

        assert!(!toml_string.is_empty());

    }

}