use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Create a Renode script that loads an ELF file.
pub fn create_renode_script(elf_path: &Path, script_path: &Path) -> io::Result<()> {
    // Open the script file for writing (create it if it doesn't exist)
    let mut script_file = File::create(script_path)?;

    // Write Renode script commands
    writeln!(script_file, "# Renode script for ELF file")?;
    writeln!(script_file, "using sysbus")?;
    writeln!(script_file, "mach create")?;

    // Load the platform description
    writeln!(script_file, "machine LoadPlatformDescription @platforms/boards/stm32f4_discovery-kit.repl")?;

    // CPU settings
    writeln!(script_file, "cpu PerformanceInMips 125")?;

    // Optional: Define a macro for resetting
    writeln!(script_file, "macro reset")?;
    writeln!(script_file, "\"\"\"")?;
    writeln!(script_file, "sysbus LoadELF $CWD/{}", elf_path.display())?;
    writeln!(script_file, "\"\"\"")?;

    // Execute the reset macro
    writeln!(script_file, "runMacro $reset")?;

    // Setup the log file for output
    writeln!(script_file, "logFile $ORIGIN/output.txt")?;

    // Show analyzer (Could be moved to a separate function)
    writeln!(script_file, "showAnalyzer sysbus.usart2")?;

    // Start the simulation
    writeln!(script_file, "start")?;

    // Optional: Log GPIO activity (for testing peripherals)
    writeln!(script_file, "# New Testing for obtaining peripherals")?;
    writeln!(script_file, "logLevel -1 gpioPortD")?;

    // Enable logging of peripherals
    writeln!(script_file, "peripherals")?;
    writeln!(script_file, "logFile $ORIGIN/../../logs/output.txt")?;

    // Finish the script
    writeln!(script_file, "# Simulation ends")?;

    // Return success if no errors occurred
    Ok(())
}
