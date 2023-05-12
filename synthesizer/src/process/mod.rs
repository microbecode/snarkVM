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

mod stack;
pub use stack::*;

mod authorize;
mod deploy;
mod evaluate;
mod execute;
mod execute_fee;
mod finalize;

#[cfg(test)]
mod tests;

use crate::{
    atomic_batch_scope,
    block::{Input, Transition},
    program::{Instruction, Program},
    snark::{ProvingKey, UniversalSRS, VerifyingKey},
    store::{FinalizeOperation, FinalizeStorage, FinalizeStore},
};
use console::{
    account::PrivateKey,
    network::prelude::*,
    program::{Identifier, Plaintext, ProgramID, Record, Request, Response, Value},
    types::{U16, U64},
};

use aleo_std::prelude::{finish, lap, timer};
use indexmap::IndexMap;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(feature = "aleo-cli")]
use colored::Colorize;

#[derive(Clone)]
pub struct Process<N: Network> {
    /// The universal SRS.
    universal_srs: Arc<UniversalSRS<N>>,
    /// The mapping of program IDs to stacks.
    stacks: IndexMap<ProgramID<N>, Stack<N>>,
}

impl<N: Network> Process<N> {
    /// Initializes a new process.
    #[inline]
    pub fn setup<A: circuit::Aleo<Network = N>, R: Rng + CryptoRng>(rng: &mut R) -> Result<Self> {
        let timer = timer!("Process:setup");

        // Initialize the process.
        let mut process = Self { universal_srs: Arc::new(UniversalSRS::load()?), stacks: IndexMap::new() };
        lap!(timer, "Initialize process");

        // Initialize the 'credits.aleo' program.
        let program = Program::credits()?;
        lap!(timer, "Load credits program");

        // Compute the 'credits.aleo' program stack.
        let stack = Stack::new(&process, &program)?;
        lap!(timer, "Initialize stack");

        // Synthesize the 'credits.aleo' circuit keys.
        for function_name in program.functions().keys() {
            stack.synthesize_key::<A, _>(function_name, rng)?;
            lap!(timer, "Synthesize circuit keys for {function_name}");
        }
        lap!(timer, "Synthesize credits program keys");

        // Add the 'credits.aleo' stack to the process.
        process.add_stack(stack);

        finish!(timer);
        // Return the process.
        Ok(process)
    }

    /// Adds a new program to the process.
    /// If you intend to `execute` the program, use `deploy` and `finalize_deployment` instead.
    #[inline]
    pub fn add_program(&mut self, program: &Program<N>) -> Result<()> {
        // Compute the program stack, and add it to the process.
        self.add_stack(Stack::new(self, program)?);
        Ok(())
    }

    /// Adds a new stack to the process.
    /// If you intend to `execute` the program, use `deploy` and `finalize_deployment` instead.
    #[inline]
    pub fn add_stack(&mut self, stack: Stack<N>) {
        // Add the stack to the process.
        self.stacks.insert(*stack.program_id(), stack);
    }
}

impl<N: Network> Process<N> {
    /// Initializes a new process.
    #[inline]
    pub fn load() -> Result<Self> {
        let timer = timer!("Process::load");

        // Initialize the process.
        let mut process = Self { universal_srs: Arc::new(UniversalSRS::load()?), stacks: IndexMap::new() };
        lap!(timer, "Initialize process");

        // Initialize the 'credits.aleo' program.
        let program = Program::credits()?;
        lap!(timer, "Load credits program");

        // Compute the 'credits.aleo' program stack.
        let stack = Stack::new(&process, &program)?;
        lap!(timer, "Initialize stack");

        // Synthesize the 'credits.aleo' circuit keys.
        for function_name in program.functions().keys() {
            // Load the proving key.
            let proving_key = N::get_credits_proving_key(function_name.to_string())?;
            stack.insert_proving_key(function_name, ProvingKey::new(proving_key.clone()))?;
            lap!(timer, "Load proving key for {function_name}");

            // Load the verifying key.
            let verifying_key = N::get_credits_verifying_key(function_name.to_string())?;
            stack.insert_verifying_key(function_name, VerifyingKey::new(verifying_key.clone()))?;
            lap!(timer, "Load verifying key for {function_name}");
        }
        lap!(timer, "Load circuit keys");

        // Initialize the inclusion proving key.
        let _ = N::inclusion_proving_key();
        lap!(timer, "Load inclusion proving key");

        // Initialize the inclusion verifying key.
        let _ = N::inclusion_verifying_key();
        lap!(timer, "Load inclusion verifying key");

        // Add the stack to the process.
        process.add_stack(stack);

        finish!(timer, "Process::load");
        // Return the process.
        Ok(process)
    }

    /// Initializes a new process with a cache of previously used keys. This version is suitable for tests
    /// (which often use nested loops that keep reusing those), as their deserialization is slow.
    #[cfg(test)]
    #[inline]
    pub fn load_with_cache(cache: &mut HashMap<String, (ProvingKey<N>, VerifyingKey<N>)>) -> Result<Self> {
        // Initialize the process.
        let mut process = Self { universal_srs: Arc::new(UniversalSRS::load()?), stacks: IndexMap::new() };

        // Initialize the 'credits.aleo' program.
        let program = Program::credits()?;
        // Compute the 'credits.aleo' program stack.
        let stack = Stack::new(&process, &program)?;

        // Synthesize the 'credits.aleo' circuit keys.
        for function_name in program.functions().keys() {
            // Cache the proving and verifying key.
            let (proving_key, verifying_key) = cache.entry(function_name.to_string()).or_insert_with(|| {
                // Load the proving key.
                let proving_key = N::get_credits_proving_key(function_name.to_string()).unwrap();
                // Load the verifying key.
                let verifying_key = N::get_credits_verifying_key(function_name.to_string()).unwrap();

                (ProvingKey::new(proving_key.clone()), VerifyingKey::new(verifying_key.clone()))
            });
            // Insert the proving and verifying key.
            stack.insert_proving_key(function_name, proving_key.clone())?;
            stack.insert_verifying_key(function_name, verifying_key.clone())?;
        }

        // Add the stack to the process.
        process.add_stack(stack);
        // Return the process.
        Ok(process)
    }

    /// Returns the universal SRS.
    #[inline]
    pub const fn universal_srs(&self) -> &Arc<UniversalSRS<N>> {
        &self.universal_srs
    }

    /// Returns `true` if the process contains the program with the given ID.
    #[inline]
    pub fn contains_program(&self, program_id: &ProgramID<N>) -> bool {
        self.stacks.contains_key(program_id)
    }

    /// Returns the stack for the given program ID.
    #[inline]
    pub fn get_stack(&self, program_id: impl TryInto<ProgramID<N>>) -> Result<&Stack<N>> {
        // Prepare the program ID.
        let program_id = program_id.try_into().map_err(|_| anyhow!("Invalid program ID"))?;
        // Retrieve the stack.
        let stack = self.stacks.get(&program_id).ok_or_else(|| anyhow!("Program '{program_id}' does not exist"))?;
        // Ensure the program ID matches.
        ensure!(stack.program_id() == &program_id, "Expected program '{}', found '{program_id}'", stack.program_id());
        // Return the stack.
        Ok(stack)
    }

    /// Returns the program for the given program ID.
    #[inline]
    pub fn get_program(&self, program_id: impl TryInto<ProgramID<N>>) -> Result<&Program<N>> {
        self.get_stack(program_id).map(Stack::program)
    }

    /// Returns the proving key for the given program ID and function name.
    #[inline]
    pub fn get_proving_key(
        &self,
        program_id: impl TryInto<ProgramID<N>>,
        function_name: impl TryInto<Identifier<N>>,
    ) -> Result<ProvingKey<N>> {
        // Prepare the function name.
        let function_name = function_name.try_into().map_err(|_| anyhow!("Invalid function name"))?;
        // Return the proving key.
        self.get_stack(program_id)?.get_proving_key(&function_name)
    }

    /// Returns the verifying key for the given program ID and function name.
    #[inline]
    pub fn get_verifying_key(
        &self,
        program_id: impl TryInto<ProgramID<N>>,
        function_name: impl TryInto<Identifier<N>>,
    ) -> Result<VerifyingKey<N>> {
        // Prepare the function name.
        let function_name = function_name.try_into().map_err(|_| anyhow!("Invalid function name"))?;
        // Return the verifying key.
        self.get_stack(program_id)?.get_verifying_key(&function_name)
    }

    /// Inserts the given proving key, for the given program ID and function name.
    #[inline]
    pub fn insert_proving_key(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
        proving_key: ProvingKey<N>,
    ) -> Result<()> {
        self.get_stack(program_id)?.insert_proving_key(function_name, proving_key)
    }

    /// Inserts the given verifying key, for the given program ID and function name.
    #[inline]
    pub fn insert_verifying_key(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
        verifying_key: VerifyingKey<N>,
    ) -> Result<()> {
        self.get_stack(program_id)?.insert_verifying_key(function_name, verifying_key)
    }

    /// Synthesizes the proving and verifying key for the given program ID and function name.
    #[inline]
    pub fn synthesize_key<A: circuit::Aleo<Network = N>, R: Rng + CryptoRng>(
        &self,
        program_id: &ProgramID<N>,
        function_name: &Identifier<N>,
        rng: &mut R,
    ) -> Result<()> {
        // Synthesize the proving and verifying key.
        self.get_stack(program_id)?.synthesize_key::<A, R>(function_name, rng)
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use crate::{Process, Program, Transition};
    use console::{account::PrivateKey, network::Testnet3, program::Identifier};

    use once_cell::sync::OnceCell;

    type CurrentNetwork = Testnet3;
    type CurrentAleo = circuit::network::AleoV0;

    pub(crate) fn sample_key() -> (Identifier<CurrentNetwork>, ProvingKey<CurrentNetwork>, VerifyingKey<CurrentNetwork>)
    {
        static INSTANCE: OnceCell<(
            Identifier<CurrentNetwork>,
            ProvingKey<CurrentNetwork>,
            VerifyingKey<CurrentNetwork>,
        )> = OnceCell::new();
        INSTANCE
            .get_or_init(|| {
                // Initialize a new program.
                let (string, program) = Program::<CurrentNetwork>::parse(
                    r"
program testing.aleo;

function compute:
    input r0 as u32.private;
    input r1 as u32.public;
    add r0 r1 into r2;
    output r2 as u32.public;",
                )
                .unwrap();
                assert!(string.is_empty(), "Parser did not consume all of the string: '{string}'");

                // Declare the function name.
                let function_name = Identifier::from_str("compute").unwrap();

                // Initialize the RNG.
                let rng = &mut TestRng::default();

                // Construct the process.
                let process = sample_process(&program);

                // Synthesize a proving and verifying key.
                process.synthesize_key::<CurrentAleo, _>(program.id(), &function_name, rng).unwrap();

                // Get the proving and verifying key.
                let proving_key = process.get_proving_key(program.id(), function_name).unwrap();
                let verifying_key = process.get_verifying_key(program.id(), function_name).unwrap();

                (function_name, proving_key, verifying_key)
            })
            .clone()
    }

    pub(crate) fn sample_execution() -> Execution<CurrentNetwork> {
        static INSTANCE: OnceCell<Execution<CurrentNetwork>> = OnceCell::new();
        INSTANCE
            .get_or_init(|| {
                // Initialize a new program.
                let (string, program) = Program::<CurrentNetwork>::parse(
                    r"
program testing.aleo;

function compute:
    input r0 as u32.private;
    input r1 as u32.public;
    add r0 r1 into r2;
    output r2 as u32.public;",
                )
                .unwrap();
                assert!(string.is_empty(), "Parser did not consume all of the string: '{string}'");

                // Declare the function name.
                let function_name = Identifier::from_str("compute").unwrap();

                // Initialize the RNG.
                let rng = &mut TestRng::default();
                // Initialize a new caller account.
                let caller_private_key = PrivateKey::<CurrentNetwork>::new(rng).unwrap();

                // Construct the process.
                let process = sample_process(&program);
                // Authorize the function call.
                let authorization = process
                    .authorize::<CurrentAleo, _>(
                        &caller_private_key,
                        program.id(),
                        function_name,
                        ["5u32", "10u32"].into_iter(),
                        rng,
                    )
                    .unwrap();
                assert_eq!(authorization.len(), 1);
                // Execute the request.
                let (_response, execution, _inclusion, _metrics) =
                    process.execute::<CurrentAleo, _>(authorization, rng).unwrap();
                assert_eq!(execution.len(), 1);
                // Return the execution.
                execution
            })
            .clone()
    }

    pub(crate) fn sample_transition() -> Transition<CurrentNetwork> {
        // Retrieve the execution.
        let mut execution = sample_execution();
        // Ensure the execution is not empty.
        assert!(!execution.is_empty());
        // Return the transition.
        execution.pop().unwrap()
    }

    /// Initializes a new process with the given program.
    pub(crate) fn sample_process(program: &Program<CurrentNetwork>) -> Process<CurrentNetwork> {
        // Construct a new process.
        let mut process = Process::load().unwrap();
        // Add the program to the process.
        process.add_program(program).unwrap();
        // Return the process.
        process
    }
}
