#[macro_use] extern crate rocket;

use rocket::{get, launch, routes};
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use serde_json;
use walkdir::WalkDir;
use chrono::{DateTime, Utc}; // Add chrono for date-time handling

// Struct Definitions
#[derive(Deserialize, Serialize, Debug)]
pub struct Owner {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
    pub user_view_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: Owner,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub created_at: String,
    pub updated_at: String, // This field will be used to sort repositories
    pub pushed_at: String,
    pub stargazers_count: u64,
    pub watchers_count: u64,
    pub has_discussions: bool,
    pub forks_count: u64,
    pub allow_forking: bool,
    pub is_template: bool,
    pub web_commit_signoff_required: bool,
    pub topics: Vec<String>,
    pub visibility: String,
    pub forks: u64,
    pub watchers: u64,
    pub score: u64,
    pub subscribers_count: u64,
}

impl Repository {
    // Helper method to parse the `updated_at` string into DateTime<Utc>
    fn parse_updated_at(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.updated_at)
            .map(|dt| dt.with_timezone(&Utc)) // Convert to UTC
    }
}

// Function to read JSON files from a directory recursively
fn read_json_files_from_directory<P: AsRef<Path>>(dir: P) -> Vec<String> {
    let mut json_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        if entry.path().extension().map(|ext| ext == "json").unwrap_or(false) {
            json_files.push(entry.path().to_str().unwrap().to_string());
        }
    }

    json_files
}

// Function to deserialize a JSON file into Repository objects
fn deserialize_json_file(file_path: &str) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file_path)?;
    let repos: Vec<Repository> = serde_json::from_str(&data)?;
    Ok(repos)
}

// Function to read and deserialize all JSON files from a directory recursively
fn read_all_repositories_from_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
    let paths = read_json_files_from_directory(dir);
    
    let mut all_repos = Vec::new();
    for path in paths {
        match deserialize_json_file(&path) {
            Ok(repos) => all_repos.extend(repos),
            Err(e) => eprintln!("Error reading or deserializing file {}: {}", path, e),
        }
    }

    Ok(all_repos)
}

#[get("/repositories")]
async fn get_repositories() -> Template {
    // Specify the directory path containing JSON files
    let directory_path = "PoC-in-GitHub";

    // Read and deserialize all JSON files in the directory recursively
    let mut repositories = match read_all_repositories_from_directory(directory_path) {
        Ok(repos) => repos,
        Err(e) => {
            eprintln!("Error: {}", e);
            vec![]  // Return an empty vector in case of error
        }
    };

    // Sort the repositories by the `updated_at` field (most recent first)
    repositories.sort_by(|a, b| {
        // Handle both successful and unsuccessful date parsing
        match (a.parse_updated_at(), b.parse_updated_at()) {
            (Ok(a_date), Ok(b_date)) => a_date.cmp(&b_date).reverse(),
            (Ok(_), Err(_)) => std::cmp::Ordering::Greater, // If `a` is Ok and `b` is Err, `a` is more recent
            (Err(_), Ok(_)) => std::cmp::Ordering::Less,    // If `a` is Err and `b` is Ok, `b` is more recent
            (Err(_), Err(_)) => std::cmp::Ordering::Equal,   // If both are Err, consider them equal
        }
    });

    // Create a HashMap to pass the data to the template
    let mut context = std::collections::HashMap::new();
    context.insert("repositories", repositories);

    // Render the "repositories.html.tera" template with the repository data
    Template::render("repositories", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_repositories])
        .attach(Template::fairing()) // Attach the template fairing
}
