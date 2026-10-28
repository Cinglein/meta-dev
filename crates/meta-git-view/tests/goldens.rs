use meta_git_view::Ident;
use meta_golden::{GoldenFailure, Goldens};
use meta_run::GitArgv;

#[test]
fn every_recorded_ident_parses_to_its_golden_view() -> Result<(), GoldenFailure> {
    let recordings = Goldens::workspace().recordings()?;
    let idents = recordings.iter().filter(|recording| recording.argv().calls(GitArgv::Var));
    let viewed = idents.map(|ident| ident.viewed(&ident.out().parse::<Ident>()));
    assert_ne!(viewed.collect::<Result<Vec<()>, _>>()?, []);
    Ok(())
}
