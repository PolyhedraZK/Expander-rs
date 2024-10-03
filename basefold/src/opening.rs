use arith::ExtensionField;
use tree::Node;

use crate::iop::BasefoldIOPPQuery;

#[derive(Debug, Clone)]
pub struct BasefoldProof<ExtF: ExtensionField> {
    // sumcheck_transcript: SumcheckInstanceProof<ExtF>,
    pub(crate) iopp_oracles: Vec<Node>,
    pub(crate) iopp_last_oracle_message: Vec<ExtF>,
    pub(crate) iopp_queries: Vec<BasefoldIOPPQuery<ExtF::BaseField>>,
}
