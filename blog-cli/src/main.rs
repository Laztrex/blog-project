use blog_client::{BlogClient, BlogClientError, Transport};
use clap::{Parser, Subcommand};
use std::fs;

const TOKEN_FILE: &str = ".blog_token";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    grpc: bool,

    #[arg(short, long)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        username: String,
        email: String,
        password: String,
    },
    Login {
        username: String,
        password: String,
    },
    Create {
        title: String,
        content: String,
    },
    Get {
        id: i64,
    },
    Update {
        id: i64,
        title: String,
        content: String,
    },
    Delete {
        id: i64,
    },
    List {
        #[arg(default_value_t = 10)]
        limit: i32,
        #[arg(default_value_t = 0)]
        offset: i32,
    },
}

/// save_token сохраняет JWT-токен в файл .blog_token в домашней директории.
fn save_token(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let path = home.join(TOKEN_FILE);
    fs::write(path, token)?;
    Ok(())
}

/// load_token загружает токен из файла .blog_token, если он существует.
fn load_token() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(TOKEN_FILE);
    fs::read_to_string(path).ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let transport = if cli.grpc {
        let addr = cli
            .server
            .unwrap_or_else(|| "http://localhost:50051".into());
        Transport::Grpc(addr)
    } else {
        let base = cli.server.unwrap_or_else(|| "http://localhost:3000".into());
        Transport::Http(Some(base))
    };

    let mut client = BlogClient::new(transport).await?;
    if let Some(token) = load_token() {
        client.set_token(token);
    }

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let resp = client.register(username, email, password).await?;
            save_token(&resp.token)?;
            println!(
                "Registration successful! Logged in as {}",
                resp.user.username
            );
        }
        Commands::Login { username, password } => {
            let resp = client.login(username, password).await?;
            save_token(&resp.token)?;
            println!("Login successful! Welcome {}", resp.user.username);
        }
        Commands::Create { title, content } => {
            let post = client.create_post(title, content).await?;
            println!("Post created with id: {}", post.id);
        }
        Commands::Get { id } => match client.get_post(id).await {
            Ok(post) => {
                println!("Title: {}", post.title);
                println!("Content: {}", post.content);
                println!("Author: {}", post.author_id);
                println!("Created: {}", post.created_at);
            }
            Err(BlogClientError::NotFound(msg)) => eprintln!("{}", msg),
            Err(e) => return Err(e.into()),
        },
        Commands::Update { id, title, content } => {
            let post = client.update_post(id, title, content).await?;
            println!("Post {} updated: {}", post.id, post.title);
        }
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Post {} deleted", id);
        }
        Commands::List { limit, offset } => {
            let (posts, total) = client.list_posts(limit, offset).await?;
            println!("Posts (total {}):", total);
            for post in posts {
                println!("{}: {} (by user {})", post.id, post.title, post.author_id);
            }
        }
    }

    Ok(())
}
