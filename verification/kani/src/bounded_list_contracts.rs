//! Constructor/public mutator invariants for the shared bounded-list adapter.
//! This is a structural obligation, independent of original scenario credit.
use destiny_original_spec::BoundedList;

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(17)]
fn public_mutators_preserve_checked_capacity_and_order() {
    let mut list=BoundedList::<u8,3>::new();
    assert!(list.len()==0);
    // Every valid capacity-three state is represented by arbitrary prefix
    // values and any length 0..=3. Proving one arbitrary mutator preserves
    // this state space supplies the inductive step for sequences of any
    // length; the zero-state constructor supplies the base. This avoids
    // symbolic execution's exponential enumeration of twelve-step traces.
    let length=kani::any::<u8>()%4;
    let mut i=0;
    while i<usize::from(length) { assert!(list.push(kani::any::<u8>()));i+=1; }
    {
        let operation:u8=kani::any();
        let value:u8=kani::any();
        let index:usize=kani::any();
        let before=list;
        match operation%4 {
            0=>{
                let success=list.push(value);
                assert!(success==(before.len()<3));
                assert!(list.len()==before.len()+usize::from(success));
                if success { assert!(list.get(before.len())==Some(value)); }
                let mut i=0;while i<before.len() { assert!(list.get(i)==before.get(i));i+=1; }
            },
            1=>{
                let success=list.insert(index,value);
                assert!(success==(index<=before.len() && before.len()<3));
                assert!(list.len()==before.len()+usize::from(success));
                if success {
                    assert!(list.get(index)==Some(value));
                    let mut i=0;while i<before.len() {
                        assert!(list.get(if i<index {i} else {i+1})==before.get(i));i+=1;
                    }
                }else { assert!(list==before); }
            },
            2=>{
                let success=list.set(index,value);
                assert!(success==(index<before.len()));
                assert!(list.len()==before.len());
                let mut i=0;while i<before.len() {
                    assert!(list.get(i)==if i==index { Some(value) }else {before.get(i)});i+=1;
                }
                if !success { assert!(list==before); }
            },
            _=>{
                let mut tail=BoundedList::<u8,3>::new();assert!(tail.push(value));
                let success=list.extend(&tail);
                assert!(success==(before.len()<3));
                assert!(list.len()==before.len()+usize::from(success));
                if success { assert!(list.get(before.len())==Some(value)); }
                let mut i=0;while i<before.len() { assert!(list.get(i)==before.get(i));i+=1; }
            },
        }
        assert!(list.len()<=3);
        assert!(list.get(list.len()).is_none());
    }
    kani::cover!(list.len()==3,"arbitrary valid pre-state/mutator reaches a full bounded list");
    kani::cover!(list.len()==0,"arbitrary valid pre-state/mutator preserves an empty bounded list");
}
