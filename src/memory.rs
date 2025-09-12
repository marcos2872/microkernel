//! Este módulo gerencia a memória, incluindo a paginação e a alocação de frames.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Inicializa uma nova `OffsetPageTable`.
///
/// Esta função é insegura porque o chamador deve garantir que a memória física
/// completa esteja mapeada para a memória virtual no `physical_memory_offset` passado.
/// Além disso, esta função deve ser chamada apenas uma vez para evitar a criação de
/// múltiplas referências `&mut` para a mesma memória, o que é um comportamento indefinido.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Retorna uma referência mutável para a tabela de nível 4 ativa.
///
/// Esta função é insegura pelos mesmos motivos que a função `init`.
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// Um `FrameAllocator` que retorna frames usáveis a partir do mapa de memória do bootloader.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Cria um `FrameAllocator` a partir do mapa de memória passado.
    ///
    /// Esta função é insegura porque o chamador deve garantir que o mapa de memória
    /// passado é válido. O principal requisito é que todos os frames marcados
    /// como `USABLE` estejam realmente não utilizados.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Retorna um iterador sobre os frames usáveis no mapa de memória.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // obtém as regiões usáveis do mapa de memória
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // mapeia cada região para seu intervalo de endereços
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transforma em um iterador de endereços de início de frame
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // cria tipos `PhysFrame` a partir dos endereços de início
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    /// Aloca um frame de 4KiB.
    ///
    /// Retorna `None` se não houver mais frames disponíveis.
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
