use std::error::Error as StdError;

use async_trait::async_trait;
use eventually::{aggregate, command, message};
use rust_decimal::Decimal;

use crate::domain::{
    BankAccount, BankAccountHolderId, BankAccountId, BankAccountRoot, Transaction,
};

#[derive(Clone)]
pub struct Service<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
{
    bank_account_repository: R,
}

impl<R> From<R> for Service<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
{
    fn from(bank_account_repository: R) -> Self {
        Self {
            bank_account_repository,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenBankAccount {
    pub bank_account_id: BankAccountId,
    pub bank_account_holder_id: BankAccountHolderId,
    pub opening_balance: Option<Decimal>,
}

impl message::Message for OpenBankAccount {
    fn name(&self) -> &'static str {
        "OpenBankAccount"
    }
}

#[async_trait]
impl<R> command::Handler<OpenBankAccount> for Service<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
    R::Error: StdError + Send + Sync + 'static,
{
    type Error = anyhow::Error;

    async fn handle(&self, command: command::Envelope<OpenBankAccount>) -> Result<(), Self::Error> {
        let command = command.message;

        let mut bank_account = BankAccountRoot::open(
            command.bank_account_id,
            command.bank_account_holder_id,
            command.opening_balance,
        )?;

        self.bank_account_repository
            .store(&mut bank_account)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepositInBankAccount {
    pub bank_account_id: BankAccountId,
    pub amount: Decimal,
}

impl message::Message for DepositInBankAccount {
    fn name(&self) -> &'static str {
        "DepositInBankAccount"
    }
}

#[async_trait]
impl<R> command::Handler<DepositInBankAccount> for Service<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
    R::Error: StdError + Send + Sync + 'static,
{
    type Error = anyhow::Error;

    async fn handle(
        &self,
        command: command::Envelope<DepositInBankAccount>,
    ) -> Result<(), Self::Error> {
        let command = command.message;

        let mut bank_account = self
            .bank_account_repository
            .get(&command.bank_account_id)
            .await?;

        bank_account.deposit(command.amount)?;

        self.bank_account_repository
            .store(&mut bank_account)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendTransferToBankAccount {
    pub bank_account_id: BankAccountId,
    pub transaction: Transaction,
    pub message: Option<String>,
}

impl message::Message for SendTransferToBankAccount {
    fn name(&self) -> &'static str {
        "SendTransferToBankAccount"
    }
}

#[async_trait]
impl<R> command::Handler<SendTransferToBankAccount> for Service<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
    R::Error: StdError + Send + Sync + 'static,
{
    type Error = anyhow::Error;

    async fn handle(
        &self,
        command: command::Envelope<SendTransferToBankAccount>,
    ) -> Result<(), Self::Error> {
        let command = command.message;

        let mut bank_account = self
            .bank_account_repository
            .get(&command.bank_account_id)
            .await?;

        bank_account.send_transfer(command.transaction, command.message)?;

        self.bank_account_repository
            .store(&mut bank_account)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use eventually::{command, event};
    use rust_decimal::Decimal;

    use crate::{
        application,
        domain::{BankAccountEvent, BankAccountRepository, Transaction},
    };

    #[tokio::test]
    async fn open_bank_account_works_when_bank_account_has_just_been_opened_for_the_first_time() {
        command::test::Scenario
            .when(
                application::OpenBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    bank_account_holder_id: "dani".to_owned(),
                    opening_balance: Some(Decimal::new(1000, 2)), // 10,00
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 1,
                event: BankAccountEvent::WasOpened {
                    id: "account-test".to_owned(),
                    account_holder_id: "dani".to_owned(),
                    initial_balance: Some(Decimal::new(1000, 2)),
                }
                .into(),
            }])
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn open_bank_account_fails_if_the_account_already_exists() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 1,
                event: BankAccountEvent::WasOpened {
                    id: "account-test".to_owned(),
                    account_holder_id: "dani".to_owned(),
                    initial_balance: Some(Decimal::new(1000, 2)),
                }
                .into(),
            }])
            .when(
                application::OpenBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    bank_account_holder_id: "dani".to_owned(),
                    opening_balance: Some(Decimal::new(1000, 2)), // 10,00
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn deposit_money_fails_on_unexisting_bank_account() {
        command::test::Scenario
            .when(
                application::DepositInBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    amount: Decimal::new(2000, 2), // 20,00
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn deposit_money_on_existing_bank_account_works_when_amount_is_positive() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 1,
                event: BankAccountEvent::WasOpened {
                    id: "account-test".to_owned(),
                    account_holder_id: "dani".to_owned(),
                    initial_balance: Some(Decimal::new(1000, 2)),
                }
                .into(),
            }])
            .when(
                application::DepositInBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    amount: Decimal::new(2000, 2), // 20,00
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 2,
                event: BankAccountEvent::DepositWasRecorded {
                    amount: Decimal::new(2000, 2), // 20,00
                }
                .into(),
            }])
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn deposit_money_on_existing_bank_account_fails_when_amount_is_negative() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 1,
                event: BankAccountEvent::WasOpened {
                    id: "account-test".to_owned(),
                    account_holder_id: "dani".to_owned(),
                    initial_balance: Some(Decimal::new(1000, 2)),
                }
                .into(),
            }])
            .when(
                application::DepositInBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    amount: Decimal::new(-2000, 2), // -20,00
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn deposit_money_with_zero_amount_in_open_bank_account_fails() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: "account-test".to_owned(),
                version: 1,
                event: BankAccountEvent::WasOpened {
                    id: "account-test".to_owned(),
                    account_holder_id: "dani".to_owned(),
                    initial_balance: Some(Decimal::new(1000, 2)),
                }
                .into(),
            }])
            .when(
                application::DepositInBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    amount: Decimal::new(0, 0),
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn deposit_money_on_existing_bank_account_fails_when_account_is_closed() {
        command::test::Scenario
            .given(vec![
                event::Persisted {
                    stream_id: "account-test".to_owned(),
                    version: 1,
                    event: BankAccountEvent::WasOpened {
                        id: "account-test".to_owned(),
                        account_holder_id: "dani".to_owned(),
                        initial_balance: Some(Decimal::new(1000, 2)),
                    }
                    .into(),
                },
                event::Persisted {
                    stream_id: "account-test".to_owned(),
                    version: 2,
                    event: BankAccountEvent::WasClosed.into(),
                },
            ])
            .when(
                application::DepositInBankAccount {
                    bank_account_id: "account-test".to_owned(),
                    amount: Decimal::new(2000, 2), // 20,00
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn send_transfer_fails_if_bank_account_does_not_exist() {
        command::test::Scenario
            .when(
                application::SendTransferToBankAccount {
                    bank_account_id: "sender".to_owned(),
                    transaction: Transaction {
                        id: "transaction".to_owned(),
                        beneficiary_account_id: "receiver".to_owned(),
                        amount: Decimal::new(2000, 2),
                    },
                    message: None,
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn send_transfer_fails_if_bank_account_does_not_have_sufficient_funds() {
        command::test::Scenario
            .given(vec![
                event::Persisted {
                    stream_id: "sender".to_owned(),
                    version: 1,
                    event: BankAccountEvent::WasOpened {
                        id: "sender".to_owned(),
                        account_holder_id: "sender-name".to_owned(),
                        initial_balance: Some(Decimal::new(1_000, 0)),
                    }
                    .into(),
                },
                event::Persisted {
                    stream_id: "receiver".to_owned(),
                    version: 1,
                    event: BankAccountEvent::WasOpened {
                        id: "receiver".to_owned(),
                        account_holder_id: "receiver-name".to_owned(),
                        initial_balance: None,
                    }
                    .into(),
                },
            ])
            .when(
                application::SendTransferToBankAccount {
                    bank_account_id: "sender".to_owned(),
                    transaction: Transaction {
                        id: "transaction".to_owned(),
                        beneficiary_account_id: "receiver".to_owned(),
                        amount: Decimal::new(2_000, 0),
                    },
                    message: None,
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }

    #[tokio::test]
    async fn send_transfer_works_if_bank_account_has_sufficient_funds() {
        command::test::Scenario
            .given(vec![
                event::Persisted {
                    stream_id: "sender".to_owned(),
                    version: 1,
                    event: BankAccountEvent::WasOpened {
                        id: "sender".to_owned(),
                        account_holder_id: "sender-name".to_owned(),
                        initial_balance: Some(Decimal::new(1_000, 0)),
                    }
                    .into(),
                },
                event::Persisted {
                    stream_id: "receiver".to_owned(),
                    version: 1,
                    event: BankAccountEvent::WasOpened {
                        id: "receiver".to_owned(),
                        account_holder_id: "receiver-name".to_owned(),
                        initial_balance: None,
                    }
                    .into(),
                },
            ])
            .when(
                application::SendTransferToBankAccount {
                    bank_account_id: "sender".to_owned(),
                    transaction: Transaction {
                        id: "transaction".to_owned(),
                        beneficiary_account_id: "receiver".to_owned(),
                        amount: Decimal::new(500, 0),
                    },
                    message: None,
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: "sender".to_owned(),
                version: 2,
                event: BankAccountEvent::TransferWasSent {
                    transaction: Transaction {
                        id: "transaction".to_owned(),
                        beneficiary_account_id: "receiver".to_owned(),
                        amount: Decimal::new(500, 0),
                    },
                    message: None,
                }
                .into(),
            }])
            .assert_on(|event_store| application::Service {
                bank_account_repository: BankAccountRepository::from(event_store),
            })
            .await;
    }
}
