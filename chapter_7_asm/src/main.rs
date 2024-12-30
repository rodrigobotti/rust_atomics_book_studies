use chapter_7_asm::cache_line;
use chapter_7_asm::weak_ordering_cpu;

fn main() {
    cache_line::perf_atomic_cache_lines();

    weak_ordering_cpu::weak_ordering_bug();
}
