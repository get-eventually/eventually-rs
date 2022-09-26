use eventually_postgres::serde::{ByteArray, Deserializer, Serializer};
use prost::{bytes::Bytes, Message};
use rust_decimal::prelude::ToPrimitive;

use crate::{domain, domain::BankAccountEvent, proto, proto::Event as ProtoBankAccountEvent};

#[derive(Debug, Clone, Copy)]
pub struct BankAccountEventSerde;

impl Serializer<ProtoBankAccountEvent> for BankAccountEventSerde {
    fn serialize(&self, value: ProtoBankAccountEvent) -> ByteArray {
        value.encode_to_vec()
    }
}

impl Deserializer<ProtoBankAccountEvent> for BankAccountEventSerde {
    type Error = prost::DecodeError;

    fn deserialize(&self, value: ByteArray) -> Result<ProtoBankAccountEvent, Self::Error> {
        let buf = Bytes::from(value);
        let event = ProtoBankAccountEvent::decode(buf)?;
        Ok(event)
    }
}

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
                }
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
                }
                BankAccountEvent::TransferWasDeclined {
                    transaction_id,
                    reason,
                } => proto::event::Event::TransferWasDeclined(proto::event::TransferWasDeclined {
                    transaction_id,
                    reason,
                }),
                BankAccountEvent::WasClosed => {
                    proto::event::Event::WasClosed(proto::event::WasClosed {})
                }
                BankAccountEvent::WasReopened { reopening_balance } => {
                    proto::event::Event::WasReopened(proto::event::WasReopened {
                        reopening_balance: reopening_balance.unwrap_or_default().to_f32().unwrap(),
                    })
                }
            }),
        }
    }
}

impl From<ProtoBankAccountEvent> for BankAccountEvent {
    fn from(proto: ProtoBankAccountEvent) -> Self {
        todo!()
    }
}
