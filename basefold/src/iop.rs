use arith::{Field, FieldSerde};
use babybear::BabyBearx16;
// use p3_baby_bear::PackedBabyBearAVX512 as BabyBearx16;
use tree::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct BasefoldIOPPQuery<F: Field + FieldSerde> {
    // NOTE: the folding r's are in sumcheck verification, deriving from Fiat-Shamir.
    pub(crate) iopp_round_query: Vec<BasefoldIOPPQuerySingleRound<F>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BasefoldIOPPQuerySingleRound<F: Field + FieldSerde> {
    pub(crate) left: Path<F>,
    pub(crate) right: Path<F>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BasefoldVirtualIOPPQuery<F: Field + FieldSerde> {
    virtual_queries: Vec<BasefoldIOPPQuerySingleRound<F>>,
    iopp_query: BasefoldIOPPQuery<F>,
}

impl<F: Field + FieldSerde> BasefoldIOPPQuerySingleRound<F> {
    pub fn check_expected_codeword(
        &self,
        entry_index: usize,
        oracle_len: usize,
        entry: &F,
    ) -> bool {
        let side = &match entry_index & (oracle_len >> 1) {
            0 => self.left.leaf(),
            _ => self.right.leaf(),
        };

        side.data == *entry
    }
}

impl<F: Field + FieldSerde> BasefoldVirtualIOPPQuery<F> {
    #[inline]
    fn deteriorate_to_basefold_iopp_query(&self) -> BasefoldIOPPQuery<F> {
        // NOTE: the deterioration happens only when there is only one virtual query,
        // namely, using batch for one single polynomial.
        assert_eq!(self.virtual_queries.len(), 1);

        let mut iopp_round_query = self.virtual_queries.clone();
        iopp_round_query.extend_from_slice(&self.iopp_query.iopp_round_query);
        BasefoldIOPPQuery { iopp_round_query }
    }
}
