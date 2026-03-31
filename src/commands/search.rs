use anyhow::Result;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use crate::job::Job;
use crate::output;
use super::common::gather_all_jobs;

pub fn run(query: &str, json: bool) -> Result<()> {
    let (jobs, _sidecar) = gather_all_jobs()?;

    // Fuzzy match
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(query, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy, false);

    let matched: Vec<Job> = jobs
        .into_iter()
        .filter(|job| {
            let searchable = format!(
                "{} {} {} {}",
                job.id.as_deref().unwrap_or(""),
                job.schedule_human,
                job.command,
                job.source,
            );
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(&searchable, &mut buf);
            pattern.score(haystack, &mut matcher).is_some()
        })
        .collect();

    if json {
        println!("{}", output::format_json(&matched)?);
    } else if matched.is_empty() {
        println!("No jobs matching '{query}'");
    } else {
        println!("{}", output::format_table(&matched));
    }

    Ok(())
}
