use clap::{Parser, Subcommand};
use error::{AppError, AppResult};
use packages::Packages;
use reqwest::header::HeaderMap;
use std::path::PathBuf;

mod error;
mod packages;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// The List of packages to use
    #[arg(required = true, short, long, value_name = "PACKAGE_NAME")]
    package_names: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List Packages
    List {
        #[arg(short, long, default_value_t = false)]
        raw: bool,
        #[arg(short, long, default_value_t = false)]
        debug: bool,
    },
    /// Run Garbage Collector
    GC { skip: Option<usize> },
}

#[tokio::main]
async fn main() -> AppResult {
    let cli = Cli::parse();

    if let Some(config) = cli.config.as_deref() {
        dotenvy::from_filename(config)?;
    } else {
        dotenvy::dotenv()?;
    }

    let package_type = "container";

    for package_name in cli.package_names {
        println!("Package: {package_name}");
        // https://api.github.com/user/packages/PACKAGE_TYPE/PACKAGE_NAME/versions
        let json = get(&format!(
            "https://api.github.com/user/packages/{package_type}/{package_name}/versions"
        ))
        .await?;

        let packages: Packages = serde_json::from_str(&json)?;

        match &cli.command {
            Some(Commands::List { raw, debug }) => {
                println!("Printing lists...");

                if *debug {
                    println!("{}", json);
                }

                for package in packages {
                    if *raw {
                        println!("{:#?}", package)
                    } else {
                        println!(
                            "{} {}",
                            package.created_at,
                            package
                                .metadata
                                .container
                                .tags
                                .first()
                                .map(String::as_str)
                                .unwrap_or_else(|| {
                                    eprintln!("Tag Not Found");
                                    "<Unkown>"
                                })
                        )
                    }
                }
            }
            Some(Commands::GC { skip }) => {
                let skip = if let Some(skip) = *skip { skip } else { 4 };
                println!("Running GC...");

                for package in packages.iter().skip(skip) {
                    let id = package.id;

                    println!(
                        "Deleting: {}; tag:{}",
                        id,
                        package
                            .metadata
                            .container
                            .tags
                            .first()
                            .map(String::as_str)
                            .unwrap_or_else(|| {
                                eprintln!("Tag Not Found");
                                "<Unkown>"
                            })
                    );

                    delete(&format!(
                        "https://api.github.com/user/packages/{}/{}/versions/{}",
                        package_type, package_name, package.id
                    ))
                    .await?;
                }
            }
            None => {}
        }
    }

    Ok(())
}

async fn get(url: &str) -> Result<String, AppError> {
    Ok(reqwest::Client::new()
        .get(url)
        .headers(get_headers()?)
        .send()
        .await?
        .text()
        .await?)
}

async fn delete(url: &str) -> Result<(), AppError> {
    reqwest::Client::new()
        .delete(url)
        .headers(get_headers()?)
        .send()
        .await?;

    Ok(())
}

fn get_headers() -> anyhow::Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "github_package_gc".parse()?);
    headers.insert("Accept", "application/vnd.github+json".parse()?);
    headers.insert(
        "Authorization",
        format!(
            "Bearer {}",
            std::env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN .env or EnvVar")
        )
        .parse()?,
    );
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);
    Ok(headers)
}
