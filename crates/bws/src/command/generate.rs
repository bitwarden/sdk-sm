use bitwarden::{
    Client,
    generators::{GeneratorClientsExt, PasswordGeneratorRequest},
};
use color_eyre::eyre::Result;

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_secret(
    include_lowercase: bool,
    include_uppercase: bool,
    include_numbers: bool,
    length: u8,
    include_special: bool,
    include_ambiguous: bool,
    min_lowercase: Option<u8>,
    min_uppercase: Option<u8>,
    min_number: Option<u8>,
    min_special: Option<u8>,
) -> Result<()> {
    let input = PasswordGeneratorRequest {
        lowercase: include_lowercase,
        uppercase: include_uppercase,
        numbers: include_numbers,
        length,
        special: include_special,
        avoid_ambiguous: !include_ambiguous,
        min_lowercase,
        min_uppercase,
        min_number,
        min_special,
    };

    let generated_secret = Client::new(None).generator().password(input)?;
    print!("{generated_secret}");

    Ok(())
}
