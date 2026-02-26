use bitwarden::secrets_manager::{
    access_policies::{AccessPoliciesResponse, GrantedPoliciesResponse, PotentialGranteesResponse},
    projects::ProjectResponse,
    secrets::SecretResponse,
};
use bitwarden_cli::Color;
use chrono::{DateTime, Utc};
use comfy_table::Table;
use serde::Serialize;

use crate::{cli::Output, util::is_valid_posix_name};

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

fn pretty_print(language: &str, data: &str, color: Color) {
    if color.is_enabled() {
        bat::PrettyPrinter::new()
            .input_from_bytes(data.as_bytes())
            .language(language)
            .print()
            .expect("Input is valid");
    } else {
        print!("{}", data);
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

fn permission_label(read: bool, write: bool, manage: bool) -> &'static str {
    if manage {
        "manage"
    } else if write {
        "write"
    } else if read {
        "read"
    } else {
        "none"
    }
}

/// Flatten all three policy lists into a unified 4-column table.
impl TableSerialize<4> for AccessPoliciesResponse {
    fn get_headers() -> [&'static str; 4] {
        ["Type", "ID", "Name", "Permission"]
    }

    fn get_values(&self) -> Vec<[String; 4]> {
        let mut rows = Vec::new();

        for p in &self.user_access_policies {
            rows.push([
                "User".to_string(),
                p.organization_user_id.to_string(),
                p.organization_user_name.clone().unwrap_or_default(),
                permission_label(p.policy.read, p.policy.write, p.policy.manage).to_string(),
            ]);
        }

        for p in &self.group_access_policies {
            rows.push([
                "Group".to_string(),
                p.group_id.to_string(),
                p.group_name.clone().unwrap_or_default(),
                permission_label(p.policy.read, p.policy.write, p.policy.manage).to_string(),
            ]);
        }

        for p in &self.service_account_access_policies {
            rows.push([
                "MachineAccount".to_string(),
                p.service_account_id.to_string(),
                p.service_account_name.clone().unwrap_or_default(),
                permission_label(p.policy.read, p.policy.write, p.policy.manage).to_string(),
            ]);
        }

        rows
    }
}

impl TableSerialize<3> for GrantedPoliciesResponse {
    fn get_headers() -> [&'static str; 3] {
        ["Project ID", "Project Name", "Permission"]
    }

    fn get_values(&self) -> Vec<[String; 3]> {
        self.granted_project_policies
            .iter()
            .map(|p| {
                [
                    p.project_id.to_string(),
                    p.project_name.clone().unwrap_or_default(),
                    permission_label(p.policy.read, p.policy.write, p.policy.manage).to_string(),
                ]
            })
            .collect()
    }
}

impl TableSerialize<4> for PotentialGranteesResponse {
    fn get_headers() -> [&'static str; 4] {
        ["ID", "Name", "Type", "Email"]
    }

    fn get_values(&self) -> Vec<[String; 4]> {
        self.data
            .iter()
            .map(|g| {
                [
                    g.id.to_string(),
                    g.name.clone().unwrap_or_default(),
                    g.r#type.clone().unwrap_or_default(),
                    g.email.clone().unwrap_or_default(),
                ]
            })
            .collect()
    }
}
