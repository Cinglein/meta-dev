pub struct Amber(Ginger, Kelp); //~ crate crate-graph: conductance 0.167 between 6 items [Amber Clove Fennel Ginger Kelp Oregano] and 7 items [Basil Dill Hyssop Juniper Lovage Mint Nutmeg]
pub struct Basil(Juniper, Nutmeg);
pub struct Clove(Kelp);
pub struct Dill(Fennel, Nutmeg);
pub struct Fennel(Ginger, Oregano);
pub struct Ginger(Hyssop);
pub struct Hyssop(Juniper, Nutmeg);
pub struct Juniper;
pub struct Kelp;
pub struct Lovage(Mint, Nutmeg);
pub struct Mint;
pub struct Nutmeg;
pub struct Oregano;
