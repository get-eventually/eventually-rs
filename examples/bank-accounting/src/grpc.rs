use std::error::Error as StdError;

use async_trait::async_trait;
use eventually::{aggregate, command::Handler, version};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    application,
    domain::{BankAccount, BankAccountError, BankAccountRoot},
    proto,
};

#[derive(Clone)]
pub struct BankAccountingApi<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
{
    application_service: application::Service<R>,
}

impl<R> From<application::Service<R>> for BankAccountingApi<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot>,
{
    fn from(application_service: application::Service<R>) -> Self {
        Self {
            application_service,
        }
    }
}

#[async_trait]
impl<R> proto::bank_accounting_server::BankAccounting for BankAccountingApi<R>
where
    R: aggregate::Repository<BankAccount, BankAccountRoot> + 'static,
    R::Error: StdError + Send + Sync + 'static,
{
    async fn open_bank_account(
        &self,
        request: tonic::Request<proto::OpenBankAccountRequest>,
    ) -> Result<tonic::Response<proto::OpenBankAccountResponse>, tonic::Status> {
        let request = request.into_inner();

        self.application_service
            .handle(
                application::OpenBankAccount {
                    bank_account_id: request.bank_account_id,
                    bank_account_holder_id: request.bank_account_holder_id,
                    opening_balance: Decimal::from_f32(request.opening_balance),
                }
                .into(),
            )
            .await
            .map(|_| tonic::Response::new(proto::OpenBankAccountResponse {}))
            .map_err(|e| {
                use BankAccountError::*;

                let bank_error = e
                    .source()
                    .and_then(|e| e.downcast_ref::<BankAccountError>());

                let conflict_error = e
                    .source()
                    .and_then(|e| e.downcast_ref::<version::ConflictError>());

                if let Some(EmptyAccountId | EmptyAccountHolderId) = bank_error {
                    tonic::Status::invalid_argument(e.to_string())
                } else if conflict_error.is_some() {
                    tonic::Status::already_exists(AlreadyOpened.to_string())
                } else {
                    tonic::Status::internal(e.to_string())
                }
            })
    }

    async fn deposit_in_bank_account(
        &self,
        request: tonic::Request<proto::DepositInBankAccountRequest>,
    ) -> Result<tonic::Response<proto::DepositInBankAccountResponse>, tonic::Status> {
        let request = request.into_inner();

        self.application_service
            .handle(
                application::DepositInBankAccount {
                    bank_account_id: request.bank_account_id,
                    amount: Decimal::from_f32(request.amount).ok_or_else(|| {
                        tonic::Status::invalid_argument("amount should be more than 0")
                    })?,
                }
                .into(),
            )
            .await
            .map(|_| tonic::Response::new(proto::DepositInBankAccountResponse {}))
            .map_err(|e| {
                use BankAccountError::*;

                let bank_error = e
                    .source()
                    .and_then(|e| e.downcast_ref::<BankAccountError>());

                let conflict_error = e
                    .source()
                    .and_then(|e| e.downcast_ref::<version::ConflictError>());

                if let Some(Closed | NegativeDepositAttempted) = bank_error {
                    tonic::Status::failed_precondition(e.to_string())
                } else if let Some(NoMoneyDeposited) = bank_error {
                    tonic::Status::invalid_argument(e.to_string())
                } else if conflict_error.is_some() {
                    tonic::Status::failed_precondition(e.to_string())
                } else {
                    tonic::Status::internal(e.to_string())
                }
            })
    }
}
