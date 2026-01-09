use bitwarden::secrets_manager::{projects::ProjectResponse, secrets::SecretResponse};
use bitwarden_cli::Color;
use chrono::{DateTime, Utc};
use comfy_table::Table;
use serde::Serialize;

use crate::{cli::Output, util::is_valid_posix_name};

const ASCII_HEADER_ONLY: &str = "     --            ";

pub(crate) struct OutputSettings {
    pub(crate) output: Output,
    pub(crate) color: Color,
    pub(crate) theme: String,
}

impl OutputSettings {
    pub(crate) fn new(output: Output, color: Color, theme: String) -> Self {
        OutputSettings {
            output,
            color,
            theme,
        }
    }
}

pub(crate) fn serialize_response<T: Serialize + TableSerialize<N>, const N: usize>(
    data: T,
    output_settings: OutputSettings,
) {
    match output_settings.output {
        Output::JSON => {
            let mut data =
                serde_json::to_string_pretty(&data).expect("Serialize should be infallible");
            // Yaml/table/tsv serializations add a newline at the end, so we do the same here for
            // consistency
            data.push('\n');
            pretty_print("json", &data, output_settings);
        }
        Output::YAML => {
            let data = serde_yaml::to_string(&data).expect("Serialize should be infallible");
            pretty_print("yaml", &data, output_settings);
        }
        Output::Env => {
            let mut commented_out = false;
            let mut data: Vec<String> = data
                .get_values()
                .into_iter()
                .map(|row| {
                    if is_valid_posix_name(&row[1]) {
                        format!("{}=\"{}\"", &row[1], &row[2])
                    } else {
                        commented_out = true;
                        format!("# {}=\"{}\"", &row[1], &row[2].replace('\n', "\n# "))
                    }
                })
                .collect();

            if commented_out {
                data.push(String::from(
                    "\n# one or more secrets have been commented-out due to a problematic key name",
                ));
            }

            pretty_print("sh", &format!("{}\n", data.join("\n")), output_settings);
        }
        Output::Table => {
            let mut table = Table::new();
            table
                .load_preset(ASCII_HEADER_ONLY)
                .set_header(T::get_headers())
                .add_rows(data.get_values());

            println!("{table}");
        }
        Output::TSV => {
            println!("{}", T::get_headers().join("\t"));

            let rows: Vec<String> = data
                .get_values()
                .into_iter()
                .map(|row| row.join("\t"))
                .collect();
            println!("{}", rows.join("\n"));
        }
        Output::None => {}
    }
}

fn pretty_print(language: &str, data: &str, output_settings: OutputSettings) {
    bat::PrettyPrinter::new()
        .input_from_bytes(data.as_bytes())
        .language(&language)
        .theme(output_settings.theme)
        .colored_output(output_settings.color.is_enabled())
        .print()
        .expect("Input is valid");
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
