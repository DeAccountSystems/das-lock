
#include "inc_def.h"


int verify_signature(uint8_t *message, uint8_t *lock_bytes,
                    const void *lock_args) {
	debug_print("Enter verify_signature");
	debug_print_data("digest: ", message, HASH_SIZE);
	debug_print_data("WitnessArgs.lock(lock_bytes): ", lock_bytes, RECID_INDEX + 1);
	debug_print_data("lock_args: ", lock_args, BLAKE160_SIZE);
	int ret = CKB_SUCCESS;
	uint8_t temp[TEMP_SIZE];

	/* Load signature */
	secp256k1_context context;
	uint8_t secp_data[CKB_SECP256K1_DATA_SIZE];
	ret = ckb_secp256k1_custom_verify_only_initialize(&context, secp_data);
	SIMPLE_ASSERT(CKB_SUCCESS);

	debug_print_int("recid ", lock_bytes[RECID_INDEX]);
	secp256k1_ecdsa_recoverable_signature signature;
	if (secp256k1_ecdsa_recoverable_signature_parse_compact(
		&context, &signature, lock_bytes, lock_bytes[RECID_INDEX]) == 0) {
		return ERROR_SECP_PARSE_SIGNATURE;
	}
	debug_print_data("secp256k1_ecdsa_recoverable_signature signature: ", signature.data, 65);

	/* Recover pubkey */
	secp256k1_pubkey pubkey;
	if (secp256k1_ecdsa_recover(&context, &pubkey, &signature, message) != 1) {
		return ERROR_SECP_RECOVER_PUBKEY;
	}
	debug_print_data("secp256k1_pubkey pubkey: ", pubkey.data, 64);

	/* Check pubkey hash */
	size_t pubkey_size = SIGNATURE_SIZE;
	if (secp256k1_ec_pubkey_serialize(&context, temp, &pubkey_size, &pubkey,
					  SECP256K1_EC_UNCOMPRESSED) != 1) {
		return ERROR_SECP_SERIALIZE_PUBKEY;
	}
	debug_print_data("before last blake2b, temp: ", temp, pubkey_size);

	SHA3_CTX sha3_ctx;
	keccak_init(&sha3_ctx);
	keccak_update(&sha3_ctx, &temp[1], pubkey_size - 1);
	keccak_final(&sha3_ctx, temp);

	debug_print_data("after last blake2b, temp: ", temp, pubkey_size);
	if (memcmp(lock_args, &temp[12], BLAKE160_SIZE) != 0) {
		return ERROR_PUBKEY_BLAKE160_HASH;
	}

	debug_print_int("END OF TRON SIGN: ret ", ret);
	return ret;
}

__attribute__((visibility("default"))) int validate(int type, uint8_t* message, uint8_t* lock_bytes, uint8_t* eth_address) {

	debug_print("Enter validate");
	debug_print_data("digest before keccak with tron prefix: ", message, HASH_SIZE);

	SHA3_CTX sha3_ctx;
	keccak_init(&sha3_ctx);
	/* personal hash, tron prefix  \u0019TRON Signed Message:\n32  */
	uint8_t tron_prefix[24];
	tron_prefix[0] = 0x19;
	memcpy(tron_prefix + 1, "TRON Signed Message:\n32", 23);

	keccak_update(&sha3_ctx, tron_prefix, 24);
	keccak_update(&sha3_ctx, message, 32);

	uint8_t for_compatible[1];
	for_compatible[0] = 0x4;
	keccak_update(&sha3_ctx, for_compatible, 1);
	keccak_final(&sha3_ctx, message);

	/* verify signature with peronsal hash */
	return verify_signature(message, lock_bytes, eth_address);
}
