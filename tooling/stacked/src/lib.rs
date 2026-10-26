mod gh_argument;
mod gh_field;
mod gh_key;
mod repository;
mod stack_answer;

use std::env;

use gh_argument::GhArgument;
use gh_field::GhField;
use gh_key::GhKey;
use repository::Repository;
use stack_answer::StackAnswer;

use task::Failure;
use task::{Branch, Policy};
use task::{CommandLine, Program, Stdout};

#[derive(Debug)]
pub struct Stacked;

impl Stacked {
    const NUMBER: &str = "PULL_REQUEST";
    const REPOSITORY: &str = "GITHUB_REPOSITORY";
    const BASE: &str = "GITHUB_BASE_REF";
    const QUERY: &str = "query($owner: String!, $name: String!, $number: Int!) { repository(owner: $owner, name: $name) { pullRequest(number: $number) { baseRefName stack { number } } } }";
    const BASE_AND_STACK: &str = r#".data.repository.pullRequest | "\(.baseRefName) \(.stack.number // "none")""#;

    pub fn run(policy: &Policy) -> Result<(), Failure> {
        let number = Self::required(Self::NUMBER)?;
        let default = policy.branch(Branch::Default);
        let on_default = env::var(Self::BASE).ok().and_then(|base| base.parse().ok());
        let answer = match on_default.and_then(|base| StackAnswer::on_default_branch(base, default)) {
            Some(answer) => answer,
            None => Self::base_and_stack(&Self::required(Self::REPOSITORY)?.parse()?, &number)?.parse()?,
        };
        println!("{}", answer.verdict(&number, default)?);
        Ok(())
    }

    fn required(name: &'static str) -> Result<String, Failure> {
        env::var(name).map_err(|_| Failure::Unset(name))
    }

    fn base_and_stack(repository: &Repository, number: &str) -> Result<String, Failure> {
        let [owner, name] = repository.fields();
        CommandLine::new(Program::GitHub)
            .argument(GhArgument::Api)
            .argument(GhArgument::Graphql)
            .argument(GhArgument::Field)
            .argument(GhField::new(GhKey::Query, Self::QUERY))
            .argument(GhArgument::TypedField)
            .argument(owner)
            .argument(GhArgument::TypedField)
            .argument(name)
            .argument(GhArgument::TypedField)
            .argument(GhField::new(GhKey::Number, number))
            .argument(GhArgument::Jq)
            .argument(Self::BASE_AND_STACK)
            .captured::<Stdout>()
    }
}
