use crate::project::{Application, Package, Project};
use clap::ArgMatches;
use colored::Colorize;
use dialoguer;
use failure::{bail, format_err, Error};
use serde::Serialize;
use serde_json;
use slog::Logger;
use std::{fs::OpenOptions, io::BufWriter};

pub fn run(matches: &ArgMatches, _logger: &Logger) -> Result<(), Error> {
    let options = vec!["application", "package"];
    let option_idx = dialoguer::Select::new()
        .with_prompt("What type of elm.json file do you want to create?")
        .items(&options)
        .default(0)
        .interact()?;

    match options[option_idx] {
        "application" => create_application(matches),
        "package" => create_package(matches),
        _ => unreachable!(),
    }
}

fn create_package(_matches: &ArgMatches) -> Result<(), Error> {
    println!("Need settings: name, description, license, dependencies, test-dependencies");

    let name = until_valid(
        validate_package_name,
        "Enter a name for your package: (format: author/project)",
    )?;
    let summary = until_valid(
        validate_summary,
        "Enter a summary for your package (max 80 characters)",
    )?;

    let license_options = vec!["BSD-3-Clause", "MIT", "other..."];
    let license_option_idx = dialoguer::Select::new()
        .with_prompt("Choose a license for your package")
        .items(&license_options)
        .default(0)
        .interact()?;

    let license = match license_options[license_option_idx] {
        "other..." => until_valid(
            |input| {
                if APPROVED_LICENSES.contains(&&*input) {
                    Ok(input)
                } else {
                    Err(format_err!("Please pick a valid license"))
                }
            },
            "License in SPDX format",
        )?,
        selected_license => selected_license.to_string(),
    };

    let proj = Project::Package(Package::new(name, summary, license));
    create_elm_json(&proj)
}

fn validate_summary(summary: String) -> Result<String, Error> {
    if summary.len() > 80 {
        bail!("Summary may not be over 80 characters long.")
    }

    Ok(summary)
}

fn validate_package_name(name: String) -> Result<String, Error> {
    let parts: Vec<_> = name.trim().split('/').collect();
    match parts.as_slice() {
        [author, project] => validate_author(author)
            .and(validate_project(project))
            .map(|_| name.trim().into()),
        _ => Err(format_err!(
            "A valid package name look like \"author/project\""
        )),
    }
}

fn validate_author(author: &str) -> Result<(), Error> {
    if author.is_empty() {
        bail!("Author name may not be empty. A valid package name looks like \"author/project\".")
    }

    if author.starts_with('-') {
        bail!("Author name may not start with a dash. Please use your github username!")
    }

    if author.ends_with('-') {
        bail!("Author name may not end with a dash. Please user your github username!")
    }

    if author.contains("--") {
        bail!("Author name may not contain a double dash. Please use your github username!")
    }

    if author.len() > 39 {
        bail!("Author name may not be over 39 characters long. Please use your github username!")
    }

    if !author
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        bail!("Author name may only contain ascii alphanumeric characters.")
    }

    Ok(())
}

fn validate_project(project: &str) -> Result<(), Error> {
    if project.is_empty() {
        bail!(
            "Project name maybe not be empty. A valid package name looks like \"author/project\"."
        )
    }

    if project.contains("--") {
        bail!("Project name cannot contain a double dash.")
    }

    if project.ends_with('-') {
        bail!("Project name cannot end with a dash.")
    }

    if !project
        .chars()
        .all(|x| x.is_ascii_lowercase() || x.is_digit(10) || x == '-')
    {
        bail!("Project name may only contains lowercase letters, digits and dashes.")
    }

    if !project.chars().nth(0).unwrap().is_ascii_lowercase() {
        bail!("Project name must start with a letter")
    }

    Ok(())
}

fn until_valid<F>(validate: F, prompt: &str) -> Result<String, Error>
where
    F: Fn(String) -> Result<String, Error>,
{
    let mut res: String;

    loop {
        res = dialoguer::Input::new().with_prompt(prompt).interact()?;
        match validate(res) {
            Ok(v) => {
                res = v;
                break;
            }
            Err(e) => println!("{}: {}", "Error".red(), e),
        }
    }
    Ok(res)
}

fn create_application(_matches: &ArgMatches) -> Result<(), Error> {
    let proj = Project::Application(Application::new());
    create_elm_json(&proj)
}

fn create_elm_json(info: &Project) -> Result<(), Error> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("elm.json")?;
    let writer = BufWriter::new(file);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut serializer = serde_json::Serializer::with_formatter(writer, formatter);

    info.serialize(&mut serializer)?;
    Ok(())
}

const APPROVED_LICENSES: &[&str] = &[
    "AFL-1.1",
    "AFL-1.2",
    "AFL-2.0",
    "AFL-2.1",
    "AFL-3.0",
    "APL-1.0",
    "Apache-1.1",
    "Apache-2.0",
    "APSL-1.0",
    "APSL-1.1",
    "APSL-1.2",
    "APSL-2.0",
    "Artistic-1.0",
    "Artistic-1.0-Perl",
    "Artistic-1.0-cl8",
    "Artistic-2.0",
    "AAL",
    "BSL-1.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "0BSD",
    "CECILL-2.1",
    "CNRI-Python",
    "CDDL-1.0",
    "CPAL-1.0",
    "CPL-1.0",
    "CATOSL-1.1",
    "CUA-OPL-1.0",
    "EPL-1.0",
    "ECL-1.0",
    "ECL-2.0",
    "EFL-1.0",
    "EFL-2.0",
    "Entessa",
    "EUDatagrid",
    "EUPL-1.1",
    "Fair",
    "Frameworx-1.0",
    "AGPL-3.0",
    "GPL-2.0",
    "GPL-3.0",
    "LGPL-2.1",
    "LGPL-3.0",
    "LGPL-2.0",
    "HPND",
    "IPL-1.0",
    "Intel",
    "IPA",
    "ISC",
    "LPPL-1.3c",
    "LiLiQ-P-1.1",
    "LiLiQ-Rplus-1.1",
    "LiLiQ-R-1.1",
    "LPL-1.02",
    "LPL-1.0",
    "MS-PL",
    "MS-RL",
    "MirOS",
    "MIT",
    "Motosoto",
    "MPL-1.0",
    "MPL-1.1",
    "MPL-2.0",
    "MPL-2.0-no-copyleft-exception",
    "Multics",
    "NASA-1.3",
    "Naumen",
    "NGPL",
    "Nokia",
    "NPOSL-3.0",
    "NTP",
    "OCLC-2.0",
    "OGTSL",
    "OSL-1.0",
    "OSL-2.0",
    "OSL-2.1",
    "OSL-3.0",
    "OSET-PL-2.1",
    "PHP-3.0",
    "PostgreSQL",
    "Python-2.0",
    "QPL-1.0",
    "RPSL-1.0",
    "RPL-1.1",
    "RPL-1.5",
    "RSCPL",
    "OFL-1.1",
    "SimPL-2.0",
    "Sleepycat",
    "SISSL",
    "SPL-1.0",
    "Watcom-1.0",
    "UPL-1.0",
    "NCSA",
    "VSL-1.0",
    "W3C",
    "Xnet",
    "Zlib",
    "ZPL-2.0",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_author() {
        assert!(validate_author("foobar").is_ok());
        assert!(validate_author("").is_err());
        assert!(validate_author("\n").is_err());
        assert!(validate_author("1").is_ok());
        assert!(validate_author("foo-bar-123").is_ok());
        assert!(validate_author("-foo").is_err());
        assert!(validate_author("foo-").is_err());
    }

    #[test]
    fn test_validate_project() {
        assert!(validate_project("foobar").is_ok());
        assert!(validate_project("").is_err());
        assert!(validate_project("fo-").is_err());
        assert!(validate_project("f-o").is_ok());
    }
}