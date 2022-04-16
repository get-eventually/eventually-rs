use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
};

use eventually::{aggregate, aggregate::Root as AggregateRoot, event, message};
use rust_decimal::Decimal;

pub type TransactionId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub beneficiary_account_id: BankAccountId,
    pub amount: Decimal,
}

pub type BankAccountHolderId = String;
pub type BankAccountId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BankAccountEvent {
    WasOpened {
        id: BankAccountId,
        account_holder_id: BankAccountHolderId,
        initial_balance: Option<Decimal>,
    },
    DepositWasRecorded {
        amount: Decimal,
    },
    TransferWasSent {
        transaction: Transaction,
        message: Option<String>,
    },
    TransferWasReceived {
        transaction: Transaction,
        message: Option<String>,
    },
    TransferWasDeclined {
        transaction_id: TransactionId,
        reason: Option<String>, // TODO: maybe turn into an enum?
    },
    TransferWasConfirmed {
        transaction_id: TransactionId,
    },
    WasClosed,
    WasReopened {
        reopening_balance: Option<Decimal>,
    },
}

impl message::Message for BankAccountEvent {
    fn name(&self) -> &'static str {
        match self {
            BankAccountEvent::WasOpened { .. } => "BankAccountWasOpened",
            BankAccountEvent::DepositWasRecorded { .. } => "BankAccountDepositWasRecorded",
            BankAccountEvent::TransferWasReceived { .. } => "BankAccountTransferWasReceived",
            BankAccountEvent::TransferWasSent { .. } => "BankAccountTransferWasSent",
            BankAccountEvent::TransferWasDeclined { .. } => "BankAccountTransferWasDeclined",
            BankAccountEvent::TransferWasConfirmed { .. } => "BankAccountTransferWasConfirmed",
            BankAccountEvent::WasClosed => "BankAccountWasClosed",
            BankAccountEvent::WasReopened { .. } => "BankAccountWasReopened",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BankAccountError {
    #[error("bank account has not been opened yet")]
    NotOpenedYet,
    #[error("bank account has already been opened")]
    AlreadyOpened,
    #[error("empty id provided for the new bank account")]
    EmptyAccountId,
    #[error("empty account holder id provided for the new bank account")]
    EmptyAccountHolderId,
    #[error("a deposit was attempted with negative import")]
    NegativeDepositAttempted,
    #[error("no money to deposit has been specified")]
    NoMoneyDeposited,
    #[error("transfer could not be sent due to insufficient funds")]
    InsufficientFunds,
    #[error("transfer transaction was destined to a different recipient: {0}")]
    WrongTransactionRecipient(BankAccountId),
    #[error("the account is closed")]
    Closed,
    #[error("bank account has already been closed")]
    AlreadyClosed,
}

#[derive(Debug, Clone)]
pub struct BankAccount {
    id: BankAccountId,
    current_balance: Decimal,
    pending_transactions: HashMap<TransactionId, Transaction>,
    is_closed: bool,
}

impl aggregate::Aggregate for BankAccount {
    type Id = BankAccountId;
    type Event = BankAccountEvent;
    type Error = BankAccountError;

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match state {
            None => match event {
                BankAccountEvent::WasOpened {
                    id,
                    initial_balance,
                    ..
                } => Ok(BankAccount {
                    id,
                    current_balance: initial_balance.unwrap_or_default(),
                    pending_transactions: HashMap::default(),
                    is_closed: false,
                }),
                _ => Err(BankAccountError::NotOpenedYet),
            },
            Some(mut account) => match event {
                BankAccountEvent::DepositWasRecorded { amount } => {
                    account.current_balance += amount;
                    Ok(account)
                }
                BankAccountEvent::TransferWasReceived { transaction, .. } => {
                    account.current_balance += transaction.amount;
                    Ok(account)
                }
                BankAccountEvent::TransferWasSent { transaction, .. } => {
                    account.current_balance -= transaction.amount;
                    account
                        .pending_transactions
                        .insert(transaction.id.clone(), transaction);
                    Ok(account)
                }
                BankAccountEvent::TransferWasConfirmed { transaction_id } => {
                    account.pending_transactions.remove(&transaction_id);
                    Ok(account)
                }
                BankAccountEvent::TransferWasDeclined { transaction_id, .. } => {
                    if let Some(transaction) = account.pending_transactions.remove(&transaction_id)
                    {
                        account.current_balance += transaction.amount;
                    }

                    Ok(account)
                }
                BankAccountEvent::WasClosed => {
                    account.is_closed = true;
                    account.current_balance = Decimal::default();
                    Ok(account)
                }
                BankAccountEvent::WasReopened { reopening_balance } => {
                    account.is_closed = false;
                    account.current_balance = reopening_balance.unwrap_or_default();
                    Ok(account)
                }
                BankAccountEvent::WasOpened { .. } => Err(BankAccountError::AlreadyOpened),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankAccountRoot(aggregate::Context<BankAccount>);

impl AggregateRoot<BankAccount> for BankAccountRoot {}

impl From<aggregate::Context<BankAccount>> for BankAccountRoot {
    fn from(context: aggregate::Context<BankAccount>) -> Self {
        Self(context)
    }
}

impl Borrow<aggregate::Context<BankAccount>> for BankAccountRoot {
    fn borrow(&self) -> &aggregate::Context<BankAccount> {
        &self.0
    }
}

impl BorrowMut<aggregate::Context<BankAccount>> for BankAccountRoot {
    fn borrow_mut(&mut self) -> &mut aggregate::Context<BankAccount> {
        &mut self.0
    }
}

impl BankAccountRoot {
    pub fn open(
        id: BankAccountId,
        account_holder_id: BankAccountHolderId,
        opening_balance: Option<Decimal>,
    ) -> Result<Self, BankAccountError> {
        if id.is_empty() {
            return Err(BankAccountError::EmptyAccountId);
        }

        if account_holder_id.is_empty() {
            return Err(BankAccountError::EmptyAccountHolderId);
        }

        Ok(BankAccountRoot::record_new(event::Envelope::from(
            BankAccountEvent::WasOpened {
                id,
                account_holder_id,
                initial_balance: opening_balance,
            },
        ))?)
    }

    pub fn deposit(&mut self, money: Decimal) -> Result<(), BankAccountError> {
        if self.state().is_closed {
            return Err(BankAccountError::Closed);
        }

        if money.is_sign_negative() {
            return Err(BankAccountError::NegativeDepositAttempted);
        }

        if money.is_zero() {
            return Err(BankAccountError::NoMoneyDeposited);
        }

        self.record_that(event::Envelope::from(
            BankAccountEvent::DepositWasRecorded { amount: money },
        ))
    }

    pub fn send_transfer(
        &mut self,
        mut transaction: Transaction,
        message: Option<String>,
    ) -> Result<(), BankAccountError> {
        if self.state().is_closed {
            return Err(BankAccountError::Closed);
        }

        // NOTE: transaction amounts should be positive, so they can be subtracted
        // when applied to a Bank Account.
        if transaction.amount.is_sign_negative() {
            transaction.amount.set_sign_positive(true);
        }

        if self.state().current_balance < transaction.amount {
            return Err(BankAccountError::InsufficientFunds);
        }

        let transaction_already_pending = self
            .state()
            .pending_transactions
            .get(&transaction.id)
            .is_some();

        if transaction_already_pending {
            return Ok(());
        }

        self.record_that(event::Envelope::from(BankAccountEvent::TransferWasSent {
            message,
            transaction,
        }))
    }

    pub fn receive_transfer(
        &mut self,
        transaction: Transaction,
        message: Option<String>,
    ) -> Result<(), BankAccountError> {
        if self.state().is_closed {
            return Err(BankAccountError::Closed);
        }

        if self.state().id != transaction.beneficiary_account_id {
            return Err(BankAccountError::WrongTransactionRecipient(
                transaction.beneficiary_account_id,
            ));
        }

        self.record_that(event::Envelope::from(
            BankAccountEvent::TransferWasReceived {
                transaction,
                message,
            },
        ))
    }

    pub fn record_transfer_success(
        &mut self,
        transaction_id: TransactionId,
    ) -> Result<(), BankAccountError> {
        let is_transaction_recorded = self
            .state()
            .pending_transactions
            .get(&transaction_id)
            .is_some();

        if !is_transaction_recorded {
            // TODO: return error
        }

        self.record_that(event::Envelope::from(
            BankAccountEvent::TransferWasConfirmed { transaction_id },
        ))
    }

    pub fn close(&mut self) -> Result<(), BankAccountError> {
        if self.state().is_closed {
            return Err(BankAccountError::AlreadyClosed);
        }

        self.record_that(event::Envelope::from(BankAccountEvent::WasClosed))
    }

    pub fn reopen(&mut self, reopening_balance: Option<Decimal>) -> Result<(), BankAccountError> {
        if !self.state().is_closed {
            return Err(BankAccountError::AlreadyOpened);
        }

        self.record_that(event::Envelope::from(BankAccountEvent::WasReopened {
            reopening_balance,
        }))
    }
}

pub type BankAccountRepository<S> =
    aggregate::EventSourcedRepository<BankAccount, BankAccountRoot, S>;
