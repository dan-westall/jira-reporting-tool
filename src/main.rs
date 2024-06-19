use dotenv::dotenv;
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use structopt::StructOpt;
use comfy_table::{Table, ContentArrangement };

#[derive(Debug, StructOpt)]
#[structopt(name = "fetch_jira_tickets", about = "Fetch Jira Tickets")]
struct Opt {
    #[structopt(long, help = "Sprint ID")]
    sprint: Option<String>,
    #[structopt(long, help = "Date range in the format 'YYYY/MM/DD,YYYY/MM/DD'")]
    date: Option<String>,
    #[structopt(long, help = "Project key")]
    project: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    onboarding();

    let opt = Opt::from_args();

    let jira_base_url = env::var("JIRA_BASE_URL")?;
    let email = env::var("JIRA_EMAIL")?;
    let api_token = env::var("JIRA_API_TOKEN")?;
    let auth = format!("Basic {}", base64::encode(format!("{}:{}", email, api_token)));

    let jql = if let Some(sprint) = opt.sprint {
        format!("\"cf[10010]\"={} ORDER BY created DESC", sprint)
    } else if let (Some(date_range), Some(project)) = (opt.date, opt.project) {
        let dates: Vec<&str> = date_range.split(',').collect();
        if dates.len() != 2 {
            eprintln!("Invalid date range. Usage: --date \"2024/06/01,2024/06/15\"");
            std::process::exit(1);
        }
        let start_date = dates[0].trim();
        let end_date = dates[1].trim();
        format!(
            "project = \"{}\" AND status CHANGED TO \"Done\" DURING (\"{}\", \"{}\") ORDER BY created DESC",
            project, start_date, end_date
        )
    } else {
        eprintln!("Usage: --sprint <id> OR --date \"YYYY/MM/DD,YYYY/MM/DD\" --project <PROJECT_KEY>");
        std::process::exit(1);
    };

    let client = Client::new();
    let response = client
        .post(format!("{}/rest/api/3/search", jira_base_url))
        .header("Authorization", auth)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "jql": jql }))
        .send()?
        .json::<Value>()?; // Fetch raw response as JSON

    // Parse JSON response dynamically
    if let Some(issues) = response.get("issues").and_then(|v| v.as_array()) {
        let mut table1 = Table::new();
        table1
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(100)
            .set_header(vec!["Issue Number", "Title", "Business Value"]);
        let mut business_value_map: HashMap<String, Vec<String>> = HashMap::new();

        for issue in issues {
            let issue_number = issue.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let fields = issue.get("fields").unwrap_or(&Value::Null);
            let title = fields.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let description_string = fields.get("description").map_or("".to_string(), |d| parse_description(d));
            let business_value_content = extract_business_value_content(&description_string);

            table1.add_row(vec![issue_number, title, &*business_value_content]);

            if business_value_content != "No content found" {
                business_value_map
                    .entry(business_value_content.clone())
                    .or_insert_with(Vec::new)
                    .push(issue_number.to_string());
            }
        }

        println!("Detailed Ticket Information:");
        println!("{table1}");

        let mut table2 = Table::new();
        table2
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(100)
            .set_header(vec!["Business Value", "Ticket Numbers"]);

        for (business_value, ticket_numbers) in business_value_map {
            table2.add_row(vec![business_value, ticket_numbers.join(", ")]);
        }

        println!("Grouped Business Value Information:");
        println!("{table2}");

    }

    Ok(())
}

fn onboarding() {
    if !std::path::Path::new(".env").exists() {
        let jira_base_url = prompt("Enter your Jira base URL: ");
        let email = prompt("Enter your Jira email: ");
        let api_token = prompt("Enter your Jira API token: ");

        let mut file = File::create(".env").expect("Unable to create .env file");
        writeln!(file, "JIRA_BASE_URL={}", jira_base_url).expect("Unable to write to .env file");
        writeln!(file, "JIRA_EMAIL={}", email).expect("Unable to write to .env file");
        writeln!(file, "JIRA_API_TOKEN={}", api_token).expect("Unable to write to .env file");

        println!(".env file created successfully.");
    }
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn parse_description(description: &Value) -> String {
    match description.get("content") {
        Some(content) => content.as_array().unwrap_or(&vec![])
            .iter()
            .flat_map(parse_adf_node)
            .collect::<Vec<String>>()
            .join("\n"),
        None => "".to_string(),
    }
}

fn parse_adf_node(node: &Value) -> Vec<String> {
    if let Some(node_type) = node.get("type").and_then(|v| v.as_str()) {
        if node_type == "text" {
            node.get("text").and_then(|v| v.as_str()).map_or(vec![], |text| vec![text.to_string()])
        } else if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
            content.iter().flat_map(parse_adf_node).collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn extract_business_value_content(description: &str) -> String {
    let business_value_regex = regex::Regex::new(r"Business value\s*([\s\S]*?)\s*Customer value").unwrap();
    if let Some(caps) = business_value_regex.captures(description) {
        let content = caps.get(1).map_or("", |m| m.as_str()).trim();
        if content.contains("<>") {
            "No content found".to_string()
        } else {
            content.to_string()
        }
    } else {
        "No content found".to_string()
    }
}
