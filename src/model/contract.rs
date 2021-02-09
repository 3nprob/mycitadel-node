// MyCitadel: node, wallet library & command-line tool
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@mycitadel.io>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use chrono::NaiveDateTime;
#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};
use std::collections::BTreeMap;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{OutPoint, Txid};
use lnpbp::client_side_validation::{CommitEncode, ConsensusCommit};
use strict_encoding::StrictEncode;
use wallet::{Psbt, TimeHeight};

use super::{ContractId, Operation, PaymentSlip, Policy, PolicyType, State};

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Getters, Clone, PartialEq, Debug, Display, StrictEncode, StrictDecode,
)]
#[display("{id}:{policy}")]
pub struct Contract {
    /// Unique contract id used to identify contract across different
    /// application instances. Created as a taproot-style bitcoin tagged
    /// hash out of strict-encoded wallet policy data: when policy
    /// changes contract id changes; if two contract on different devices have
    /// the same underlying policies they will have the same id.
    ///
    /// The id is kept pre-computed: the contract policy can't be changed after
    /// the creation, so there is no need to perform expensive commitment
    /// process each time we need contract id
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    id: ContractId,

    name: String,

    policy: Policy,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<chrono::DateTime<chrono::Utc>>")
    )]
    created_at: NaiveDateTime,

    #[cfg_attr(feature = "serde", serde(flatten))]
    data: ContractData,
}

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Getters, Clone, PartialEq, Debug, Default, StrictEncode, StrictDecode,
)]
pub struct ContractData {
    state: State,

    // TODO: Must be moved into rgb-node
    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")
    )]
    blinding_factors: BTreeMap<OutPoint, u64>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    sent_invoices: Vec<String>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    received_invoices: Vec<String>,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")
    )]
    paid_invoices: BTreeMap<String, PaymentSlip>,

    transactions: BTreeMap<Txid, Psbt>,

    /* #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, _)>>")
    )]*/
    // Due to some weird bug the variant above ^^^ is not working
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    operations: BTreeMap<TimeHeight, Operation>,
}

impl ConsensusCommit for Contract {
    type Commitment = ContractId;
}

impl CommitEncode for Contract {
    fn commit_encode<E: io::Write>(self, e: E) -> usize {
        self.policy
            .strict_encode(e)
            .expect("Memory encoders does not fail")
    }
}

impl Contract {
    pub fn with(policy: Policy, name: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed time service");
        Contract {
            id: policy.id(),
            name,
            policy,
            created_at: NaiveDateTime::from_timestamp(
                timestamp.as_secs() as i64,
                0,
            ),
            data: ContractData::default(),
        }
    }

    pub fn policy_type(&self) -> PolicyType {
        self.policy.policy_type()
    }
}