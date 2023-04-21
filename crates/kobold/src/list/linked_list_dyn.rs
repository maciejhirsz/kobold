use std::ptr::NonNull;
use std::mem::MaybeUninit;
use std::alloc::{alloc, dealloc, Layout};

#[repr(C)]
struct Node<T> {
    /// Pointer to the next `Node` or written length if this is a tail node
    meta: Meta<T>,

    /// All the elements of the `Node` in
    data: [MaybeUninit<T>],
}

union Meta<T> {
    len: usize,
    next: NonNull<Node<T>>,
}

union FatPtr<T> {
    raw: (NonNull<Meta<T>>, usize),
    fat: NonNull<Node<T>>,
}

pub struct LinkedList<T> {
    /// Total number of nodes in the list
    nodes: usize,

    /// First `Node` in the list
    first: NonNull<Node<T>>,
}

impl<T> Node<T> {
    const MIN_PAGE_SIZE: usize = {
        let n = 1024 / std::mem::size_of::<T>();

        if n == 0 {
            1
        } else {
            n
        }
    };

    fn new(cap: usize) -> NonNull<Self> {
        let cap = std::cmp::max(cap, Self::MIN_PAGE_SIZE);

        debug_assert_eq!(
            std::mem::size_of::<(NonNull<Meta<T>>, usize)>(),
            std::mem::size_of::<NonNull<Node<T>>>()
        );

        Vec::<u32>::new().into_boxed_slice();

        unsafe {
            let meta = NonNull::new_unchecked(alloc(Self::layout(cap)) as *mut Meta<T>);

            meta.as_ptr().write(Meta { len: 0 });

            FatPtr { raw: (meta, cap) }.fat
        }
    }

    fn dealloc(ptr: NonNull<Self>) {
        unsafe { dealloc(ptr.as_ptr().cast(), Layout::for_value(ptr.as_ref())) }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    unsafe fn assume_page(&mut self) -> &mut [T] {
        &mut *(&mut self.data as *mut _ as *mut [T])
    }

    unsafe fn assume_slice(&mut self) -> &mut [T] {
        &mut *(&mut self.data[..self.meta.len] as *mut _ as *mut [T])
    }

    fn as_mut<'a>(ptr: NonNull<Self>) -> &'a mut Self {
        unsafe { &mut *ptr.as_ptr() }
    }

    const fn layout(cap: usize) -> Layout {
        use std::mem::{size_of, align_of};

        let mut align = align_of::<Meta<T>>();
        let mut pad = 0;

        if align_of::<T>() > align {
            pad = align_of::<T>() - align;
            align= align_of::<T>();
        }

        unsafe { Layout::from_size_align_unchecked(size_of::<Meta<T>>() + pad + cap * size_of::<T>(), align) }
    }
}
