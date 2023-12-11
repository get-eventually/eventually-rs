use async_trait::async_trait;
use eventually::command::Handler;
use eventually::version;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tracing::instrument;

use crate::domain::BankAccountError;
use crate::{application, proto};

#[derive(Clone)]
pub struct BankAccountingApi {
    application_service: application::Service,
}

impl From<application::Service> for BankAccountingApi {
    fn from(application_service: application::Service) -> Self {
        Self {
            application_service,
        }
    }
}

#[async_trait]
impl proto::bank_accounting_server::BankAccounting for BankAccountingApi {
    #[instrument(skip(self))]
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

                let bank_error = as_error::<BankAccountError>(&e);
                let conflict_error = as_error::<version::ConflictError>(&e);

                if let Some(EmptyAccountId | EmptyAccountHolderId) = bank_error {
                    tonic::Status::invalid_argument(e.to_string())
                } else if let Some(e) = conflict_error {
                    tonic::Status::already_exists(AlreadyOpened.to_string())
                } else {
                    tonic::Status::internal(e.to_string())
                }
            })
    }

    #[instrument(skip(self))]
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

                let bank_error = as_error::<BankAccountError>(&e);
                let conflict_error = as_error::<version::ConflictError>(&e);

                if let Some(Closed | NegativeDepositAttempted) = bank_error {
                    tonic::Status::failed_precondition(e.to_string())
                } else if let Some(NoMoneyDeposited) = bank_error {
                    tonic::Status::invalid_argument(e.to_string())
                } else if let Some(e) = conflict_error {
                    tonic::Status::failed_precondition(e.to_string())
                } else {
                    tonic::Status::internal(e.to_string())
                }
            })
    }
}

fn as_error<T>(e: &anyhow::Error) -> Option<&T>
where
    T: std::error::Error + Send + Sync + 'static,
{
    e.source().and_then(move |e| e.downcast_ref::<T>())
}
