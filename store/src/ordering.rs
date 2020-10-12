use exocore_chain::operation::OperationId;
use exocore_core::protos::generated::exocore_store::{ordering_value, OrderingValue, Paging};
use std::cmp::Ordering;

/// Wraps a trait or entities search result's ordering value so that it can be
/// easily reversed when required or ignored if it's outside of the requested
/// paging.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderingValueWrapper {
    pub value: OrderingValue,
    pub reverse: bool,

    // means that this result should not be returned (probably because it's not withing paging)
    // and should match less than any other non-ignored result
    pub ignore: bool,
}

impl PartialOrd for OrderingValueWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // an ignored result should be always be less, unless they are both
        if self.ignore && other.ignore {
            return Some(std::cmp::Ordering::Equal);
        } else if self.ignore {
            return Some(std::cmp::Ordering::Less);
        } else if other.ignore {
            return Some(std::cmp::Ordering::Greater);
        }

        let cmp = self.value.partial_cmp(&other.value);

        // reverse if needed
        if self.reverse {
            cmp.map(|o| o.reverse())
        } else {
            cmp
        }
    }
}

impl Ord for OrderingValueWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for OrderingValueWrapper {}

impl OrderingValueWrapper {
    pub fn is_within_bound(&self, lower: &OrderingValue, higher: &OrderingValue) -> bool {
        self.value.is_after(lower) && self.value.is_before(higher)
    }
}

/// Extensions of ordering value to add comparison methods.
pub trait OrderingValueExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
    fn is_after(&self, other: &Self) -> bool;
    fn is_before(&self, other: &Self) -> bool;
    fn is_within_page_bound(&self, page: &Paging) -> bool;
}

impl OrderingValueExt for OrderingValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use ordering_value::Value as V;
        use std::cmp::Ordering as O;

        if self.value == other.value {
            return self.operation_id.partial_cmp(&other.operation_id);
        }

        match (self.value.as_ref(), other.value.as_ref()) {
            (Some(V::Min(_)), _) => Some(O::Less),
            (_, Some(V::Min(_))) => Some(O::Greater),
            (Some(V::Max(_)), _) => Some(O::Greater),
            (_, Some(V::Max(_))) => Some(O::Less),
            (Some(V::Float(va)), Some(V::Float(vb))) => va.partial_cmp(&vb),
            (Some(V::Uint64(va)), Some(V::Uint64(vb))) => va.partial_cmp(&vb),
            (Some(V::Date(va)), Some(V::Date(vb))) => {
                if va.seconds != vb.seconds {
                    va.seconds.partial_cmp(&vb.seconds)
                } else {
                    va.nanos.partial_cmp(&vb.nanos)
                }
            }
            _other => None,
        }
    }

    fn is_after(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Greater))
    }

    fn is_before(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Less))
    }

    fn is_within_page_bound(&self, page: &Paging) -> bool {
        if let Some(before) = page.before_ordering_value.as_ref() {
            if !self.is_before(before) {
                return false;
            }
        }

        if let Some(after) = page.after_ordering_value.as_ref() {
            if !self.is_after(after) {
                return false;
            }
        }

        true
    }
}

pub fn value_from_u64(value: u64, operation_id: OperationId) -> OrderingValue {
    OrderingValue {
        value: Some(ordering_value::Value::Uint64(value)),
        operation_id,
    }
}

pub fn value_from_f32(value: f32, operation_id: OperationId) -> OrderingValue {
    OrderingValue {
        value: Some(ordering_value::Value::Float(value)),
        operation_id,
    }
}

pub fn value_max() -> OrderingValue {
    OrderingValue {
        value: Some(ordering_value::Value::Max(true)),
        operation_id: 0,
    }
}

pub fn value_min() -> OrderingValue {
    OrderingValue {
        value: Some(ordering_value::Value::Min(true)),
        operation_id: 0,
    }
}
