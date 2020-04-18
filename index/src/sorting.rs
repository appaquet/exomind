use exocore_chain::operation::OperationId;
use exocore_core::protos::generated::exocore_index::{sorting_value, Paging, SortingValue};
use std::cmp::Ordering;

/// Wraps a mutation search result's sorting value so that it can be easily reversed when required
/// or ignored if it's outside of the requested paging.
#[derive(Clone, Debug, PartialEq)]
pub struct SortingValueWrapper {
    pub value: SortingValue,
    pub reverse: bool,

    // means that this result should not be returned (probably because it's not withing paging)
    // and should match less than any other non-ignored result
    pub ignore: bool,
}

impl PartialOrd for SortingValueWrapper {
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

impl Ord for SortingValueWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for SortingValueWrapper {}

impl SortingValueWrapper {
    pub fn is_within_bound(&self, lower: &SortingValue, higher: &SortingValue) -> bool {
        self.value.is_after(lower) && self.value.is_before(higher)
    }
}

/// Extensions of sorting value to add comparison methods.
pub trait SortingValueExt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
    fn is_after(&self, other: &Self) -> bool;
    fn is_before(&self, other: &Self) -> bool;
    fn is_within_page_bound(&self, page: &Paging) -> bool;
}

impl SortingValueExt for SortingValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.value == other.value {
            return self.operation_id.partial_cmp(&other.operation_id);
        }

        use sorting_value::Value as V;
        use std::cmp::Ordering as O;
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
        if let Some(std::cmp::Ordering::Greater) = self.partial_cmp(other) {
            true
        } else {
            false
        }
    }

    fn is_before(&self, other: &Self) -> bool {
        if let Some(std::cmp::Ordering::Less) = self.partial_cmp(other) {
            true
        } else {
            false
        }
    }

    fn is_within_page_bound(&self, page: &Paging) -> bool {
        if let Some(before) = page.before_sort_value.as_ref() {
            if !self.is_before(before) {
                return false;
            }
        }

        if let Some(after) = page.after_sort_value.as_ref() {
            if !self.is_after(after) {
                return false;
            }
        }

        true
    }
}

pub fn value_from_u64(value: u64, operation_id: OperationId) -> SortingValue {
    SortingValue {
        value: Some(sorting_value::Value::Uint64(value)),
        operation_id,
    }
}

pub fn value_from_f32(value: f32, operation_id: OperationId) -> SortingValue {
    SortingValue {
        value: Some(sorting_value::Value::Float(value)),
        operation_id,
    }
}

pub fn value_max() -> SortingValue {
    SortingValue {
        value: Some(sorting_value::Value::Max(true)),
        operation_id: 0,
    }
}

pub fn value_min() -> SortingValue {
    SortingValue {
        value: Some(sorting_value::Value::Min(true)),
        operation_id: 0,
    }
}
