use std::error::Error as StdError;

use anyhow::anyhow;
use async_trait::async_trait;
use eventually::{aggregate, command, entity, message::Message};
use rusty_money::iso;

use crate::bank_account::{BankAccount, BankAccountRoot, Error as BankAccountError};

pub struct Command {
    pub iban: String,
    pub account_holder_id: String,
    pub currency_code: String,
}

impl Message for Command {
    fn name(&self) -> &'static str {
        "OpenNewBankAccount"
    }
}

pub struct Handler<R>
where
    R: entity::Repository<BankAccountRoot>,
    <R as entity::Saver<BankAccountRoot>>::Error: StdError + Send + Sync + 'static,
{
    pub bank_account_repository: R,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid currency code, should follow ISO-4217 standard")]
    InvalidCurrencyCode,
    #[error("failed to open new bank account: {0}")]
    BankAccountError(#[from] BankAccountError),
    #[error("failed to save bank account: {0}")]
    RepositoryError(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

#[async_trait]
impl<R> command::Handler<Command> for Handler<R>
where
    R: entity::Repository<BankAccountRoot>,
    <R as entity::Saver<BankAccountRoot>>::Error: StdError + Send + Sync + 'static,
{
    type Error = Error;

    async fn handle(&self, command: command::Envelope<Command>) -> Result<(), Self::Error> {
        let command = command.message;

        let currency = iso::find(&command.currency_code).ok_or(Error::InvalidCurrencyCode)?;

        let mut bank_account =
            BankAccountRoot::open(command.iban, command.account_holder_id, *currency).unwrap();

        self.bank_account_repository
            .store(&mut bank_account)
            .await
            .map_err(|err| Error::RepositoryError(Box::new(err)))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use eventually::{aggregate, command, event};
    use rusty_money::iso;

    use crate::bank_account::{
        open_new_account::{Command as OpenNewBankAccount, Handler as OpenNewBankAccountHandler},
        BankAccount,
        DomainEvent::*,
    };

    type BankAccountRepository<S> = aggregate::EventSourcedRepository<BankAccount, S>;

    static TEST_IBAN: &'static str = "DE10071007100710078888";
    static TEST_ACCOUNT_HOLDER_ID: &'static str = "account-holder-test-1";
    static TEST_CURRENCY_CODE: &'static str = "EUR";
    static TEST_CURRENCY: &'static iso::Currency = iso::EUR;

    #[tokio::test]
    async fn it_opens_a_new_bank_account_if_it_does_not_exist_yet() {
        command::test::Scenario
            .when(
                OpenNewBankAccount {
                    iban: TEST_IBAN.into(),
                    account_holder_id: TEST_ACCOUNT_HOLDER_ID.into(),
                    currency_code: TEST_CURRENCY_CODE.into(),
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: TEST_IBAN.into(),
                version: 1,
                event: NewBankAccountOpened {
                    iban: TEST_IBAN.into(),
                    account_holder_id: TEST_ACCOUNT_HOLDER_ID.into(),
                    currency: *TEST_CURRENCY,
                }
                .into(),
            }])
            .assert_on(|event_store| OpenNewBankAccountHandler {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn it_fails_to_open_a_new_bank_account_if_currency_code_is_not_ISO_4217() {
        command::test::Scenario
            .when(
                OpenNewBankAccount {
                    iban: TEST_IBAN.into(),
                    account_holder_id: TEST_ACCOUNT_HOLDER_ID.into(),
                    currency_code: "XXX".into(),
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| OpenNewBankAccountHandler {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }
}
