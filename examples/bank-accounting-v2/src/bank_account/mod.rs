use eventually::{
    aggregate::{Aggregate, Root as AggregateRoot},
    entity::{Entity, Identifiable, Named},
    message::Message,
    version::Version,
};
use eventually_macros::aggregate_root;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};

pub mod open_new_account;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    NewBankAccountOpened {
        iban: String,
        account_holder_id: String,
        currency: iso::Currency,
    },
    MoneyDepositedIntoBankAccount {
        amount: Decimal,
    },
    MoneyWithdrawnFromBankAccount {
        amount: Decimal,
    },
    BankAccountClosed,
}

impl Message for DomainEvent {
    fn name(&self) -> &'static str {
        use DomainEvent::*;

        match self {
            NewBankAccountOpened { .. } => "NewBankAccountOpened",
            MoneyDepositedIntoBankAccount { .. } => "MoneyDepositedIntoBankAccount",
            MoneyWithdrawnFromBankAccount { .. } => "MoneyWithdrawnFromBankAccount",
            BankAccountClosed => "BankAccountClosed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankAccount {
    iban: String,
    account_holder_id: String,
    currency: iso::Currency,
    balance: Decimal,
    is_closed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("bank account is already open")]
    AlreadyOpened,
    #[error("bank account is not opened yet")]
    NotOpenedYet,
}

impl Identifiable for BankAccount {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.iban
    }
}

impl Named for BankAccount {
    fn type_name() -> &'static str {
        "BankAccount"
    }
}

impl Aggregate for BankAccount {
    type Event = DomainEvent;
    type Error = ApplyError;

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        use DomainEvent::*;

        match (state, event) {
            (
                None,
                NewBankAccountOpened {
                    iban,
                    account_holder_id,
                    currency,
                },
            ) => Ok(BankAccount {
                iban,
                account_holder_id,
                currency,
                balance: Decimal::default(),
                is_closed: false,
            }),
            (None, _) => Err(ApplyError::NotOpenedYet),
            (Some(_), NewBankAccountOpened { .. }) => Err(ApplyError::AlreadyOpened),
            (Some(mut account), MoneyDepositedIntoBankAccount { amount }) => {
                account.balance += amount;
                Ok(account)
            }
            (Some(mut account), MoneyWithdrawnFromBankAccount { amount }) => {
                account.balance -= amount;
                Ok(account)
            }
            (Some(mut account), BankAccountClosed) => {
                account.is_closed = true;
                Ok(account)
            }
        }
    }
}

#[aggregate_root(BankAccount)]
#[derive(Debug, Clone)]
pub struct BankAccountRoot;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bank account iban provided is empty")]
    IbanIsEmpty,
    #[error("transaction amount specified is zero or negative, should be positive")]
    InvalidTransactionAmount,
    #[error("currency should be {expected}, received {received}")]
    CurrencyIsDifferent {
        received: iso::Currency,
        expected: iso::Currency,
    },
    #[error("account balance is insufficient to complete transaction")]
    InsufficientBalance,
    #[error("bank account is closed, action is no longer available")]
    Closed,
    #[error("failed to record domain event: {0}")]
    ApplyError(#[from] ApplyError),
}

impl BankAccountRoot {
    pub fn open(
        iban: String,
        account_holder_id: String,
        currency: iso::Currency,
    ) -> Result<Self, Error> {
        if iban.is_empty() {
            return Err(Error::IbanIsEmpty);
        }

        Ok(AggregateRoot::<BankAccount>::record_new(
            DomainEvent::NewBankAccountOpened {
                iban,
                account_holder_id,
                currency,
            }
            .into(),
        )
        .map(Self)?)
    }

    fn guard_money_are_as_expected(&self, amount: &Money<iso::Currency>) -> Result<(), Error> {
        if amount.currency() != &self.currency {
            return Err(Error::CurrencyIsDifferent {
                received: *amount.currency(),
                expected: self.currency,
            });
        }

        if amount.is_zero() || amount.is_negative() {
            return Err(Error::InvalidTransactionAmount);
        }

        Ok(())
    }

    fn guard_account_should_be_open(&self) -> Result<(), Error> {
        match self.is_closed {
            false => Ok(()),
            true => Err(Error::Closed),
        }
    }

    pub fn deposit_money(&mut self, amount: Money<iso::Currency>) -> Result<(), Error> {
        self.guard_money_are_as_expected(&amount)?;
        self.guard_account_should_be_open()?;

        Ok(self.record_that(
            DomainEvent::MoneyDepositedIntoBankAccount {
                amount: *amount.amount(),
            }
            .into(),
        )?)
    }

    pub fn withdrawn_money(&mut self, amount: Money<iso::Currency>) -> Result<(), Error> {
        self.guard_money_are_as_expected(&amount)?;
        self.guard_account_should_be_open()?;

        if amount.amount() > &self.balance {
            return Err(Error::InsufficientBalance);
        }

        Ok(self.record_that(
            DomainEvent::MoneyWithdrawnFromBankAccount {
                amount: *amount.amount(),
            }
            .into(),
        )?)
    }

    pub fn close(&mut self) -> Result<(), Error> {
        if self.is_closed {
            return Ok(());
        }

        Ok(self.record_that(DomainEvent::BankAccountClosed.into())?)
    }
}
