use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::domain::BankAccountEvent;
use crate::proto::Event as ProtoBankAccountEvent;
use crate::{domain, proto};

impl From<domain::Transaction> for proto::Transaction {
    fn from(tx: domain::Transaction) -> Self {
        Self {
            id: tx.id,
            beneficiary_account_id: tx.beneficiary_account_id,
            amount: tx.amount.to_f32().unwrap(),
        }
    }
}

impl From<BankAccountEvent> for ProtoBankAccountEvent {
    fn from(event: BankAccountEvent) -> Self {
        Self {
            event: Some(match event {
                BankAccountEvent::WasOpened {
                    id,
                    account_holder_id,
                    initial_balance,
                } => proto::event::Event::WasOpened(proto::event::WasOpened {
                    id,
                    account_holder_id,
                    initial_balance: initial_balance.unwrap_or_default().to_f32().unwrap(),
                }),
                BankAccountEvent::DepositWasRecorded { amount } => {
                    proto::event::Event::DepositWasRecorded(proto::event::DepositWasRecorded {
                        amount: amount.to_f32().unwrap(),
                    })
                },
                BankAccountEvent::TransferWasSent {
                    transaction,
                    message,
                } => proto::event::Event::TransferWasSent(proto::event::TransferWasSent {
                    transaction: Some(transaction.into()),
                    msg: message,
                }),
                BankAccountEvent::TransferWasReceived {
                    transaction,
                    message,
                } => proto::event::Event::TransferWasReceived(proto::event::TransferWasReceived {
                    transaction: Some(transaction.into()),
                    msg: message,
                }),
                BankAccountEvent::TransferWasConfirmed { transaction_id } => {
                    proto::event::Event::TransferWasConfimed(proto::event::TransferWasConfirmed {
                        transaction_id,
                    })
                },
                BankAccountEvent::TransferWasDeclined {
                    transaction_id,
                    reason,
                } => proto::event::Event::TransferWasDeclined(proto::event::TransferWasDeclined {
                    transaction_id,
                    reason,
                }),
                BankAccountEvent::WasClosed => {
                    proto::event::Event::WasClosed(proto::event::WasClosed {})
                },
                BankAccountEvent::WasReopened { reopening_balance } => {
                    proto::event::Event::WasReopened(proto::event::WasReopened {
                        reopening_balance: reopening_balance.unwrap_or_default().to_f32().unwrap(),
                    })
                },
            }),
        }
    }
}

impl From<ProtoBankAccountEvent> for BankAccountEvent {
    fn from(proto: ProtoBankAccountEvent) -> Self {
        match proto.event.expect("event is a required field") {
            proto::event::Event::WasOpened(proto::event::WasOpened {
                id,
                account_holder_id,
                initial_balance,
            }) => BankAccountEvent::WasOpened {
                id,
                account_holder_id,
                initial_balance: Some(Decimal::from_f32(initial_balance).unwrap()),
            },
            proto::event::Event::DepositWasRecorded(proto::event::DepositWasRecorded {
                amount,
            }) => BankAccountEvent::DepositWasRecorded {
                amount: Decimal::from_f32(amount).unwrap(),
            },
            // TODO: fill these as more implementations are added to the service.
            proto::event::Event::TransferWasSent(_) => todo!(),
            proto::event::Event::TransferWasReceived(_) => todo!(),
            proto::event::Event::TransferWasConfimed(_) => todo!(),
            proto::event::Event::TransferWasDeclined(_) => todo!(),
            proto::event::Event::WasClosed(_) => todo!(),
            proto::event::Event::WasReopened(_) => todo!(),
        }
    }
}
