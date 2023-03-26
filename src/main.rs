use clap::{Parser, Subcommand};
use error::{AppError, AppResult};
use reqwest::header::HeaderMap;
use std::{path::PathBuf, str::FromStr};

mod error;

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
        let json: serde_json::Value = get(&format!(
            "https://api.github.com/user/packages/{package_type}/{package_name}/versions"
        ))
        .await?;

        let serde_json::Value::Array(result) = &json else {
            return Ok(())
        };

        match &cli.command {
            Some(Commands::List { raw }) => {
                println!("Printing lists...");
                for item in result {
                    let serde_json::Value::String(created_at) = &item["created_at"] else {
                        return Ok(())
                    };

                    if *raw {
                        println!("{:#?}", item)
                    } else {
                        let created_at: chrono::DateTime<chrono::Utc> =
                            chrono::DateTime::from_str(created_at)?;

                        println!(
                            "{} {}",
                            created_at, &item["metadata"]["container"]["tags"][0]
                        )
                    }
                }
            }
            Some(Commands::GC { skip }) => {
                let skip = if let Some(skip) = *skip { skip } else { 4 };
                println!("Running GC...");

                for item in result.iter().skip(skip) {
                    let serde_json::Value::Object(item) = item else {
                        return Ok(())
                    };

                    let serde_json::Value::Number(id) = &item["id"] else {
                        return Ok(())
                    };

                    let id = id
                        .as_i64()
                        .ok_or(AppError::InvalidId(serde_json::to_string_pretty(&json)?))?;

                    println!(
                        "Deleting: {}; tag:{}",
                        id, &item["metadata"]["container"]["tags"][0]
                    );

                    delete(&format!(
                        "https://api.github.com/user/packages/{package_type}/{package_name}/versions/{id}"
                    ))
                    .await?;
                }
            }
            None => {}
        }
    }

    Ok(())
}

async fn get<T>(url: &str) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    let t = reqwest::Client::new()
        .get(url)
        .headers(get_headers()?)
        .send()
        .await?
        .json()
        .await?;

    Ok(t)
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
