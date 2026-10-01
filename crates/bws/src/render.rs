use std::io::{self, Write};

use bitwarden::secrets_manager::{projects::ProjectResponse, secrets::SecretResponse};
use bitwarden_cli::Color;
use chrono::{DateTime, Utc};
use comfy_table::Table;
use serde::Serialize;

use crate::{cli::Output, error, util::is_valid_posix_name};

const ASCII_HEADER_ONLY: &str = "     --            ";

pub(crate) struct OutputSettings {
    pub(crate) output: Output,
    pub(crate) color: Color,
}

impl OutputSettings {
    pub(crate) fn new(output: Output, color: Color) -> Self {
        OutputSettings { output, color }
    }
}

pub(crate) fn serialize_response<T: Serialize + TableSerialize<N>, const N: usize>(
    data: T,
    output_settings: OutputSettings,
) {
    match output_settings.output {
        Output::JSON => {
            let mut text =
                serde_json::to_string_pretty(&data).expect("Serialize should be infallible");
            // Yaml/table/tsv serializations add a newline at the end, so we do the same here for
            // consistency
            text.push('\n');
            pretty_print("json", &text, output_settings.color);
        }
        Output::YAML => {
            let text = serde_yaml::to_string(&data).expect("Serialize should be infallible");
            pretty_print("yaml", &text, output_settings.color);
        }
        Output::Env => {
            let mut commented_out = false;
            let mut text: Vec<String> = data
                .get_values()
                .into_iter()
                .map(|row| {
                    if is_valid_posix_name(&row[1]) {
                        format!("{}=\"{}\"", row[1], row[2])
                    } else {
                        commented_out = true;
                        format!("# {}=\"{}\"", row[1], row[2].replace('\n', "\n# "))
                    }
                })
                .collect();

            if commented_out {
                text.push(String::from(
                    "\n# one or more secrets have been commented-out due to a problematic key name",
                ));
            }

            pretty_print(
                "sh",
                &format!("{}\n", text.join("\n")),
                output_settings.color,
            );
        }
        Output::Table => {
            let mut table = Table::new();
            table
                .load_preset(ASCII_HEADER_ONLY)
                .set_header(T::get_headers())
                .add_rows(data.get_values());

            write_stdout(format!("{table}\n"));
        }
        Output::TSV => {
            // Built as one buffer so there is a single broken-pipe check per output.
            let mut text = T::get_headers().join("\t");
            text.push('\n');
            for (i, row) in data.get_values().into_iter().enumerate() {
                if i > 0 {
                    text.push('\n');
                }
                text.push_str(&row.join("\t"));
            }
            text.push('\n');
            write_stdout(text);
        }
        Output::None => {}
    }
}

fn pretty_print(language: &str, data: &str, color: Color) {
    if color.is_enabled() {
        let mut highlighted = String::new();
        bat::PrettyPrinter::new()
            .input_from_bytes(data.as_bytes())
            .language(language)
            .print_with_writer(Some(&mut highlighted))
            .expect("Input is valid");
        write_stdout(highlighted);
    } else {
        write_stdout(data);
    }
}

/// Writes `data` to stdout, exiting quietly if the reader has gone away (e.g. `bws ... | head`).
///
/// Any other write failure is environmental (a full disk, a closed descriptor), so it is reported
/// on stderr and exits non-zero instead of unwinding as a crash.
pub(crate) fn write_stdout(data: impl AsRef<[u8]>) {
    let mut stdout = io::stdout().lock();
    if let Err(e) = stdout
        .write_all(data.as_ref())
        .and_then(|()| stdout.flush())
    {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        // Stdout is unusable, so this cannot travel back up as a report and be printed there.
        eprint!(
            "{}",
            error::render_io_error("Could not write to stdout", &e)
        );
        std::process::exit(1);
    }
}

// We're using const generics for the array lengths to make sure the header count and value count
// match
pub(crate) trait TableSerialize<const N: usize>: Sized {
    fn get_headers() -> [&'static str; N];
    fn get_values(&self) -> Vec<[String; N]>;
}

// Generic impl for Vec<T> so we can call `serialize_response` with both individual
// elements and lists of elements, like we do with the JSON and YAML cases
impl<T: TableSerialize<N>, const N: usize> TableSerialize<N> for Vec<T> {
    fn get_headers() -> [&'static str; N] {
        T::get_headers()
    }
    fn get_values(&self) -> Vec<[String; N]> {
        let mut values = Vec::new();
        for t in self {
            values.append(&mut t.get_values());
        }
        values
    }
}

fn format_date(date: &DateTime<Utc>) -> String {
    date.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl TableSerialize<3> for ProjectResponse {
    fn get_headers() -> [&'static str; 3] {
        ["ID", "Name", "Creation Date"]
    }

    fn get_values(&self) -> Vec<[String; 3]> {
        vec![[
            self.id.to_string(),
            self.name.clone(),
            format_date(&self.creation_date),
        ]]
    }
}

impl TableSerialize<4> for SecretResponse {
    fn get_headers() -> [&'static str; 4] {
        ["ID", "Key", "Value", "Creation Date"]
    }

    fn get_values(&self) -> Vec<[String; 4]> {
        vec![[
            self.id.to_string(),
            self.key.clone(),
            self.value.clone(),
            format_date(&self.creation_date),
        ]]
    }
}
