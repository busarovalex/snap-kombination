use crate::analyse::{AnalysisExecutor, SuppressWarnings};
use crate::deck::{CardIdentity, Deck};
use analyse::PermutationIterator;

mod analyse;
mod condition;
mod condition_count;
mod cost_efficiency;
mod deck;
mod input;
mod permutation_optimized;
mod permutation_simple;
mod placement;
#[cfg(test)]
mod tests;

const MAX_ID: u8 = 12;
const MAX_COST: u8 = 6;
const PERMUTATION_COUNT_WARNING_THRESHOLD: u64 = 10_000_000;

fn main() {
    let filename = match std::env::args().nth(1) {
        Some(name) => name,
        None => {
            println!("Please enter a filename");
            std::process::exit(1);
        }
    };
    let input = input::read_from_file(&filename);
    let analysis = match input::parse::<{ MAX_ID as usize }>(input) {
        Ok(result) => result,
        Err(err) => {
            println!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    match std::env::args().nth(2) {
        Some(simple) if simple.as_str() == "simple" => {
            execute::<crate::permutation_simple::AllPermutationsIterator<CardIdentity>>(analysis);
        }
        _ => {
            execute::<
                crate::permutation_optimized::DeckPermutationIterator<
                    CardIdentity,
                    { MAX_ID as usize },
                >,
            >(analysis);
        }
    }
}

fn execute<T: PermutationIterator<Deck<CardIdentity, { MAX_ID as usize }>>>(
    analysis: Vec<AnalysisExecutor<{ MAX_ID as usize }>>,
) {
    for result in analysis
        .into_iter()
        .map(|a| a.execute::<T>(SuppressWarnings::No))
        .flat_map(|result| match result {
            Ok(success) => success,
            Err((analysis, err)) => {
                println!("WARNING: {}", err);
                analysis.execute::<T>(SuppressWarnings::Yes).unwrap()
            }
        })
    {
        println!("{}", result);
    }
}
