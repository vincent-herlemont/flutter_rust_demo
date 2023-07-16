use clap::{Parser, Subcommand};
use config::Config;
use control_plane::{SupabaseClient, VersionDeployment};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    cmd: Commands,
    #[clap(long, default_value = ".env")]
    dotenv: String,
}

#[derive(Subcommand)]
enum Commands {
    New { version: String },
    Fail { version: String },
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let Cli { dotenv, .. } = cli;
    let config = Config::new(dotenv);
    let client = SupabaseClient::new(&config);
    match cli.cmd {
        Commands::New { version } => {
            let res = VersionDeployment::insert(&client, &version).await.unwrap();
            println!("{:?}", res);
            println!("New deployment: {}", version);
        }
        Commands::Fail { version } => {
            let res = VersionDeployment::update_status(
                &client,
                &version,
                control_plane::VersionDeploymentStatus::Failed,
            )
            .await
            .unwrap();
            println!("{:?}", res);
            println!("Deployment {} set to failed", version);
        }
        Commands::List => {
            let res = VersionDeployment::list_all(&client).await.unwrap();
            for version_deployment in res {
                println!("{:?}", version_deployment);
            }
        }
    }
}
