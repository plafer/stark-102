use crate::field::BaseField;

/// The trace is 4 elements long so that we can use a small subgroup as domain,
/// and also be able to extend it to a domain of size 8
pub fn generate_trace() -> Vec<BaseField> {
    let first_ele: BaseField = 3.into();
    let mut out_trace = vec![first_ele];
    let mut last_ele = first_ele;

    for _i in 0..3 {
        last_ele = last_ele.square();
        out_trace.push(last_ele);
    }

    out_trace
}
