use vm_core::StackInputs;

use super::{
    utils, AdviceInputs, InputNotes, OutputNotes, TransactionId, TransactionInputs,
    TransactionOutputs, TransactionScript, TransactionWitness,
};
use crate::{
    accounts::{Account, AccountDelta, AccountId, AccountStub},
    assembly::Program,
    BlockHeader, ExecutedTransactionError,
};

// EXECUTED TRANSACTION
// ================================================================================================

/// Describes the result of executing a transaction program.
///
/// Executed transaction contains the following data:
/// -
#[derive(Debug, Clone)]
pub struct ExecutedTransaction {
    id: TransactionId,
    program: Program,
    tx_inputs: TransactionInputs,
    tx_outputs: TransactionOutputs,
    account_delta: AccountDelta,
    tx_script: Option<TransactionScript>,
    advice_witness: AdviceInputs,
}

impl ExecutedTransaction {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Returns a new [ExecutedTransaction] instantiated from the provided data.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Input and output account IDs are not the same.
    /// - For a new account, account seed is not provided or the provided seed is invalid.
    /// - For an existing account, account seed was provided.
    pub fn new(
        program: Program,
        tx_inputs: TransactionInputs,
        tx_outputs: TransactionOutputs,
        account_delta: AccountDelta,
        tx_script: Option<TransactionScript>,
        advice_witness: AdviceInputs,
    ) -> Result<Self, ExecutedTransactionError> {
        // make sure account IDs are consistent across transaction inputs and outputs
        if tx_inputs.account.id() != tx_inputs.account.id() {
            return Err(ExecutedTransactionError::InconsistentAccountId {
                input_id: tx_inputs.account.id(),
                output_id: tx_outputs.account.id(),
            });
        }

        // if this transaction was executed against a new account, validate the account seed
        tx_inputs
            .validate_new_account_seed()
            .map_err(ExecutedTransactionError::AccountSeedError)?;

        // build transaction ID
        let id = TransactionId::new(
            tx_inputs.account.hash(),
            tx_outputs.account.hash(),
            tx_inputs.input_notes.commitment(),
            tx_outputs.output_notes.commitment(),
        );

        Ok(Self {
            id,
            program,
            tx_inputs,
            tx_outputs,
            account_delta,
            tx_script,
            advice_witness,
        })
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a unique identifier of this transaction.
    pub fn id(&self) -> TransactionId {
        self.id
    }

    /// Returns a reference the program defining this transaction.
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Returns the ID of the account against which this transaction was executed.
    pub fn account_id(&self) -> AccountId {
        self.initial_account().id()
    }

    /// Returns the description of the account before the transaction was executed.
    pub fn initial_account(&self) -> &Account {
        &self.tx_inputs.account
    }

    /// Returns description of the account after the transaction was executed.
    pub fn final_account(&self) -> &AccountStub {
        &self.tx_outputs.account
    }

    /// Returns the notes consumed in this transaction.
    pub fn input_notes(&self) -> &InputNotes {
        &self.tx_inputs.input_notes
    }

    /// Returns the notes created in this transaction.
    pub fn output_notes(&self) -> &OutputNotes {
        &self.tx_outputs.output_notes
    }

    /// Returns a reference to the transaction script.
    pub fn tx_script(&self) -> Option<&TransactionScript> {
        self.tx_script.as_ref()
    }

    /// Returns the block header for the block against which the transaction was executed.
    pub fn block_header(&self) -> &BlockHeader {
        &self.tx_inputs.block_header
    }

    /// Returns a description of changes between the initial and final account states.
    pub fn account_delta(&self) -> &AccountDelta {
        &self.account_delta
    }

    /// Returns the stack inputs required when executing the transaction.
    pub fn stack_inputs(&self) -> StackInputs {
        utils::generate_stack_inputs(&self.tx_inputs)
    }

    /// Returns the advice inputs required when executing the transaction.
    pub fn advice_provider_inputs(&self) -> AdviceInputs {
        utils::generate_advice_provider_inputs(&self.tx_inputs, &self.tx_script)
    }

    // CONVERSIONS
    // --------------------------------------------------------------------------------------------

    /// Converts this transaction into a [TransactionWitness].
    pub fn into_witness(self) -> TransactionWitness {
        TransactionWitness::new(
            self.initial_account().id(),
            self.initial_account().hash(),
            self.block_header().hash(),
            self.input_notes().commitment(),
            self.tx_script().map(|s| *s.hash()),
            self.program,
            self.advice_witness,
        )
    }
}
