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

mod bytes;
mod serialize;
mod string;

use crate::{
    block::Transaction,
    process::{Deployment, Execution},
    store::FinalizeOperation,
};
use console::network::prelude::*;

pub type NumFinalizeSize = u16;

/// A safety wrapper around a rejected type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected<T: Clone + Debug + PartialEq + Eq + ToBytes>(pub T);

impl<T: Clone + Debug + PartialEq + Eq + ToBytes> Deref for Rejected<T> {
    type Target = T;

    /// Returns a reference to the rejected type.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The confirmed transaction.
#[derive(Clone, PartialEq, Eq)]
pub enum ConfirmedTransaction<N: Network> {
    /// The accepted deploy transaction is composed of `(index, deploy_transaction, finalize_operations)`.
    AcceptedDeploy(u32, Transaction<N>, Vec<FinalizeOperation<N>>),
    /// The accepted execute transaction is composed of `(index, execute_transaction, finalize_operations)`.
    AcceptedExecute(u32, Transaction<N>, Vec<FinalizeOperation<N>>),
    /// The rejected deploy transaction is composed of `(index, fee_transaction, rejected_deployment)`.
    RejectedDeploy(u32, Transaction<N>, Box<Rejected<Deployment<N>>>),
    /// The rejected execute transaction is composed of `(index, fee_transaction, rejected_execution)`.
    RejectedExecute(u32, Transaction<N>, Rejected<Execution<N>>),
}

impl<N: Network> ConfirmedTransaction<N> {
    /// Returns a new instance of an accepted deploy transaction.
    pub fn accepted_deploy(
        index: u32,
        transaction: Transaction<N>,
        finalize_operations: Vec<FinalizeOperation<N>>,
    ) -> Result<Self> {
        // Retrieve the program from the deployment, and ensure the transaction is a deploy transaction.
        let program = match &transaction {
            Transaction::Deploy(_, _, deployment, _) => deployment.program(),
            Transaction::Execute(..) | Transaction::Fee(..) => {
                bail!("Transaction '{}' is not a deploy transaction", transaction.id())
            }
        };
        // Ensure the number of program mappings matches the number of finalize operations.
        if program.mappings().len() != finalize_operations.len() {
            bail!(
                "The number of program mappings ({}) does not match the nubmer of finalize operations ({})",
                program.mappings().len(),
                finalize_operations.len()
            )
        }
        // Ensure the finalize operations contain the correct types.
        for operation in finalize_operations.iter() {
            // Ensure the finalize operation is an initialize mapping.
            if !matches!(operation, FinalizeOperation::InitializeMapping(..)) {
                bail!("Transaction '{}' (deploy) contains an invalid finalize operation type", transaction.id())
            }
        }
        // Return the accepted deploy transaction.
        Ok(ConfirmedTransaction::AcceptedDeploy(index, transaction, finalize_operations))
    }

    /// Returns a new instance of an accepted execute transaction.
    pub fn accepted_execute(
        index: u32,
        transaction: Transaction<N>,
        finalize_operations: Vec<FinalizeOperation<N>>,
    ) -> Result<Self> {
        // Ensure the finalize operations contain the correct types.
        for operation in finalize_operations.iter() {
            // Ensure the finalize operation is an insert or update key-value operation.
            match operation {
                FinalizeOperation::InsertKeyValue(..) | FinalizeOperation::UpdateKeyValue(..) => (),
                FinalizeOperation::InitializeMapping(..)
                | FinalizeOperation::RemoveMapping(..)
                | FinalizeOperation::RemoveKeyValue(..) => {
                    bail!("Transaction '{}' (execute) contains an invalid finalize operation type", transaction.id())
                }
            }
        }
        // Ensure the transaction is an execute transaction.
        match transaction.is_execute() {
            true => Ok(ConfirmedTransaction::AcceptedExecute(index, transaction, finalize_operations)),
            false => bail!("Transaction '{}' is not an execute transaction", transaction.id()),
        }
    }

    /// Returns a new instance of a rejected deploy transaction.
    pub fn rejected_deploy(
        index: u32,
        transaction: Transaction<N>,
        rejected_deployment: Deployment<N>,
    ) -> Result<Self> {
        // Ensure the transaction is a fee transaction.
        match transaction.is_fee() {
            true => {
                Ok(ConfirmedTransaction::RejectedDeploy(index, transaction, Box::new(Rejected(rejected_deployment))))
            }
            false => bail!("Transaction '{}' is not a fee transaction", transaction.id()),
        }
    }

    /// Returns a new instance of a rejected execute transaction.
    pub fn rejected_execute(index: u32, transaction: Transaction<N>, rejected_execution: Execution<N>) -> Result<Self> {
        // Ensure the transaction is a fee transaction.
        match transaction.is_fee() {
            true => Ok(ConfirmedTransaction::RejectedExecute(index, transaction, Rejected(rejected_execution))),
            false => bail!("Transaction '{}' is not a fee transaction", transaction.id()),
        }
    }
}

impl<N: Network> ConfirmedTransaction<N> {
    /// Returns 'true' if the confirmed transaction is accepted.
    pub fn is_accepted(&self) -> bool {
        match self {
            ConfirmedTransaction::AcceptedDeploy(..) | ConfirmedTransaction::AcceptedExecute(..) => true,
            ConfirmedTransaction::RejectedDeploy(..) | ConfirmedTransaction::RejectedExecute(..) => false,
        }
    }

    /// Returns 'true' if the confirmed transaction is rejected.
    pub fn is_rejected(&self) -> bool {
        !self.is_accepted()
    }
}

impl<N: Network> ConfirmedTransaction<N> {
    /// Returns the confirmed transaction index.
    pub fn index(&self) -> u32 {
        match self {
            ConfirmedTransaction::AcceptedDeploy(index, ..) => *index,
            ConfirmedTransaction::AcceptedExecute(index, ..) => *index,
            ConfirmedTransaction::RejectedDeploy(index, ..) => *index,
            ConfirmedTransaction::RejectedExecute(index, ..) => *index,
        }
    }

    /// Returns the transaction.
    pub fn transaction(&self) -> &Transaction<N> {
        match self {
            ConfirmedTransaction::AcceptedDeploy(_, transaction, _) => transaction,
            ConfirmedTransaction::AcceptedExecute(_, transaction, _) => transaction,
            ConfirmedTransaction::RejectedDeploy(_, transaction, _) => transaction,
            ConfirmedTransaction::RejectedExecute(_, transaction, _) => transaction,
        }
    }

    /// Returns the transaction.
    pub fn into_transaction(self) -> Transaction<N> {
        match self {
            ConfirmedTransaction::AcceptedDeploy(_, transaction, _) => transaction,
            ConfirmedTransaction::AcceptedExecute(_, transaction, _) => transaction,
            ConfirmedTransaction::RejectedDeploy(_, transaction, _) => transaction,
            ConfirmedTransaction::RejectedExecute(_, transaction, _) => transaction,
        }
    }

    /// Returns the number of finalize operations.
    pub fn num_finalize(&self) -> usize {
        match self {
            ConfirmedTransaction::AcceptedDeploy(_, _, finalize)
            | ConfirmedTransaction::AcceptedExecute(_, _, finalize) => finalize.len(),
            ConfirmedTransaction::RejectedDeploy(..) | ConfirmedTransaction::RejectedExecute(..) => 0,
        }
    }
}

impl<N: Network> Deref for ConfirmedTransaction<N> {
    type Target = Transaction<N>;

    /// Returns a reference to the valid transaction.
    fn deref(&self) -> &Self::Target {
        self.transaction()
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use console::network::Testnet3;

    type CurrentNetwork = Testnet3;

    /// Samples an accepted deploy transaction at the given index.
    pub(crate) fn sample_accepted_deploy(index: u32, rng: &mut TestRng) -> ConfirmedTransaction<CurrentNetwork> {
        // Sample a deploy transaction.
        let tx = crate::vm::test_helpers::sample_deployment_transaction(rng);
        // Return the confirmed transaction.
        ConfirmedTransaction::accepted_deploy(index, tx, vec![FinalizeOperation::InitializeMapping(Uniform::rand(rng))])
            .unwrap()
    }

    /// Samples an accepted execute transaction at the given index.
    pub(crate) fn sample_accepted_execute(index: u32, rng: &mut TestRng) -> ConfirmedTransaction<CurrentNetwork> {
        // Sample an execute transaction.
        let tx = crate::vm::test_helpers::sample_execution_transaction_with_fee(rng);
        // Return the confirmed transaction.
        ConfirmedTransaction::accepted_execute(index, tx, vec![]).unwrap()
    }

    /// Samples a rejected deploy transaction at the given index.
    pub(crate) fn sample_rejected_deploy(index: u32, rng: &mut TestRng) -> ConfirmedTransaction<CurrentNetwork> {
        // Sample a fee transaction.
        let fee_transaction = crate::vm::test_helpers::sample_fee_transaction(rng);

        // Extract the deployment.
        let deploy = match crate::vm::test_helpers::sample_deployment_transaction(rng) {
            Transaction::Deploy(_, _, deploy, _) => (*deploy).clone(),
            _ => unreachable!(),
        };

        // Return the confirmed transaction.
        ConfirmedTransaction::rejected_deploy(index, fee_transaction, deploy).unwrap()
    }

    /// Samples a rejected execute transaction at the given index.
    pub(crate) fn sample_rejected_execute(index: u32, rng: &mut TestRng) -> ConfirmedTransaction<CurrentNetwork> {
        // Sample a fee transaction.
        let fee_transaction = crate::vm::test_helpers::sample_fee_transaction(rng);

        // Extract the execution.
        let execute = match crate::vm::test_helpers::sample_execution_transaction_with_fee(rng) {
            Transaction::Execute(_, execute, _) => execute,
            _ => unreachable!(),
        };

        // Return the confirmed transaction.
        ConfirmedTransaction::rejected_execute(index, fee_transaction, execute).unwrap()
    }

    /// Sample a list of randomly confirmed transactions.
    pub(crate) fn sample_confirmed_transactions() -> Vec<ConfirmedTransaction<CurrentNetwork>> {
        let rng = &mut TestRng::default();

        vec![
            sample_accepted_deploy(0, rng),
            sample_accepted_execute(1, rng),
            sample_rejected_deploy(2, rng),
            sample_rejected_execute(3, rng),
            sample_accepted_deploy(Uniform::rand(rng), rng),
            sample_accepted_execute(Uniform::rand(rng), rng),
            sample_rejected_deploy(Uniform::rand(rng), rng),
            sample_rejected_execute(Uniform::rand(rng), rng),
        ]
    }
}
