use clap::Parser;
use inquire::{Confirm, Text};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    load: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Subject {
    domain: String,
    context: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct KnowledgeBase {
    subjects: Vec<Subject>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PromptFilter {
    domain: String,
    context: String,
    variables: Vec<String>,
    constraints: String,
    prompt_body: String,
}

impl fmt::Display for PromptFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Domain: {}]\n[Context: {}]\n[Variables: {}]\n[Output Constraint: {}]\n[Prompt: {}]",
            self.domain, self.context, self.variables.join(", "), self.constraints, self.prompt_body
        )
    }
}

fn main() {
    let args = Args::parse();
    let dir_name = "filter_files";
    fs::create_dir_all(dir_name).ok();

    let filter = if let Some(path) = args.load.as_ref() {
        let content = fs::read_to_string(path).expect("Failed to read template");
        toml::from_str(&content).expect("Invalid TOML")
    } else {
        run_interactive_session()
    };

    println!("\n--- Final Vector Filter ---\n\n{}", filter);

    if args.load.is_none() && Confirm::new("Save to library?").with_default(true).prompt().unwrap() {
        save_filter(dir_name, &filter);
    }
}

fn run_interactive_session() -> PromptFilter {
    let kb_path = "knowledge_base.toml";
    let (p_domain, p_context) = if let Ok(content) = fs::read_to_string(kb_path) {
        let kb: KnowledgeBase = toml::from_str(&content).unwrap_or(KnowledgeBase { subjects: vec![] });
        
        // FIX: Using the 0.10 location for thread_rng and the IndexedRandom trait
        let mut rng = rand::rng(); 
        kb.subjects.choose(&mut rng)
            .map(|s| (s.domain.clone(), s.context.clone()))
            .unwrap_or(("Biochemistry".into(), "Metabolic recovery".into()))
    } else {
        ("Biochemistry".into(), "Metabolic recovery".into())
    };

    let domain = Text::new("Domain:")
        .with_placeholder(&format!("e.g., {}", p_domain))
        .prompt()
        .unwrap();

    let context = Text::new("Context:")
        .with_placeholder(&format!("e.g., {}", p_context))
        .prompt()
        .unwrap();

    let vars_raw = Text::new("Variables (comma-separated):")
        .with_placeholder("e.g., Glutathione, Vitamin E, Magnesium")
        .prompt()
        .unwrap();
    
    let variables = vars_raw.split(',').map(|s| s.trim().to_string()).collect();
    
    let constraints = Text::new("Output Constraints:")
        .with_default("Strict mechanics, no conversational filler")
        .prompt()
        .unwrap();
        
    let prompt_body = Text::new("Prompt Directive:")
        .with_placeholder("e.g., Explain the catalytic interaction of these variables")
        .prompt()
        .unwrap();

    PromptFilter { domain, context, variables, constraints, prompt_body }
}

fn save_filter(dir: &str, filter: &PromptFilter) {
    let name = Text::new("Filename:").with_default("filter_export").prompt().unwrap();
    let file_path = Path::new(dir).join(format!("{}.toml", name));
    let serialized = toml::to_string(filter).unwrap();
    fs::write(&file_path, serialized).expect("Save failed");
    println!("Archived to {}", file_path.display());
}