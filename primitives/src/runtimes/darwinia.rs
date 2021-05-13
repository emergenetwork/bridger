//! Darwinia Runtime
#![cfg(feature = "runtime")]

use crate::{
	chain::{
		ethereum::EthereumRelayHeaderParcel, proxy_type::ProxyType, RelayAffirmation,
		RelayAffirmationId, RelayVotingState,
	},
	frame::{
		bridge::relay_authorities::EthereumRelayAuthorities,
		ethereum::{
            backing::EthereumBacking, 
            game::EthereumRelayerGame, 
            relay::EthereumRelay,
            relay_helper::EthereumRelayHelper,
        },
		proxy::Proxy,
		sudo::Sudo,
		technical_committee::TechnicalCommittee,
	},
};

use crate::frame::bridge::relay_authorities::RelayAuthority;
use crate::frame::ethereum::runtime_ext::{
    DarwiniaMainnetVerifier, 
    DarwiniaMainnetErc20Redeemer, 
    DarwiniaMainnetErc20Register,
    RuntimeExt
};

use substrate_subxt::{
	balances::{AccountData, Balances},
	extrinsic::DefaultExtra,
	register_default_type_sizes, sp_core,
	sp_runtime::{
		generic::Header,
		traits::{BlakeTwo256, IdentifyAccount, Verify},
		MultiAddress, MultiSignature, OpaqueExtrinsic,
	},
	system::System,
	EventTypeRegistry, Runtime,
};

use crate::chain::ethereum::{EcdsaAddress, EcdsaMessage, EcdsaSignature};
use codec::{Decode, Encode};
use sp_core::{H160, H256, U256};

type SessionIndex = u32;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
struct Log {
	address: H160,
	topics: Vec<H256>,
	data: Vec<u8>,
}

/// Darwinia Runtime
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DarwiniaRuntime;
impl Runtime for DarwiniaRuntime {
	type Signature = MultiSignature;
	type Extra = DefaultExtra<Self>;

	fn register_type_sizes(registry: &mut EventTypeRegistry<Self>) {
		registry.register_type_size::<u128>("Balance");
		registry.register_type_size::<u128>("RingBalance");
		registry.register_type_size::<u128>("KtonBalance");
		registry.register_type_size::<[u8; 20]>("EthereumAddress");
		registry.register_type_size::<[u8; 20]>("EcdsaAddress");
		registry.register_type_size::<H256>("MMRRoot");
		registry.register_type_size::<[u8; 32]>("RelayAuthorityMessage");
		registry.register_type_size::<[u8; 20]>("RelayAuthoritySigner");
		registry.register_type_size::<EcdsaSignature>("RelayAuthoritySignature");
		registry.register_type_size::<u8>("ElectionCompute"); // just a hack
		registry.register_type_size::<u32>("Term");
		registry.register_type_size::<u64>("EthereumTransactionIndex");
		registry.register_type_size::<(u64, u32, u32)>("RelayAffirmationId");
		registry.register_type_size::<u32>("EraIndex");
		registry.register_type_size::<u64>("EthereumBlockNumber");
		registry.register_type_size::<Self::BlockNumber>("BlockNumber");
		registry.register_type_size::<Self::AccountId>("AccountId");
		registry.register_type_size::<SessionIndex>("SessionIndex");
		registry.register_type_size::<Log>("Log");
		registry.register_type_size::<U256>("U256");
		registry.register_type_size::<H256>("H256");
		registry.register_type_size::<H160>("H160");
		registry.register_type_size::<ExitReason>("ExitReason");
		register_default_type_sizes(registry);
	}
}

impl Balances for DarwiniaRuntime {
	type Balance = u128;
}

impl System for DarwiniaRuntime {
	type Index = u32;
	type BlockNumber = u32;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Address = MultiAddress<Self::AccountId, ()>;
	type Header = Header<Self::BlockNumber, BlakeTwo256>;
	type Extrinsic = OpaqueExtrinsic;
	type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl TechnicalCommittee for DarwiniaRuntime {}
impl Sudo for DarwiniaRuntime {}
impl EthereumRelay for DarwiniaRuntime {
	type RingBalance = u128;
	type EthereumBlockNumber = u64;
	type PendingRelayHeaderParcel = (
		<Self as System>::BlockNumber,
		EthereumRelayHeaderParcel,
		RelayVotingState<<Self as System>::AccountId>,
	);
	type RelayAffirmationId = RelayAffirmationId<Self::EthereumBlockNumber>;
}

impl EthereumRelayHelper for DarwiniaRuntime {
    fn get_pending_relay_header_number(parcel: Self::PendingRelayHeaderParcel)
        -> Self::EthereumBlockNumber {
            parcel.1.header.number
        }
}

impl EthereumRelayerGame for DarwiniaRuntime {
	type RelayAffirmation = RelayAffirmation<
		EthereumRelayHeaderParcel,
		<Self as System>::AccountId,
		<Self as Balances>::Balance,
		RelayAffirmationId<u64>,
	>;
}

impl EthereumBacking for DarwiniaRuntime {
	type EthereumTransactionIndex = u64;
}

impl RuntimeExt for DarwiniaRuntime {
	type VerifierHandler = DarwiniaMainnetVerifier<Self>;
	type RedeemErc20Handler = DarwiniaMainnetErc20Redeemer<Self>;
	type RegisterErc20Handler = DarwiniaMainnetErc20Register<Self>;
}

impl Proxy for DarwiniaRuntime {
	type ProxyType = ProxyType;
}

impl EthereumRelayAuthorities for DarwiniaRuntime {
	type RelayAuthority = RelayAuthority<
		<Self as System>::AccountId,
		Self::RelayAuthoritySigner,
		<Self as Balances>::Balance,
		<Self as System>::BlockNumber,
	>;
	type RelayAuthoritySigner = EcdsaAddress;
	type RelayAuthoritySignature = EcdsaSignature;
	type RelayAuthorityMessage = EcdsaMessage;
}

#[derive(Encode, Decode)]
enum ExitReason {
	Succeed(ExitSucceed),
	Error(ExitError),
	Revert(ExitRevert),
	Fatal(ExitFatal),
}

#[derive(Encode, Decode)]
enum ExitSucceed {
	Stopped,
	Returned,
	Suicided,
}

#[derive(Encode, Decode)]
enum ExitError {
	StackUnderflow,
	StackOverflow,
	InvalidJump,
	InvalidRange,
	DesignatedInvalid,
	CallTooDeep,
	CreateCollision,
	CreateContractLimit,
	OutOfOffset,
	OutOfGas,
	OutOfFund,
	PCUnderflow,
	CreateEmpty,
	Other(String),
}

#[derive(Encode, Decode)]
enum ExitRevert {
	Reverted,
}

#[derive(Encode, Decode)]
enum ExitFatal {
	NotSupported,
	UnhandledInterrupt,
	CallErrorAsFatal(ExitError),
	Other(String),
}
