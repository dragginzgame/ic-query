use super::*;

// Independent short-field encoding from the ICP Ledger protobuf schema.
fn message(tag: u8, value: &[u8]) -> Vec<u8> {
    assert!(value.len() < 128);
    [
        vec![(tag << 3) | 2, u8::try_from(value.len()).unwrap()],
        value.to_vec(),
    ]
    .concat()
}

fn fixture(parent: bool) -> Vec<u8> {
    let send = [
        message(1, &message(1, &[1; 32])),
        message(2, &message(1, &[2; 32])),
        message(3, &[8, 10]),
        message(4, &[8, 1]),
    ]
    .concat();
    let transaction = [message(3, &send), message(4, &[8, 3]), message(6, &[8, 5])].concat();
    let mut block = if parent {
        message(1, &message(1, &[9; 32]))
    } else {
        vec![]
    };
    block.extend(message(2, &[8, 6]));
    block.extend(message(3, &transaction));
    block
}

#[test]
fn decodes_exact_transfer_at_genesis_and_later_indices() {
    for (parent, index) in [(false, 0), (true, 77)] {
        let bytes = fixture(parent);
        assert_eq!(
            decode_transfer(&bytes, index, bytes.len()).unwrap(),
            IcpTransferFields {
                from: [1; 32],
                to: [2; 32],
                amount_e8s: 10,
                fee_e8s: 1,
                memo: 3,
                created_at_time_ns: 5,
            }
        );
    }
}

#[test]
fn byte_budget_is_checked_before_decoding() {
    assert_eq!(
        decode_transfer(&[255], 0, 0),
        Err(IcpBlockError::BudgetExceeded)
    );
    let bytes = fixture(true);
    assert_eq!(
        decode_transfer(&bytes, 1, bytes.len() - 1),
        Err(IcpBlockError::BudgetExceeded)
    );
}

#[test]
fn preserves_present_zero_wrappers() {
    let send = [
        message(1, &message(1, &[1; 32])),
        message(2, &message(1, &[2; 32])),
        message(3, &[]),
        message(4, &[]),
    ]
    .concat();
    let transaction = [message(3, &send), message(4, &[]), message(6, &[])].concat();
    let block = [message(2, &[]), message(3, &transaction)].concat();
    assert_eq!(
        decode_transfer(&block, 0, block.len()).unwrap(),
        IcpTransferFields {
            from: [1; 32],
            to: [2; 32],
            amount_e8s: 0,
            fee_e8s: 0,
            memo: 0,
            created_at_time_ns: 0,
        }
    );
}

#[test]
fn refuses_approve_transfer_from_and_duplicate_nested_accounts() {
    let send = [
        message(1, &message(1, &[1; 32])),
        message(2, &message(1, &[2; 32])),
        message(3, &[8, 10]),
        message(4, &[8, 1]),
    ]
    .concat();
    for extra in [
        message(5, &[]),
        message(6, &[]),
        message(1, &message(1, &[1; 32])),
    ] {
        let transaction = [
            message(3, &[send.clone(), extra].concat()),
            message(4, &[8, 3]),
            message(6, &[8, 5]),
        ]
        .concat();
        let block = [message(2, &[8, 6]), message(3, &transaction)].concat();
        assert_eq!(
            decode_transfer(&block, 0, block.len()),
            Err(IcpBlockError::UnsupportedBlock)
        );
    }
}

#[test]
fn rejects_noncanonical_unknown_duplicate_and_truncated_fields() {
    let bytes = fixture(true);
    for extra in [message(4, &[]), message(2, &[8, 6])] {
        let mut changed = bytes.clone();
        changed.extend(extra);
        assert_eq!(
            decode_transfer(&changed, 1, 1024),
            Err(IcpBlockError::UnsupportedBlock)
        );
    }
    assert_eq!(
        decode_transfer(&bytes[..bytes.len() - 1], 1, 1024),
        Err(IcpBlockError::UnsupportedBlock)
    );
    let mut overlong = bytes;
    overlong.splice(1..2, [overlong[1] | 128, 0]);
    assert_eq!(
        decode_transfer(&overlong, 1, 1024),
        Err(IcpBlockError::UnsupportedBlock)
    );
}

#[test]
fn rejects_missing_fields_wrong_accounts_and_parent_shape() {
    for case in 0..10 {
        let mut block = protobuf::Block::decode(fixture(true).as_slice()).unwrap();
        match case {
            0 => block.timestamp = None,
            1 => block.transaction = None,
            2 => block.transaction.as_mut().unwrap().send = None,
            3 => block.transaction.as_mut().unwrap().memo = None,
            4 => block.transaction.as_mut().unwrap().created_at_time = None,
            5 => {
                block
                    .transaction
                    .as_mut()
                    .unwrap()
                    .send
                    .as_mut()
                    .unwrap()
                    .fee = None;
            }
            6 => {
                block
                    .transaction
                    .as_mut()
                    .unwrap()
                    .send
                    .as_mut()
                    .unwrap()
                    .amount = None;
            }
            7 => {
                block
                    .transaction
                    .as_mut()
                    .unwrap()
                    .send
                    .as_mut()
                    .unwrap()
                    .from = None;
            }
            8 => block
                .parent_hash
                .as_mut()
                .unwrap()
                .value
                .pop()
                .map(|_| ())
                .unwrap(),
            9 => {
                block
                    .transaction
                    .as_mut()
                    .unwrap()
                    .send
                    .as_mut()
                    .unwrap()
                    .to = None;
            }
            _ => unreachable!(),
        }
        assert_eq!(
            decode_transfer(&block.encode_to_vec(), 1, 1024),
            Err(IcpBlockError::UnsupportedBlock)
        );
    }
    for source in [true, false] {
        let mut block = protobuf::Block::decode(fixture(true).as_slice()).unwrap();
        let send = block.transaction.as_mut().unwrap().send.as_mut().unwrap();
        let account = if source { &mut send.from } else { &mut send.to };
        account.as_mut().unwrap().value.pop();
        assert_eq!(
            decode_transfer(&block.encode_to_vec(), 1, 1024),
            Err(IcpBlockError::InvalidAccount)
        );
    }
    assert_eq!(
        decode_transfer(&fixture(true), 0, 1024),
        Err(IcpBlockError::UnsupportedBlock)
    );
    assert_eq!(
        decode_transfer(&fixture(false), 1, 1024),
        Err(IcpBlockError::UnsupportedBlock)
    );
}
