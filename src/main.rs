use dotenv::dotenv;
use run_script::run_script;
use std::{
    collections::hash_map::DefaultHasher,
    fs::{DirEntry, File},
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
    process::Command,
};

use rocket::{
    http::Status,
    serde::{json::Json, Deserialize},
};

#[macro_use]
extern crate rocket;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DeployRequest<'r> {
    giturl: &'r str,
    name: &'r str,
    downdir: Option<&'r str>,
    token: &'r str,
}

#[post("/deploy", data = "<request>")]
fn deploy(request: Json<DeployRequest<'_>>) -> Result<String, Status> {
    let token = std::env::var("TOKEN").expect("No TOKEN variable was found in the environment");
    if token != request.token {
        return Err(Status::Unauthorized);
    }
    let name: String = get_project_name(&request);
    let mut checkout = format!("/etc/simple-server-deployment/checkouts/{}", name);
    Command::new("git")
        .arg("clone")
        .arg(request.giturl)
        .arg(&checkout)
        .output()
        .unwrap_or_else(|_| panic!("Unable to clone the url {}", request.giturl));

    let config_dir = format!("/etc/simple-server-deployment/conf/{}", name);
    if let Some(downdir) = request.downdir {
        checkout.push('/');
        checkout.push_str(downdir);
    }
    let from = format!("{}/conf", checkout);
    std::fs::create_dir_all(&config_dir).unwrap();
    for file in std::fs::read_dir(&from).unwrap() {
        let file = file.unwrap();
        std::fs::copy(
            file.path(),
            format!("{}/{}", config_dir, file.file_name().to_str().unwrap()),
        )
        .unwrap();
    }

    run_in_all_files(Path::new(&config_dir), &replace_env_with_values)
        .expect("unable to change environment variables");

    Ok(format!("Hello, world! {}, {}", name, token))
}

fn replace_env_with_values(file: &DirEntry) {
    if let Some(full_path) = file.path().to_str() {
        if !full_path.ends_with(".template") {
            return;
        }
        run_script!(format!(
            "envsubst < {} > {}",
            full_path,
            full_path.replace(".template", "")
        ))
        .unwrap_or_else(|_| panic!("unable to run command 'envsubst' on file {}", full_path));
        std::fs::remove_file(full_path)
            .unwrap_or_else(|_| panic!("unable to remove file {}", full_path))
    }
}

fn run_in_all_files(dir: &Path, cb: &dyn Fn(&DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                run_in_all_files(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

#[derive(Hash)]
struct Project<'r> {
    name: &'r str,
    giturl: &'r str,
    downdir: Option<&'r str>,
}

fn get_project_name(request: &DeployRequest<'_>) -> String {
    let project = Project {
        giturl: request.giturl,
        downdir: request.downdir,
        name: request.name,
    };
    let mut hasher = DefaultHasher::new();
    project.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{}-{}", request.name, hash)
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build().mount("/", routes![deploy])
}
