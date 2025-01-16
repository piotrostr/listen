use crate::serum_dex::{
    error::{DexErrorCode, DexResult},
    fees::FeeTier,
};
use arrayref::{array_refs, mut_array_refs};
use bytemuck::{
    cast, cast_mut, cast_ref, cast_slice, cast_slice_mut, Pod, Zeroable,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    convert::{identity, TryFrom},
    mem::{align_of, size_of},
    num::NonZeroU64,
};

pub type NodeHandle = u32;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum NodeTag {
    Uninitialized = 0,
    InnerNode = 1,
    LeafNode = 2,
    FreeNode = 3,
    LastFreeNode = 4,
}

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
struct InnerNode {
    tag: u32,
    prefix_len: u32,
    key: u128,
    children: [u32; 2],
    _padding: [u64; 5],
}
unsafe impl Zeroable for InnerNode {}
unsafe impl Pod for InnerNode {}

impl InnerNode {
    fn walk_down(&self, search_key: u128) -> (NodeHandle, bool) {
        let crit_bit_mask = (1u128 << 127) >> self.prefix_len;
        let crit_bit = (search_key & crit_bit_mask) != 0;
        (self.children[crit_bit as usize], crit_bit)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(packed)]
pub struct LeafNode {
    tag: u32,
    owner_slot: u8,
    fee_tier: u8,
    padding: [u8; 2],
    key: u128,
    owner: [u64; 4],
    quantity: u64,
    client_order_id: u64,
}
unsafe impl Zeroable for LeafNode {}
unsafe impl Pod for LeafNode {}

impl LeafNode {
    #[inline]
    pub fn new(
        owner_slot: u8,
        key: u128,
        owner: [u64; 4],
        quantity: u64,
        fee_tier: FeeTier,
        client_order_id: u64,
    ) -> Self {
        LeafNode {
            tag: NodeTag::LeafNode.into(),
            owner_slot,
            fee_tier: fee_tier.into(),
            padding: [0; 2],
            key,
            owner,
            quantity,
            client_order_id,
        }
    }

    #[inline]
    pub fn fee_tier(&self) -> FeeTier {
        FeeTier::try_from_primitive(self.fee_tier).unwrap()
    }

    #[inline]
    pub fn price(&self) -> NonZeroU64 {
        NonZeroU64::new((self.key >> 64) as u64).unwrap()
    }

    #[inline]
    pub fn order_id(&self) -> u128 {
        self.key
    }

    #[inline]
    pub fn quantity(&self) -> u64 {
        self.quantity
    }

    #[inline]
    pub fn set_quantity(&mut self, quantity: u64) {
        self.quantity = quantity;
    }

    #[inline]
    pub fn owner(&self) -> [u64; 4] {
        self.owner
    }

    #[inline]
    pub fn owner_slot(&self) -> u8 {
        self.owner_slot
    }

    #[inline]
    pub fn client_order_id(&self) -> u64 {
        self.client_order_id
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
struct FreeNode {
    tag: u32,
    next: u32,
    _padding: [u64; 8],
}
unsafe impl Zeroable for FreeNode {}
unsafe impl Pod for FreeNode {}

const fn _const_max(a: usize, b: usize) -> usize {
    let gt = (a > b) as usize;
    gt * a + (1 - gt) * b
}

const _INNER_NODE_SIZE: usize = size_of::<InnerNode>();
const _LEAF_NODE_SIZE: usize = size_of::<LeafNode>();
const _FREE_NODE_SIZE: usize = size_of::<FreeNode>();
const _NODE_SIZE: usize = 72;

const _INNER_NODE_ALIGN: usize = align_of::<InnerNode>();
const _LEAF_NODE_ALIGN: usize = align_of::<LeafNode>();
const _FREE_NODE_ALIGN: usize = align_of::<FreeNode>();
const _NODE_ALIGN: usize = 1;

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
pub struct AnyNode {
    tag: u32,
    data: [u32; 17],
}
unsafe impl Zeroable for AnyNode {}
unsafe impl Pod for AnyNode {}

enum NodeRef<'a> {
    Inner(&'a InnerNode),
    Leaf(&'a LeafNode),
}

enum NodeRefMut<'a> {
    Inner(&'a mut InnerNode),
    Leaf(&'a mut LeafNode),
}

impl AnyNode {
    fn key(&self) -> Option<u128> {
        match self.case()? {
            NodeRef::Inner(inner) => Some(inner.key),
            NodeRef::Leaf(leaf) => Some(leaf.key),
        }
    }

    fn prefix_len(&self) -> u32 {
        match self.case().unwrap() {
            NodeRef::Inner(&InnerNode { prefix_len, .. }) => prefix_len,
            NodeRef::Leaf(_) => 128,
        }
    }

    fn children(&self) -> Option<[u32; 2]> {
        match self.case().unwrap() {
            NodeRef::Inner(&InnerNode { children, .. }) => Some(children),
            NodeRef::Leaf(_) => None,
        }
    }

    fn case(&self) -> Option<NodeRef> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRef::Inner(cast_ref(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRef::Leaf(cast_ref(self))),
            _ => None,
        }
    }

    fn case_mut(&mut self) -> Option<NodeRefMut> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRefMut::Inner(cast_mut(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRefMut::Leaf(cast_mut(self))),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf(&self) -> Option<&LeafNode> {
        match self.case() {
            Some(NodeRef::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode> {
        match self.case_mut() {
            Some(NodeRefMut::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }
}

impl AsRef<AnyNode> for InnerNode {
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

impl AsRef<AnyNode> for LeafNode {
    #[inline]
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
struct SlabHeader {
    bump_index: u64,
    free_list_len: u64,
    free_list_head: u32,

    root_node: u32,
    leaf_count: u64,
}
unsafe impl Zeroable for SlabHeader {}
unsafe impl Pod for SlabHeader {}

const SLAB_HEADER_LEN: usize = size_of::<SlabHeader>();

#[cfg(debug_assertions)]
unsafe fn invariant(check: bool) {
    if check {
        unreachable!();
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
unsafe fn invariant(check: bool) {
    if check {
        std::hint::unreachable_unchecked();
    }
}

#[repr(transparent)]
pub struct Slab([u8]);

impl Slab {
    /// Creates a slab that holds and references the bytes
    ///
    /// ```compile_fail
    /// let slab = {
    ///     let mut bytes = [10; 100];
    ///     serum_dex::critbit::Slab::new(&mut bytes)
    /// };
    /// ```
    #[inline]
    pub fn new(bytes: &mut [u8]) -> &mut Self {
        let len_without_header =
            bytes.len().checked_sub(SLAB_HEADER_LEN).unwrap();
        let slop = len_without_header % size_of::<AnyNode>();
        let truncated_len = bytes.len() - slop;
        let bytes = &mut bytes[..truncated_len];
        let slab: &mut Self =
            unsafe { &mut *(bytes as *mut [u8] as *mut Slab) };
        slab.check_size_align(); // check alignment
        slab
    }

    #[inline]
    pub fn new_check(bytes: &[u8]) -> &Self {
        let len_without_header =
            bytes.len().checked_sub(SLAB_HEADER_LEN).unwrap();
        let slop = len_without_header % size_of::<AnyNode>();
        let truncated_len = bytes.len() - slop;
        let bytes = &bytes[..truncated_len];
        let slab: &Self = unsafe { &*(bytes as *const [u8] as *const Slab) };
        slab.check_size_align(); // check alignment
        slab
    }

    #[inline]
    pub fn assert_minimum_capacity(&self, capacity: u32) -> DexResult {
        if self.nodes().len() <= (capacity as usize) * 2 {
            Err(DexErrorCode::SlabTooSmall)?
        }
        Ok(())
    }

    fn check_size_align(&self) {
        let (header_bytes, nodes_bytes) =
            array_refs![&self.0, SLAB_HEADER_LEN; .. ;];
        let _header: &SlabHeader = cast_ref(header_bytes);
        let _nodes: &[AnyNode] = cast_slice(nodes_bytes);
    }

    fn parts(&self) -> (&SlabHeader, &[AnyNode]) {
        unsafe {
            invariant(self.0.len() < size_of::<SlabHeader>());
            invariant(
                (self.0.as_ptr() as usize) % align_of::<SlabHeader>() != 0,
            );
            invariant(
                ((self.0.as_ptr() as usize) + size_of::<SlabHeader>())
                    % align_of::<AnyNode>()
                    != 0,
            );
        }

        let (header_bytes, nodes_bytes) =
            array_refs![&self.0, SLAB_HEADER_LEN; .. ;];
        let header = cast_ref(header_bytes);
        let nodes = cast_slice(nodes_bytes);
        (header, nodes)
    }

    fn parts_mut(&mut self) -> (&mut SlabHeader, &mut [AnyNode]) {
        unsafe {
            invariant(self.0.len() < size_of::<SlabHeader>());
            invariant(
                (self.0.as_ptr() as usize) % align_of::<SlabHeader>() != 0,
            );
            invariant(
                ((self.0.as_ptr() as usize) + size_of::<SlabHeader>())
                    % align_of::<AnyNode>()
                    != 0,
            );
        }

        let (header_bytes, nodes_bytes) =
            mut_array_refs![&mut self.0, SLAB_HEADER_LEN; .. ;];
        let header = cast_mut(header_bytes);
        let nodes = cast_slice_mut(nodes_bytes);
        (header, nodes)
    }

    fn header(&self) -> &SlabHeader {
        self.parts().0
    }

    fn header_mut(&mut self) -> &mut SlabHeader {
        self.parts_mut().0
    }

    fn nodes(&self) -> &[AnyNode] {
        self.parts().1
    }

    fn nodes_mut(&mut self) -> &mut [AnyNode] {
        self.parts_mut().1
    }
}

pub trait SlabView<T> {
    fn capacity(&self) -> u64;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn get(&self, h: NodeHandle) -> Option<&T>;
    fn get_mut(&mut self, h: NodeHandle) -> Option<&mut T>;
    fn insert(&mut self, val: &T) -> Result<u32, ()>;
    fn remove(&mut self, h: NodeHandle) -> Option<T>;
    fn contains(&self, h: NodeHandle) -> bool;
}

impl SlabView<AnyNode> for Slab {
    fn capacity(&self) -> u64 {
        self.nodes().len() as u64
    }

    fn clear(&mut self) {
        let (header, _nodes) = self.parts_mut();
        *header = SlabHeader {
            bump_index: 0,
            free_list_len: 0,
            free_list_head: 0,

            root_node: 0,
            leaf_count: 0,
        }
    }

    fn is_empty(&self) -> bool {
        let SlabHeader {
            bump_index,
            free_list_len,
            ..
        } = *self.header();
        bump_index == free_list_len
    }

    fn get(&self, key: u32) -> Option<&AnyNode> {
        let node = self.nodes().get(key as usize)?;
        let tag = NodeTag::try_from(node.tag);
        match tag {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => Some(node),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: u32) -> Option<&mut AnyNode> {
        let node = self.nodes_mut().get_mut(key as usize)?;
        let tag = NodeTag::try_from(node.tag);
        match tag {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => Some(node),
            _ => None,
        }
    }

    fn insert(&mut self, val: &AnyNode) -> Result<u32, ()> {
        match NodeTag::try_from(identity(val.tag)) {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => (),
            _ => unreachable!(),
        };

        let (header, nodes) = self.parts_mut();

        if header.free_list_len == 0 {
            if header.bump_index as usize == nodes.len() {
                return Err(());
            }

            if header.bump_index == std::u32::MAX as u64 {
                return Err(());
            }
            let key = header.bump_index as u32;
            header.bump_index += 1;

            nodes[key as usize] = *val;
            return Ok(key);
        }

        let key = header.free_list_head;
        let node = &mut nodes[key as usize];

        match NodeTag::try_from(node.tag) {
            Ok(NodeTag::FreeNode) => assert!(header.free_list_len > 1),
            Ok(NodeTag::LastFreeNode) => {
                assert_eq!(identity(header.free_list_len), 1)
            }
            _ => unreachable!(),
        };

        let next_free_list_head: u32;
        {
            let free_list_item: &FreeNode = cast_ref(node);
            next_free_list_head = free_list_item.next;
        }
        header.free_list_head = next_free_list_head;
        header.free_list_len -= 1;
        *node = *val;
        Ok(key)
    }

    fn remove(&mut self, key: u32) -> Option<AnyNode> {
        let val = *self.get(key)?;
        let (header, nodes) = self.parts_mut();
        let any_node_ref = &mut nodes[key as usize];
        let free_node_ref: &mut FreeNode = cast_mut(any_node_ref);
        *free_node_ref = FreeNode {
            tag: if header.free_list_len == 0 {
                NodeTag::LastFreeNode.into()
            } else {
                NodeTag::FreeNode.into()
            },
            next: header.free_list_head,
            _padding: Zeroable::zeroed(),
        };
        header.free_list_len += 1;
        header.free_list_head = key;
        Some(val)
    }

    fn contains(&self, key: u32) -> bool {
        self.get(key).is_some()
    }
}

#[derive(Debug)]
pub enum SlabTreeError {
    OutOfSpace,
}

impl Slab {
    fn root(&self) -> Option<NodeHandle> {
        if self.header().leaf_count == 0 {
            return None;
        }

        Some(self.header().root_node)
    }

    fn find_min_max(&self, find_max: bool) -> Option<NodeHandle> {
        let mut root: NodeHandle = self.root()?;
        loop {
            let root_contents = self.get(root).unwrap();
            match root_contents.case().unwrap() {
                NodeRef::Inner(&InnerNode { children, .. }) => {
                    root = children[if find_max { 1 } else { 0 }];
                    continue;
                }
                _ => return Some(root),
            }
        }
    }

    #[inline]
    pub fn find_min(&self) -> Option<NodeHandle> {
        self.find_min_max(false)
    }

    #[inline]
    pub fn find_max(&self) -> Option<NodeHandle> {
        self.find_min_max(true)
    }

    #[inline]
    pub fn insert_leaf(
        &mut self,
        new_leaf: &LeafNode,
    ) -> Result<(NodeHandle, Option<LeafNode>), SlabTreeError> {
        let mut root: NodeHandle = match self.root() {
            Some(h) => h,
            None => {
                // create a new root if none exists
                match self.insert(new_leaf.as_ref()) {
                    Ok(handle) => {
                        self.header_mut().root_node = handle;
                        self.header_mut().leaf_count = 1;
                        return Ok((handle, None));
                    }
                    Err(()) => return Err(SlabTreeError::OutOfSpace),
                }
            }
        };
        loop {
            // check if the new node will be a child of the root
            let root_contents = *self.get(root).unwrap();
            let root_key = root_contents.key().unwrap();
            if root_key == new_leaf.key {
                if let Some(NodeRef::Leaf(&old_root_as_leaf)) =
                    root_contents.case()
                {
                    // clobber the existing leaf
                    *self.get_mut(root).unwrap() = *new_leaf.as_ref();
                    return Ok((root, Some(old_root_as_leaf)));
                }
            }
            let shared_prefix_len: u32 =
                (root_key ^ new_leaf.key).leading_zeros();
            match root_contents.case() {
                None => unreachable!(),
                Some(NodeRef::Inner(inner)) => {
                    let keep_old_root = shared_prefix_len >= inner.prefix_len;
                    if keep_old_root {
                        root = inner.walk_down(new_leaf.key).0;
                        continue;
                    };
                }
                _ => (),
            };

            // change the root in place to represent the LCA of [new_leaf] and [root]
            let crit_bit_mask: u128 = (1u128 << 127) >> shared_prefix_len;
            let new_leaf_crit_bit = (crit_bit_mask & new_leaf.key) != 0;
            let old_root_crit_bit = !new_leaf_crit_bit;

            let new_leaf_handle = self
                .insert(new_leaf.as_ref())
                .map_err(|()| SlabTreeError::OutOfSpace)?;
            let moved_root_handle = match self.insert(&root_contents) {
                Ok(h) => h,
                Err(()) => {
                    self.remove(new_leaf_handle).unwrap();
                    return Err(SlabTreeError::OutOfSpace);
                }
            };

            let new_root: &mut InnerNode =
                cast_mut(self.get_mut(root).unwrap());
            *new_root = InnerNode {
                tag: NodeTag::InnerNode.into(),
                prefix_len: shared_prefix_len,
                key: new_leaf.key,
                children: [0; 2],
                _padding: Zeroable::zeroed(),
            };

            new_root.children[new_leaf_crit_bit as usize] = new_leaf_handle;
            new_root.children[old_root_crit_bit as usize] = moved_root_handle;
            self.header_mut().leaf_count += 1;
            return Ok((new_leaf_handle, None));
        }
    }

    pub fn find_by_key(&self, search_key: u128) -> Option<NodeHandle> {
        let mut node_handle: NodeHandle = self.root()?;
        loop {
            let node_ref = self.get(node_handle).unwrap();
            let node_prefix_len = node_ref.prefix_len();
            let node_key = node_ref.key().unwrap();
            let common_prefix_len = (search_key ^ node_key).leading_zeros();
            if common_prefix_len < node_prefix_len {
                return None;
            }
            match node_ref.case().unwrap() {
                NodeRef::Leaf(_) => break Some(node_handle),
                NodeRef::Inner(inner) => {
                    let crit_bit_mask = (1u128 << 127) >> node_prefix_len;
                    let _search_key_crit_bit =
                        (search_key & crit_bit_mask) != 0;
                    node_handle = inner.walk_down(search_key).0;
                    continue;
                }
            }
        }
    }

    pub(crate) fn find_by<F: Fn(&LeafNode) -> bool>(
        &self,
        limit: &mut u16,
        predicate: F,
    ) -> Vec<u128> {
        let mut found = Vec::new();
        let mut nodes_to_search: Vec<NodeHandle> = Vec::new();
        let mut current_node: Option<&AnyNode>;

        let top_node = self.root();

        // No found nodes.
        if top_node.is_none() {
            return found;
        }

        nodes_to_search.push(top_node.unwrap());

        // Search through the tree.
        while !nodes_to_search.is_empty() && *limit > 0 {
            *limit -= 1;

            current_node = self.get(nodes_to_search.pop().unwrap());

            // Node not found.
            if current_node.is_none() {
                break;
            }

            match current_node.unwrap().case().unwrap() {
                NodeRef::Leaf(leaf) if predicate(leaf) => {
                    // Found a matching leaf.
                    found.push(leaf.key)
                }
                NodeRef::Inner(inner) => {
                    // Search the children.
                    nodes_to_search.push(inner.children[0]);
                    nodes_to_search.push(inner.children[1]);
                }
                _ => (),
            }
        }

        found
    }

    #[inline]
    pub fn remove_by_key(&mut self, search_key: u128) -> Option<LeafNode> {
        let mut parent_h = self.root()?;
        let mut child_h;
        let mut crit_bit;
        match self.get(parent_h).unwrap().case().unwrap() {
            NodeRef::Leaf(&leaf) if leaf.key == search_key => {
                let header = self.header_mut();
                assert_eq!(identity(header.leaf_count), 1);
                header.root_node = 0;
                header.leaf_count = 0;
                let _old_root = self.remove(parent_h).unwrap();
                return Some(leaf);
            }
            NodeRef::Leaf(_) => return None,
            NodeRef::Inner(inner) => {
                let (ch, cb) = inner.walk_down(search_key);
                child_h = ch;
                crit_bit = cb;
            }
        }
        loop {
            match self.get(child_h).unwrap().case().unwrap() {
                NodeRef::Inner(inner) => {
                    let (grandchild_h, grandchild_crit_bit) =
                        inner.walk_down(search_key);
                    parent_h = child_h;
                    child_h = grandchild_h;
                    crit_bit = grandchild_crit_bit;
                    continue;
                }
                NodeRef::Leaf(&leaf) => {
                    if leaf.key != search_key {
                        return None;
                    }

                    break;
                }
            }
        }
        // replace parent with its remaining child node
        // free child_h, replace *parent_h with *other_child_h, free other_child_h
        let other_child_h = self.get(parent_h).unwrap().children().unwrap()
            [!crit_bit as usize];
        let other_child_node_contents = self.remove(other_child_h).unwrap();
        *self.get_mut(parent_h).unwrap() = other_child_node_contents;
        self.header_mut().leaf_count -= 1;
        Some(cast(self.remove(child_h).unwrap()))
    }

    #[inline]
    pub fn remove_min(&mut self) -> Option<LeafNode> {
        self.remove_by_key(self.get(self.find_min()?)?.key()?)
    }

    #[inline]
    pub fn remove_max(&mut self) -> Option<LeafNode> {
        self.remove_by_key(self.get(self.find_max()?)?.key()?)
    }

    pub fn traverse_orders(
        &self,
        filter_owner: Option<[u64; 4]>,
    ) -> Vec<LeafNode> {
        fn walk_rec<'a>(
            slab: &'a Slab,
            sub_root: NodeHandle,
            buf: &mut Vec<LeafNode>,
            filter_owner: Option<[u64; 4]>,
        ) {
            match slab.get(sub_root).unwrap().case().unwrap() {
                NodeRef::Leaf(leaf) => {
                    if let Some(owner) = filter_owner {
                        if leaf.owner() == owner {
                            buf.push(*leaf);
                        }
                    } else {
                        buf.push(*leaf);
                    }
                }
                NodeRef::Inner(inner) => {
                    walk_rec(slab, inner.children[0], buf, filter_owner);
                    walk_rec(slab, inner.children[1], buf, filter_owner);
                }
            }
        }

        let mut buf = Vec::with_capacity(self.header().leaf_count as usize);
        if let Some(r) = self.root() {
            walk_rec(self, r, &mut buf, filter_owner);
        }
        buf
    }
}
