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

pub mod confirmed;
pub use confirmed::*;

mod bytes;
mod merkle;
mod serialize;
mod string;

use crate::block::{Transaction, Transition};
use console::{
    network::prelude::*,
    program::{Ciphertext, Record, TransactionsPath, TransactionsTree, TRANSACTIONS_DEPTH},
    types::{Field, Group, U64},
};

use snarkvm_utilities::{cfg_find, cfg_find_map, cfg_values};

use indexmap::IndexMap;

#[cfg(not(feature = "serial"))]
use rayon::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Transactions<N: Network> {
    /// The transactions included in a block.
    transactions: IndexMap<N::TransactionID, ConfirmedTransaction<N>>,
}

impl<N: Network> Transactions<N> {
    /// Initializes from a given transactions list.
    pub fn from(transactions: &[ConfirmedTransaction<N>]) -> Self {
        Self::from_iter(transactions.iter())
    }
}

impl<N: Network> FromIterator<ConfirmedTransaction<N>> for Transactions<N> {
    /// Initializes from an iterator of transactions.
    fn from_iter<T: IntoIterator<Item = ConfirmedTransaction<N>>>(iter: T) -> Self {
        Self { transactions: iter.into_iter().map(|transaction| (transaction.id(), transaction)).collect() }
    }
}

impl<'a, N: Network> FromIterator<&'a ConfirmedTransaction<N>> for Transactions<N> {
    /// Initializes from an iterator of transactions.
    fn from_iter<T: IntoIterator<Item = &'a ConfirmedTransaction<N>>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().cloned())
    }
}

impl<N: Network> Transactions<N> {
    /// Returns the transaction for the given transaction ID.
    pub fn get(&self, transaction_id: &N::TransactionID) -> Option<&ConfirmedTransaction<N>> {
        self.transactions.get(transaction_id)
    }

    /// Returns 'true' if there are no accepted or rejected transactions.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Returns the number of confirmed transactions.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Returns the number of accepted transactions.
    pub fn num_accepted(&self) -> usize {
        cfg_values!(self.transactions).filter(|tx| tx.is_accepted()).count()
    }

    /// Returns the number of rejected transactions.
    pub fn num_rejected(&self) -> usize {
        cfg_values!(self.transactions).filter(|tx| tx.is_rejected()).count()
    }

    /// Returns the number of finalize operations.
    pub fn num_finalize(&self) -> usize {
        cfg_values!(self.transactions).map(|tx| tx.num_finalize()).sum()
    }
}

impl<N: Network> Transactions<N> {
    /// Returns `true` if the transactions contains the given transition ID.
    pub fn contains_transition(&self, transition_id: &N::TransitionID) -> bool {
        cfg_values!(self.transactions).any(|tx| tx.contains_transition(transition_id))
    }

    /// Returns `true` if the transactions contains the given serial number.
    pub fn contains_serial_number(&self, serial_number: &Field<N>) -> bool {
        cfg_values!(self.transactions).any(|tx| tx.contains_serial_number(serial_number))
    }

    /// Returns `true` if the transactions contains the given commitment.
    pub fn contains_commitment(&self, commitment: &Field<N>) -> bool {
        cfg_values!(self.transactions).any(|tx| tx.contains_commitment(commitment))
    }
}

impl<N: Network> Transactions<N> {
    /// Returns the transaction with the given transition ID, if it exists.
    pub fn find_transaction_for_transition_id(&self, transition_id: &N::TransitionID) -> Option<&Transaction<N>> {
        cfg_find!(self.transactions, transition_id, contains_transition).map(|tx| tx.transaction())
    }

    /// Returns the transaction with the given serial number, if it exists.
    pub fn find_transaction_for_serial_number(&self, serial_number: &Field<N>) -> Option<&Transaction<N>> {
        cfg_find!(self.transactions, serial_number, contains_serial_number).map(|tx| tx.transaction())
    }

    /// Returns the transaction with the given commitment, if it exists.
    pub fn find_transaction_for_commitment(&self, commitment: &Field<N>) -> Option<&Transaction<N>> {
        cfg_find!(self.transactions, commitment, contains_commitment).map(|tx| tx.transaction())
    }

    /// Returns the transition with the corresponding transition ID, if it exists.
    pub fn find_transition(&self, transition_id: &N::TransitionID) -> Option<&Transition<N>> {
        cfg_find_map!(self.transactions, transition_id, find_transition)
    }

    /// Returns the transition for the given serial number, if it exists.
    pub fn find_transition_for_serial_number(&self, serial_number: &Field<N>) -> Option<&Transition<N>> {
        cfg_find_map!(self.transactions, serial_number, find_transition_for_serial_number)
    }

    /// Returns the transition for the given commitment, if it exists.
    pub fn find_transition_for_commitment(&self, commitment: &Field<N>) -> Option<&Transition<N>> {
        cfg_find_map!(self.transactions, commitment, find_transition_for_commitment)
    }

    /// Returns the record with the corresponding commitment, if it exists.
    pub fn find_record(&self, commitment: &Field<N>) -> Option<&Record<N, Ciphertext<N>>> {
        cfg_find_map!(self.transactions, commitment, find_record)
    }
}

impl<N: Network> Transactions<N> {
    /// The maximum number of transactions allowed in a block.
    pub const MAX_TRANSACTIONS: usize = usize::pow(2, TRANSACTIONS_DEPTH as u32);

    /// Returns an iterator over all transactions, for all transactions in `self`.
    pub fn iter(&self) -> impl '_ + Iterator<Item = &ConfirmedTransaction<N>> {
        self.transactions.values()
    }

    /// Returns a parallel iterator over all transactions, for all transactions in `self`.
    #[cfg(not(feature = "serial"))]
    pub fn par_iter(&self) -> impl '_ + ParallelIterator<Item = &ConfirmedTransaction<N>> {
        self.transactions.par_values()
    }

    /// Returns an iterator over the transaction IDs, for all transactions in `self`.
    pub fn transaction_ids(&self) -> impl '_ + Iterator<Item = &N::TransactionID> {
        self.transactions.keys()
    }

    /// Returns an iterator over all transactions in `self` that are accepted deploy transactions.
    pub fn deployments(&self) -> impl '_ + Iterator<Item = &ConfirmedTransaction<N>> {
        self.iter().filter(|tx| tx.is_accepted() && tx.is_deploy())
    }

    /// Returns an iterator over all transactions in `self` that are accepted execute transactions.
    pub fn executions(&self) -> impl '_ + Iterator<Item = &ConfirmedTransaction<N>> {
        self.iter().filter(|tx| tx.is_accepted() && tx.is_execute())
    }

    /// Returns an iterator over all transitions.
    pub fn transitions(&self) -> impl '_ + Iterator<Item = &Transition<N>> {
        self.iter().flat_map(|tx| tx.transitions())
    }

    /// Returns an iterator over the transition IDs, for all transitions.
    pub fn transition_ids(&self) -> impl '_ + Iterator<Item = &N::TransitionID> {
        self.iter().flat_map(|tx| tx.transition_ids())
    }

    /// Returns an iterator over the transition public keys, for all transactions.
    pub fn transition_public_keys(&self) -> impl '_ + Iterator<Item = &Group<N>> {
        self.iter().flat_map(|tx| tx.transition_public_keys())
    }

    /// Returns an iterator over the tags, for all transition inputs that are records.
    pub fn tags(&self) -> impl '_ + Iterator<Item = &Field<N>> {
        self.iter().flat_map(|tx| tx.tags())
    }

    /// Returns an iterator over the serial numbers, for all transition inputs that are records.
    pub fn serial_numbers(&self) -> impl '_ + Iterator<Item = &Field<N>> {
        self.iter().flat_map(|tx| tx.serial_numbers())
    }

    /// Returns an iterator over the commitments, for all transition outputs that are records.
    pub fn commitments(&self) -> impl '_ + Iterator<Item = &Field<N>> {
        self.iter().flat_map(|tx| tx.commitments())
    }

    /// Returns an iterator over the records, for all transition outputs that are records.
    pub fn records(&self) -> impl '_ + Iterator<Item = (&Field<N>, &Record<N, Ciphertext<N>>)> {
        self.iter().flat_map(|tx| tx.records())
    }

    /// Returns an iterator over the nonces, for all transition outputs that are records.
    pub fn nonces(&self) -> impl '_ + Iterator<Item = &Group<N>> {
        self.iter().flat_map(|tx| tx.nonces())
    }

    /// Returns an iterator over the transaction fees, for all transactions.
    pub fn transaction_fees(&self) -> impl '_ + Iterator<Item = Result<U64<N>>> {
        self.iter().map(|tx| tx.fee())
    }
}

impl<N: Network> IntoIterator for Transactions<N> {
    type IntoIter = indexmap::map::IntoValues<N::TransactionID, Self::Item>;
    type Item = ConfirmedTransaction<N>;

    /// Returns a consuming iterator over all transactions, for all transactions in `self`.
    fn into_iter(self) -> Self::IntoIter {
        self.transactions.into_values()
    }
}

impl<N: Network> Transactions<N> {
    /// Returns a consuming iterator over the transaction IDs, for all transactions in `self`.
    pub fn into_transaction_ids(self) -> impl Iterator<Item = N::TransactionID> {
        self.transactions.into_keys()
    }

    /// Returns a consuming iterator over all transactions in `self` that are accepted deploy transactions.
    pub fn into_deployments(self) -> impl Iterator<Item = ConfirmedTransaction<N>> {
        self.into_iter().filter(|tx| tx.is_accepted() && tx.is_deploy())
    }

    /// Returns a consuming iterator over all transactions in `self` that are accepted execute transactions.
    pub fn into_executions(self) -> impl Iterator<Item = ConfirmedTransaction<N>> {
        self.into_iter().filter(|tx| tx.is_accepted() && tx.is_execute())
    }

    /// Returns a consuming iterator over all transitions.
    pub fn into_transitions(self) -> impl Iterator<Item = Transition<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_transitions())
    }

    /// Returns a consuming iterator over the transition IDs, for all transitions.
    pub fn into_transition_ids(self) -> impl Iterator<Item = N::TransitionID> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_transition_ids())
    }

    /// Returns a consuming iterator over the transition public keys, for all transactions.
    pub fn into_transition_public_keys(self) -> impl Iterator<Item = Group<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_transition_public_keys())
    }

    /// Returns a consuming iterator over the tags, for all transition inputs that are records.
    pub fn into_tags(self) -> impl Iterator<Item = Field<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_tags())
    }

    /// Returns a consuming iterator over the serial numbers, for all transition inputs that are records.
    pub fn into_serial_numbers(self) -> impl Iterator<Item = Field<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_serial_numbers())
    }

    /// Returns a consuming iterator over the commitments, for all transition outputs that are records.
    pub fn into_commitments(self) -> impl Iterator<Item = Field<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_commitments())
    }

    /// Returns a consuming iterator over the records, for all transition outputs that are records.
    pub fn into_records(self) -> impl Iterator<Item = (Field<N>, Record<N, Ciphertext<N>>)> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_records())
    }

    /// Returns a consuming iterator over the nonces, for all transition outputs that are records.
    pub fn into_nonces(self) -> impl Iterator<Item = Group<N>> {
        self.into_iter().flat_map(|tx| tx.into_transaction().into_nonces())
    }
}
