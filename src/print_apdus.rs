use ledger_apdu::APDUCommand;

use crate::{
    hd_path_to_bytes, log_command, CHUNK_SIZE, CLA, NETWORK_ID, P1_SIGN_NORMAL,
    P1_SIGN_NORMAL_LAST_CHUNK,
};

/// this method is counterpart of [`crate::sign_transaction`]
pub fn transaction(
    unsigned_tx: crate::BorshSerializedUnsignedTransaction,
    seed_phrase_hd_path: near_slip10::BIP32Path,
) {
    print_payload_apdus(
        unsigned_tx,
        seed_phrase_hd_path,
        crate::INS_SIGN_TRANSACTION,
    )
}

/// this method is counterpart of [`crate::sign_message_nep413`]
pub fn message_nep413(payload: &crate::NEP413Payload, seed_phrase_hd_path: near_slip10::BIP32Path) {
    print_payload_apdus(
        &borsh::to_vec(payload).unwrap(),
        seed_phrase_hd_path,
        crate::INS_SIGN_NEP413_MESSAGE,
    )
}

/// this method is counterpart of [`crate::sign_message_nep366_delegate_action`]
pub fn nep366_delegate_action(
    payload: crate::BorshSerializedDelegateAction,
    seed_phrase_hd_path: near_slip10::BIP32Path,
) {
    print_payload_apdus(
        payload,
        seed_phrase_hd_path,
        crate::INS_SIGN_NEP366_DELEGATE_ACTION,
    )
}

/// this method is counterpart of [`crate::send_payload_internal`]
/// which only prints intended interaction with transport, but doesn't send anything
/// over it
pub(crate) fn print_payload_apdus(
    payload: &[u8],
    seed_phrase_hd_path: near_slip10::BIP32Path,
    ins: u8,
) {
    // seed_phrase_hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&seed_phrase_hd_path);

    let mut data: Vec<u8> = vec![];
    data.extend(hd_path_bytes);
    data.extend_from_slice(payload);
    let chunks = data.chunks(CHUNK_SIZE);
    let chunks_count = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        let is_last_chunk = chunks_count == i + 1;
        let command = APDUCommand {
            cla: CLA,
            ins,
            p1: if is_last_chunk {
                P1_SIGN_NORMAL_LAST_CHUNK
            } else {
                P1_SIGN_NORMAL
            }, // Instruction parameter 1 (offset)
            p2: NETWORK_ID,
            data: chunk.to_vec(),
        };
        log_command(i, is_last_chunk, &command);
    }
}
