use accel::DoBlocksFn;
use std::arch::x86_64::{
    __m128i, _mm_add_epi32, _mm_cvtsi128_si32, _mm_loadu_si128, _mm_madd_epi16, _mm_maddubs_epi16,
    _mm_sad_epu8, _mm_set_epi16, _mm_set_epi32, _mm_setr_epi8, _mm_shuffle_epi32, _mm_slli_epi32,
};

const BLOCK_SIZE: usize = 1 << 5; /* The NMAX constraint. */

pub fn accelerated_do_blocks_if_supported() -> Option<DoBlocksFn> {
    if is_x86_feature_detected!("sse3") && is_x86_feature_detected!("ssse3") {
        Some(do_blocks_ssse3)
    } else {
        None
    }
}

#[target_feature(enable = "sse3", enable = "ssse3")]
pub unsafe fn do_blocks_ssse3(adler: &mut u32, sum2: &mut u32, mut buf: &[u8]) -> usize {
    let mut s1 = *adler;
    let mut s2 = *sum2;
    let mut blocks = buf.len().wrapping_div(BLOCK_SIZE);
    let next_pos = blocks * BLOCK_SIZE;

    while blocks != 0 {
        let mut n = 5552u32.wrapping_div(BLOCK_SIZE as u32);
        if n > blocks as u32 {
            n = blocks as u32
        }
        blocks = blocks.wrapping_sub(n as usize);

        let tap1 = _mm_setr_epi8(
            32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
        );
        let tap2 = _mm_setr_epi8(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1);
        let zero = _mm_setr_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        let ones = _mm_set_epi16(1, 1, 1, 1, 1, 1, 1, 1);

        /*
         * Process n blocks of data. At most NMAX data bytes can be
         * processed before s2 must be reduced modulo BASE.
         */
        let mut v_ps: __m128i = _mm_set_epi32(0, 0, 0, s1.wrapping_mul(n) as i32);
        let mut v_s2: __m128i = _mm_set_epi32(0, 0, 0, s2 as i32);
        let mut v_s1: __m128i = _mm_set_epi32(0, 0, 0, 0);

        loop {
            /*
             * Load 32 input bytes.
             */
            let bytes1: __m128i = _mm_loadu_si128(&buf[0] as *const _ as *mut __m128i);
            let bytes2: __m128i = _mm_loadu_si128(&buf[16] as *const _ as *mut __m128i);

            /*
             * Add previous block byte sum to v_ps.
             */
            v_ps = _mm_add_epi32(v_ps, v_s1);

            /*
             * Horizontally add the bytes for s1, multiply-adds the
             * bytes by [ 32, 31, 30, ... ] for s2.
             */
            v_s1 = _mm_add_epi32(v_s1, _mm_sad_epu8(bytes1, zero));
            let mad1 = _mm_maddubs_epi16(bytes1, tap1);
            v_s2 = _mm_add_epi32(v_s2, _mm_madd_epi16(mad1, ones));
            v_s1 = _mm_add_epi32(v_s1, _mm_sad_epu8(bytes2, zero));
            let mad2 = _mm_maddubs_epi16(bytes2, tap2);
            v_s2 = _mm_add_epi32(v_s2, _mm_madd_epi16(mad2, ones));

            buf = &buf[BLOCK_SIZE..];

            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }

        v_s2 = _mm_add_epi32(v_s2, _mm_slli_epi32(v_ps, 5i32));

        /*
         * Sum epi32 ints v_s1(s2) and accumulate in s1(s2).
         */
        /// A B C D -> B A D C
        const S2301: i32 = 2 << 6 | 3 << 4 | 0 << 2 | 1;
        /// A B C D -> C D A B
        const S1032: i32 = 1 << 6 | 0 << 4 | 3 << 2 | 2;
        v_s1 = _mm_add_epi32(v_s1, _mm_shuffle_epi32(v_s1, S2301));
        v_s1 = _mm_add_epi32(v_s1, _mm_shuffle_epi32(v_s1, S1032));
        s1 = s1.wrapping_add(_mm_cvtsi128_si32(v_s1) as u32) as u32;
        v_s2 = _mm_add_epi32(v_s2, _mm_shuffle_epi32(v_s2, S2301));
        v_s2 = _mm_add_epi32(v_s2, _mm_shuffle_epi32(v_s2, S1032));
        s2 = _mm_cvtsi128_si32(v_s2) as u32;

        /*
         * Reduce.
         */
        s1 = s1.wrapping_rem(65521);
        s2 = s2.wrapping_rem(65521);
    }

    *adler = s1;
    *sum2 = s2;
    next_pos
}
