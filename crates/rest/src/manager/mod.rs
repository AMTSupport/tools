use crate::hudu::web::Hudu;
use crate::nable::web::NAble;

use clap::Subcommand;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use simplelog::{info, trace};

use std::collections::HashMap;

pub mod structs;

const NABLE_REGEX: &str = r"^(?P<name>[A-z\s0-9&\-\.']+)(\s?\((?P<managed>managed|managed\s-\spartial|billable|services|notify)\))?(\s?\((?P<type>server|server\sonly)\))?";

#[derive(Debug, Clone, Subcommand)]
pub enum ManagerCommands {
    Query,
}

impl ManagerCommands {
    pub async fn run<H: Hudu, N: NAble>(&self, hudu: H, nable: N) -> anyhow::Result<()> {
        match self {
            ManagerCommands::Query => query(hudu, nable).await?,
        }

        Ok(())
    }
}

// TODO :: Sort by length to get the best match.
async fn query<H: Hudu, N: NAble>(hudu: H, nable: N) -> anyhow::Result<()> {
    let hudu_companies = hudu.get_companies().await?;
    let nable_clients = nable.get_clients().await?;
    let re = Regex::new(NABLE_REGEX).unwrap();
    let mut matched = vec![];
    let mut not_matched = vec![];
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

    let mut matching_companies = hudu_companies.clone();
    for client in nable_clients.iter() {
        let name = client.identity_name.to_lowercase();
        let caps = re.captures(&name).unwrap();
        let name = caps["name"].to_lowercase();
        let name = name.trim();

        trace!("Trying to match client: {name}", name = name);

        let mut scores = HashMap::new();
        // let mut companies = companies.values();
        while let Some(company) = hudu_companies.values().next() {
            let company_name = company.name.to_lowercase();
            let score = match matcher.fuzzy_match(name, company_name.as_str()) {
                None => company
                    .nickname
                    .as_ref()
                    .and_then(|n| matcher.fuzzy_match(name, n.to_lowercase().trim())),
                opt => opt,
            };

            if let Some(score) = score.filter(|s| s > &0) {
                trace!("Matched against: {str}", str = company_name);
                trace!("Match score: {score}", score = score);

                scores.insert(score, company);
            }
        }

        let scores = scores.into_iter();
        if let Some((score, hudu_company)) = scores.max_by_key(|(score, _)| *score) {
            trace!(
                "Best match: {str} with score {score}",
                str = hudu_company.name
            );
            let _ = matching_companies.remove_entry(&hudu_company.id);
            matched.push((hudu_company, client));
        } else {
            not_matched.push(client);
        }
    }

    for client in not_matched {
        info!(
            "Unable to match client: {name} to a Hudu Company.",
            name = client.identity_name
        );
    }

    for (company, rmm) in matched {
        info!(
            "Matched client: {name} with RMM Client (Id: {id} | Name: {rmm_name})",
            name = company.name,
            id = rmm.identity_id,
            rmm_name = rmm.identity_name
        );
    }

    Ok(())
}
