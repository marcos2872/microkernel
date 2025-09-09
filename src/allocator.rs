//! Este módulo configura o alocador de heap global.

use linked_list_allocator::LockedHeap;

/// O alocador de heap global.
///
/// Usa o `linked_list_allocator` crate, que fornece um alocador de lista encadeada.
/// O `LockedHeap` é um wrapper que adiciona um spinlock para garantir a segurança
/// em ambientes concorrentes.
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// O endereço virtual onde o heap começará.
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// O tamanho do heap.
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
