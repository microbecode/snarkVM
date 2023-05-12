// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum MapID {
    Block(BlockMap),
    Deployment(DeploymentMap),
    Execution(ExecutionMap),
    Fee(FeeMap),
    Transaction(TransactionMap),
    Transition(TransitionMap),
    TransitionInput(TransitionInputMap),
    TransitionOutput(TransitionOutputMap),
    Program(ProgramMap),
    #[cfg(test)]
    Test(TestMap),
}

impl From<MapID> for u16 {
    fn from(id: MapID) -> u16 {
        match id {
            MapID::Block(id) => id as u16,
            MapID::Deployment(id) => id as u16,
            MapID::Execution(id) => id as u16,
            MapID::Fee(id) => id as u16,
            MapID::Transaction(id) => id as u16,
            MapID::Transition(id) => id as u16,
            MapID::TransitionInput(id) => id as u16,
            MapID::TransitionOutput(id) => id as u16,
            MapID::Program(id) => id as u16,
            #[cfg(test)]
            MapID::Test(id) => id as u16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum BlockMap {
    StateRoot = DataID::BlockStateRootMap as u16,
    ReverseStateRoot = DataID::BlockReverseStateRootMap as u16,
    ID = DataID::BlockIDMap as u16,
    ReverseID = DataID::BlockReverseIDMap as u16,
    Header = DataID::BlockHeaderMap as u16,
    Transactions = DataID::BlockTransactionsMap as u16,
    ConfirmedTransactions = DataID::BlockConfirmedTransactionsMap as u16,
    CoinbaseSolution = DataID::BlockCoinbaseSolutionMap as u16,
    CoinbasePuzzleCommitment = DataID::BlockCoinbasePuzzleCommitmentMap as u16,
    Signature = DataID::BlockSignatureMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DeploymentMap {
    ID = DataID::DeploymentIDMap as u16,
    Edition = DataID::DeploymentEditionMap as u16,
    ReverseID = DataID::DeploymentReverseIDMap as u16,
    Owner = DataID::DeploymentOwnerMap as u16,
    Program = DataID::DeploymentProgramMap as u16,
    VerifyingKey = DataID::DeploymentVerifyingKeyMap as u16,
    Certificate = DataID::DeploymentCertificateMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ExecutionMap {
    ID = DataID::ExecutionIDMap as u16,
    ReverseID = DataID::ExecutionReverseIDMap as u16,
    Inclusion = DataID::ExecutionInclusionMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum FeeMap {
    Fee = DataID::FeeFeeMap as u16,
    ReverseFee = DataID::FeeReverseFeeMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TransitionInputMap {
    ID = DataID::InputIDMap as u16,
    ReverseID = DataID::InputReverseIDMap as u16,
    Constant = DataID::InputConstantMap as u16,
    Public = DataID::InputPublicMap as u16,
    Private = DataID::InputPrivateMap as u16,
    Record = DataID::InputRecordMap as u16,
    RecordTag = DataID::InputRecordTagMap as u16,
    ExternalRecord = DataID::InputExternalRecordMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TransitionOutputMap {
    ID = DataID::OutputIDMap as u16,
    ReverseID = DataID::OutputReverseIDMap as u16,
    Constant = DataID::OutputConstantMap as u16,
    Public = DataID::OutputPublicMap as u16,
    Private = DataID::OutputPrivateMap as u16,
    Record = DataID::OutputRecordMap as u16,
    RecordNonce = DataID::OutputRecordNonceMap as u16,
    ExternalRecord = DataID::OutputExternalRecordMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TransactionMap {
    ID = DataID::TransactionIDMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TransitionMap {
    Locator = DataID::TransitionLocatorMap as u16,
    Finalize = DataID::TransitionFinalizeMap as u16,
    Proof = DataID::TransitionProofMap as u16,
    TPK = DataID::TransitionTPKMap as u16,
    ReverseTPK = DataID::TransitionReverseTPKMap as u16,
    TCM = DataID::TransitionTCMMap as u16,
    ReverseTCM = DataID::TransitionReverseTCMMap as u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ProgramMap {
    ProgramID = DataID::ProgramIDMap as u16,
    MappingID = DataID::MappingIDMap as u16,
    KeyValueID = DataID::KeyValueIDMap as u16,
    Key = DataID::KeyMap as u16,
    Value = DataID::ValueMap as u16,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TestMap {
    Test = DataID::Test as u16,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
enum DataID {
    // Block
    BlockStateRootMap,
    BlockReverseStateRootMap,
    BlockIDMap,
    BlockReverseIDMap,
    BlockHeaderMap,
    BlockTransactionsMap,
    BlockConfirmedTransactionsMap,
    BlockCoinbaseSolutionMap,
    BlockCoinbasePuzzleCommitmentMap,
    BlockSignatureMap,
    // Deployment
    DeploymentIDMap,
    DeploymentEditionMap,
    DeploymentReverseIDMap,
    DeploymentOwnerMap,
    DeploymentProgramMap,
    DeploymentVerifyingKeyMap,
    DeploymentCertificateMap,
    // Execution
    ExecutionIDMap,
    ExecutionReverseIDMap,
    ExecutionInclusionMap,
    // Fee
    FeeFeeMap,
    FeeReverseFeeMap,
    // Input
    InputIDMap,
    InputReverseIDMap,
    InputConstantMap,
    InputPublicMap,
    InputPrivateMap,
    InputRecordMap,
    InputRecordTagMap,
    InputExternalRecordMap,
    // Output
    OutputIDMap,
    OutputReverseIDMap,
    OutputConstantMap,
    OutputPublicMap,
    OutputPrivateMap,
    OutputRecordMap,
    OutputRecordNonceMap,
    OutputExternalRecordMap,
    // Transaction
    TransactionIDMap,
    // Transition
    TransitionLocatorMap,
    TransitionFinalizeMap,
    TransitionProofMap,
    TransitionTPKMap,
    TransitionReverseTPKMap,
    TransitionTCMMap,
    TransitionReverseTCMMap,
    // Program
    ProgramIDMap,
    MappingIDMap,
    KeyValueIDMap,
    KeyMap,
    ValueMap,

    // Testing
    #[cfg(test)]
    Test,
}
