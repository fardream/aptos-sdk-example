module amovepackage::deployer {
	use std::signer;

	use aptos_framework::account::{Self, SignerCapability};
	use aptos_std::simple_map::{Self, SimpleMap};

	const E_RESOURCE_ACCOUNT_NOT_OWNED: u64 = 1001;
	const E_CAN_ONLY_ASSIGN_TO_SELF: u64 = 1002;

	struct CapabilityStore has key {
		owned_resource_accounts: SimpleMap<address, u8>,
	}

	public entry fun maybe_register_for_store(u: &signer) {
		if (!exists<CapabilityStore>(signer::address_of(u))) {
			move_to(u, CapabilityStore{
				owned_resource_accounts: simple_map::create(),
			});
		};
	}

	struct SelfStore has key {
		signer_capability: SignerCapability,
	}

	public entry fun create_resource_account(u: &signer, seed: vector<u8>) acquires CapabilityStore {
		maybe_register_for_store(u);
		let (resource_account_signer, signer_capability) = account::create_resource_account(u, seed);
		let store = borrow_global_mut<CapabilityStore>(signer::address_of(u));
		simple_map::add(&mut store.owned_resource_accounts, signer::address_of(&resource_account_signer), 0);
		move_to(&resource_account_signer, SelfStore{
			signer_capability,
		});
	}
}
