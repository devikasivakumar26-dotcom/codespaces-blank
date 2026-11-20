#[allow(unused)]
mod pb;

use pb::{mydata::v1 as mydata, sf::substreams::solana::v1::Transactions};

const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

#[substreams::handlers::map]
fn map_my_data(transactions: Transactions) -> mydata::MyData {
    let mut output = mydata::MyData::default();

    for confirmed in transactions.transactions {
        let Some(tx) = confirmed.transaction.as_option() else {
            continue;
        };
        let Some(message) = tx.message.as_option() else {
            continue;
        };

        for ix in &message.instructions {
            let Some(program_bytes) =
                message.account_keys.get(ix.program_id_index as usize)
            else {
                continue;
            };

            let program = bs58::encode(program_bytes).into_string();

            let accounts = &ix.accounts;
            let data = &ix.data;

            let parsed = if program == SYSTEM_PROGRAM
                && data.len() >= 12
                && u32::from_le_bytes(data[0..4].try_into().unwrap()) == 2
                && accounts.len() >= 2
            {
                Some((accounts[0] as usize, accounts[1] as usize,
                    u64::from_le_bytes(data[4..12].try_into().unwrap())))
            } else if (program == TOKEN_PROGRAM || program == TOKEN_2022_PROGRAM)
                && data.len() >= 9
                && (data[0] == 3 || data[0] == 12)
                && accounts.len() >= 2
            {
                let to_index = if data[0] == 12 { 2 } else { 1 };
                if accounts.len() > to_index {
                    Some((
                        accounts[0] as usize,
                        accounts[to_index] as usize,
                        u64::from_le_bytes(data[1..9].try_into().unwrap()),
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((from_index, to_index, amount)) = parsed {
                if let (Some(from), Some(to)) = (
                    message.account_keys.get(from_index),
                    message.account_keys.get(to_index),
                ) {
                    output.transfers.push(mydata::Transfer {
                        from: bs58::encode(from).into_string(),
                        to: bs58::encode(to).into_string(),
                        amount,
                    });
                }
            }
        }
    }

    output
}
