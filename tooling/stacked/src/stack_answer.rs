use std::str::FromStr;
use std::thread;
use std::time::Duration;

use task::BranchName;
use task::Failure;

pub struct StackAnswer {
    base: BranchName,
    stack: Option<String>,
}

impl StackAnswer {
    const NONE: &str = "none";

    pub fn on_default_branch(base: BranchName, default: &BranchName) -> Option<StackAnswer> {
        (base == *default).then_some(StackAnswer { base, stack: None })
    }

    pub fn settled(
        mut ask: impl FnMut() -> Result<StackAnswer, Failure>,
        default: &BranchName,
        waits: &[Duration],
    ) -> Result<StackAnswer, Failure> {
        let mut answer = ask()?;
        for wait in waits {
            if !answer.is_loose(default) {
                break;
            }
            thread::sleep(*wait);
            answer = ask()?;
        }
        Ok(answer)
    }

    fn is_loose(&self, default: &BranchName) -> bool {
        self.base != *default && self.stack.is_none()
    }

    pub fn verdict(&self, number: &str, default: &BranchName) -> Result<String, Failure> {
        match (&self.base, &self.stack) {
            (base, _) if base == default => Ok(format!("stacked: pull request #{number} targets {base}")),
            (base, Some(stack)) => Ok(format!("stacked: pull request #{number} targets {base} inside stack #{stack}")),
            (base, None) => Err(Failure::Refused(format!(
                "pull request #{number} targets {base} but belongs to no GitHub stack; link it with gh stack link <bottom> ... <top>"
            ))),
        }
    }
}

impl FromStr for StackAnswer {
    type Err = Failure;

    fn from_str(text: &str) -> Result<StackAnswer, Failure> {
        let (base, stack) = text.trim().split_once(' ').ok_or_else(|| Failure::GitHub(text.to_owned()))?;
        let Ok(base) = base.parse();
        Ok(StackAnswer { base, stack: (stack != Self::NONE).then(|| stack.to_owned()) })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use super::StackAnswer;
    use task::BranchName;
    const ANSWERS: [&str; 3] = ["trunk none", "feature 12", "feature none"];
    const WAITS: [Duration; 2] = [Duration::from_millis(1); 2];
    const STACKS_ON_ASK: [usize; 3] = [1, 3, 9];
    const OUTCOMES: [(usize, bool); 3] = [(1, true), (3, true), (3, false)];
    const BASES: [&str; 2] = ["trunk", "feature"];
    const NUMBER: &str = "7";

    #[test]
    fn a_pull_request_passes_on_the_default_branch_or_inside_a_stack_and_is_refused_loose_off_it() {
        let trunk: BranchName = BASES[0].parse().unwrap();
        let verdict = |answer: &str| answer.parse::<StackAnswer>().unwrap().verdict(NUMBER, &trunk);
        let [on_default, stacked, loose] = ANSWERS.map(verdict);
        let fine = on_default.is_ok() && stacked.unwrap().contains("stack #12") && loose.is_err();
        assert!(fine && "garbage".parse::<StackAnswer>().is_err());
    }

    #[test]
    fn a_loose_answer_is_asked_again_once_per_wait_and_a_stacked_one_ends_the_asking() {
        let trunk: BranchName = BASES[0].parse().unwrap();
        let asked = Cell::<usize>::default();
        let settle = |stacks_on: usize| {
            asked.set(0);
            let ask = || {
                asked.set(asked.get() + 1);
                ANSWERS[if asked.get() < stacks_on { 2 } else { 1 }].parse()
            };
            let answer = StackAnswer::settled(ask, &trunk, &WAITS).unwrap();
            (asked.get(), answer.verdict(NUMBER, &trunk).is_ok())
        };
        assert_eq!(STACKS_ON_ASK.map(settle), OUTCOMES);
    }

    #[test]
    fn only_the_default_branch_is_known_without_asking_github() {
        let trunk: BranchName = BASES[0].parse().unwrap();
        let [default, feature] = BASES.map(|base| StackAnswer::on_default_branch(base.parse().unwrap(), &trunk));
        assert!(default.is_some_and(|answer| answer.verdict(NUMBER, &trunk).is_ok()) && feature.is_none());
    }
}
