# Jira Ticket Reporter

A command-line tool for aggregating business value from Jira tickets that follow a specific template format.

## Overview

This tool connects to Jira's API to pull ticket information and aggregate business value metrics. It's designed to work with tickets that follow a standardized template, making it easier to extract and analyze consistent data across multiple tickets.

## Prerequisites

- Rust (latest stable version)
- Jira API access credentials
- Tickets must follow the required template format (see below)

## Installation

1. Clone the repository
2. Run `cargo build --release`
3. The binary will be available in `target/release/fetch_jira_tickets`

## Configuration

Apon running this app for the first time your be taken through the setup, once this initial setup has been completed, these details will be stored. 

You can also create a `.env` file in the project root with your Jira credentials:

```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_EMAIL=your-email@company.com
JIRA_API_TOKEN=your-api-token
```

## Required Ticket Template

For the tool to work correctly, Jira tickets must follow this template structure:

### Description Template
```
h2. Business Value
{Insert quantifiable business value here}

h2. Success Metrics
- Metric 1: {description}
- Metric 2: {description}

h2. Implementation Details
{Technical implementation details}
```

### Required Fields
- Summary (Title)
- Description (Following the template above)
- Story Points
- Priority
- Status

## Usage

```bash
# Basic usage - fetch all tickets from a project
fetch_jira_tickets --project PROJECT_KEY

# Fetch tickets within a date range
fetch_jira_tickets --project PROJECT_KEY --from 2024-01-01 --to 2024-12-31

# Export results to CSV
fetch_jira_tickets --project PROJECT_KEY --export csv

# Show aggregated business value metrics
fetch_jira_tickets --project PROJECT_KEY --aggregate
```

## Output Format

The tool provides several output formats:

1. **Table View**: Default console output showing key ticket information
2. **CSV Export**: Detailed export of all ticket data
3. **Aggregated View**: Summary of business value metrics across all matching tickets

## Error Handling

- The tool validates ticket format and will flag any tickets that don't follow the template
- Missing required fields are reported with the ticket ID for easy reference
- API connection issues are clearly reported with troubleshooting steps

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License - See LICENSE file for details
