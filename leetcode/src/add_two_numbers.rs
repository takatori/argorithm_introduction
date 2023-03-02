// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

struct Solution {}

impl Solution {
    pub fn add_two_numbers(
        l1: Option<Box<ListNode>>,
        l2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut carry: i32 = 0;
        let start = Box::new(ListNode::new(0));
        let mut current = start;

        let mut l1_p = &mut l1;
        let mut l2_p = &mut l2;

        while l1_p.is_some() || l2_p.is_some() {
            let a_opt = match (l1_p, l2_p) {
                (Some(n1), Some(n2)) => {
                    l1_p = &mut n1.next;
                    l2_p = &mut n2.next;
                    Some(n1.val + n2.val + carry)
                }
                (Some(n1), None) => {
                    l1_p = &mut n1.next;
                    Some(n1.val + carry)
                }
                (None, Some(n2)) => {
                    l2_p = &mut n2.next;
                    Some(n2.val + carry)
                }
                (None, None) => None,
            };

            if let Some(a) = a_opt {
                carry = a / 10;
                let n = Box::new(ListNode::new(a % 10));
                current.next = Some(n);
                current = n;
            }
        }

        start.next
    }
}
