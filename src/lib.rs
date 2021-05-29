#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::ensure_signed;

use sp_runtime::traits::{IdentifyAccount};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Public: IdentifyAccount<AccountId = Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;

		pub OwnerOf get(fn owner_of): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;

		// attribute_name, identity, attribute_provider => value
		pub AttributeOf get(fn attribute_of): map hasher(blake2_128_concat) (Vec<u8>, Vec<u8>, T::AccountId) => Vec<u8>;

		// identity => delegates
		pub DelegateOf get(fn delegate_of): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;
		
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewIdentity(Vec<u8>, AccountId),
		IdentityTransfered(Vec<u8>, AccountId, AccountId),
		AttributeAdded(Vec<u8>, Vec<u8>, AccountId, Vec<u8>),
		DelegateAdded(Vec<u8>, AccountId),
		DelegateRemoved(Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		IdentityNotOwnedByUser,
		IdentityAlreadyClaimed,
		IdentityNotClaimed,
		InvalidIdentity,
		AttributeNotFound
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 0]
		pub fn claim_identity(origin, identity: Vec<u8>) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			ensure!(!<OwnerOf<T>>::contains_key(&identity), Error::<T>::IdentityAlreadyClaimed);

			<OwnerOf<T>>::insert(&identity, &who);

			// Emit an event.
			Self::deposit_event(RawEvent::NewIdentity(identity, who));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = 0]
		pub fn transfer_identity(
			origin, 
			identity: Vec<u8>, 
			to: T::AccountId) -> dispatch::DispatchResult {

			let who = ensure_signed(origin)?;

			ensure!(<OwnerOf<T>>::get(&identity) == who, Error::<T>::IdentityNotOwnedByUser);
			ensure!(<OwnerOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);

			<OwnerOf<T>>::insert(&identity, &to);

			Self::deposit_event(RawEvent::IdentityTransfered(identity, who, to));
			Ok(())
		}

		#[weight = 0]
		pub fn add_attribute(
			origin, 
			identity: Vec<u8>, 
			name: Vec<u8>, 
			value: Vec<u8>) -> dispatch::DispatchResult {

			let who = ensure_signed(origin)?;
			ensure!(<OwnerOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);
			ensure!(<OwnerOf<T>>::get(&identity) == who, Error::<T>::IdentityNotOwnedByUser);

			<AttributeOf<T>>::insert((&name, &identity, &who), &value);

			Self::deposit_event(RawEvent::AttributeAdded(name, identity, who, value));

			Ok(())
		}

		#[weight = 0]
		pub fn verify_attribute(
			_origin, 
			from: T::AccountId, 
			to: Vec<u8>, 
			name: Vec<u8>) -> dispatch::DispatchResult {

			ensure!(<OwnerOf<T>>::contains_key(&to), Error::<T>::InvalidIdentity);

			ensure!(<AttributeOf<T>>::contains_key((&name, &to, &from)), Error::<T>::AttributeNotFound);

			<AttributeOf<T>>::get((&name, &to, &from));

			Ok(())
		}

		#[weight = 0]
		// only owner of identity can add delegates
		pub fn add_delegate(
			origin,
			delegate: T::AccountId,
			identity: Vec<u8>
		) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<OwnerOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);
			ensure!(<OwnerOf<T>>::get(&identity) == who, Error::<T>::IdentityNotOwnedByUser);

			// match <OwnerOf<T>>::get(&identity) {
			// 	None => Err(Error::<T>::IdentityNotClaimed),
			// 	Some(owner) => {
			// 		ensure!(owner == &who, Error::<T>::IdentityNotOwnedByUser)
			// 	}
			// }

			ensure!(<DelegateOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);
			
			// let delegates: Vec<T::AccountId> = <DelegateOf<T>>::get(&identity);

			// delegates.push(&who);
			<DelegateOf<T>>::insert(&identity, &delegate);

			// match <DelegateOf<T>>::get(&identity){
			// 	None => <DelegateOf<T>>::insert(&identity, vec!(&who)),
			// 	Some(delegates) => {
			// 		delegates.push(&identity);
			// 		<DelegateOf<T>>::insert(&identity, &delegates);
			// 	}
			// }

			Self::deposit_event(RawEvent::DelegateAdded(identity, delegate));
			Ok(())
		}

		#[weight = 0]
		// only owner of identity can add delegates
		pub fn remove_delegate(
			origin,
			identity: Vec<u8>
		) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<OwnerOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);
			ensure!(<OwnerOf<T>>::get(&identity) == who, Error::<T>::IdentityNotOwnedByUser);


			ensure!(<DelegateOf<T>>::contains_key(&identity), Error::<T>::IdentityNotClaimed);

			<DelegateOf<T>>::remove(&identity);

			Self::deposit_event(RawEvent::DelegateRemoved(identity));
			Ok(())
		}

		#[weight = 0]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}
	}
}
