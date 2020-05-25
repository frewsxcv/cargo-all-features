use std::process;
use termcolor::WriteColor;

pub struct CargoTestRunner {
    command: process::Command,
    feature_set: Vec<String>,
}

impl CargoTestRunner {
    pub fn new(feature_set: Vec<String>) -> Self {
        let command = process::Command::new(&crate::cargo_cmd());

        let mut s = CargoTestRunner {
            command,
            feature_set,
        };
        s.arg("test");
        s.arg("--no-default-features");

        s
    }

    pub fn arg(&mut self, arg: &str) {
        self.command.arg(arg);
    }

    pub fn run(&mut self) {
        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
        stdout
            .set_color(
                termcolor::ColorSpec::new()
                    .set_fg(Some(termcolor::Color::Cyan))
                    .set_bold(true),
            )
            .unwrap();
        print!("    Features ");
        stdout.reset().unwrap();
        println!("[{}]", self.feature_set.join(", "));

        let output = self
            .command
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .output()
            .unwrap(); // fixme

        if !output.status.success() {
            panic!("todo"); // fixme
        }
    }
}
