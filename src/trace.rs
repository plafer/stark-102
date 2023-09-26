use crate::field::BaseField;

/// First element of the trace, as defined by the statement to prove.
pub const TRACE_FIRST_ELEMENT: BaseField = BaseField::new(3);

/// The trace is 4 elements long so that we can use a small subgroup as domain,
/// and also be able to extend it to a domain of size 8
pub fn generate_trace() -> Vec<BaseField> {
    let mut out_trace = vec![TRACE_FIRST_ELEMENT];
    let mut last_ele = TRACE_FIRST_ELEMENT;

    for _i in 0..3 {
        last_ele = last_ele.square();
        out_trace.push(last_ele);
    }

    out_trace
}
