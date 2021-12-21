use std::error::Error as StdError;

use async_trait::async_trait;
use eventually::{aggregate, command::Handler};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    application,
    domain::{BankAccount, BankAccountRoot},
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
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }
}
