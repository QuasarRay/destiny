//! Arbitrary-schedule ownership protocol for a bounded machine refcount.
//! Tokens are logical allocation permissions. Their association with the C
//! pointer/Box, standard-atomic linearization and callers remains a declared
//! concrete correspondence obligation, not a claim supplied by this model.
use vstd::prelude::*;
use vstd::multiset::*;
use verus_state_machines_macros::tokenized_state_machine;

tokenized_state_machine! { runtime_references {
    fields {
        #[sharding(storage_option)]
        pub allocation: Option<nat>,
        #[sharding(variable)]
        pub count: usize,
        #[sharding(multiset)]
        pub references: Multiset<nat>,
    }

    init! {
        new(allocation: nat) {
            init allocation = Some(allocation);
            init count = 1;
            init references = Multiset::singleton(allocation);
        }
    }

    transition! {
        retain(allocation: nat) {
            have references >= { allocation };
            require pre.count < usize::MAX;
            assert pre.count > 0;
            update count = (pre.count + 1) as usize;
            add references += { allocation };
        }
    }

    transition! {
        reject_retain_at_limit(allocation: nat) {
            have references >= { allocation };
            require pre.count == usize::MAX;
        }
    }

    transition! {
        release_nonfinal(allocation: nat) {
            remove references -= { allocation };
            require pre.count > 1;
            update count = (pre.count - 1) as usize;
        }
    }

    transition! {
        release_final(allocation: nat) {
            remove references -= { allocation };
            require pre.count == 1;
            update count = 0;
            withdraw allocation -= Some(allocation);
        }
    }

    transition! {
        failed_compare_exchange() {
            // A failed/spurious CAS changes no shared state or ownership.
        }
    }

    property! {
        a_live_reference_guards_its_allocation(allocation: nat) {
            have references >= { allocation };
            guard allocation >= Some(allocation);
        }
    }

    #[invariant]
    pub spec fn ownership(&self) -> bool {
        match self.allocation {
            None => self.count == 0 && self.references =~= Multiset::empty(),
            Some(allocation) => {
                &&& self.count > 0
                &&& self.references.count(allocation) == self.count as nat
                &&& forall |other: nat| other != allocation ==> self.references.count(other) == 0
            }
        }
    }

    #[inductive(new)]
    fn new_inductive(post: Self, allocation: nat) { }
    #[inductive(retain)]
    fn retain_inductive(pre: Self, post: Self, allocation: nat) { }
    #[inductive(reject_retain_at_limit)]
    fn reject_retain_at_limit_inductive(pre: Self, post: Self, allocation: nat) { }
    #[inductive(release_nonfinal)]
    fn release_nonfinal_inductive(pre: Self, post: Self, allocation: nat) { }
    #[inductive(release_final)]
    fn release_final_inductive(pre: Self, post: Self, allocation: nat) { }
    #[inductive(failed_compare_exchange)]
    fn failed_compare_exchange_inductive(pre: Self, post: Self) { }
} }
