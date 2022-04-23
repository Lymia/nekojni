pub const fn const_concat<const LEN: usize>(strs: &'static [&'static str]) -> [u8; LEN] {
    let mut const_buf: [u8; LEN] = [0xCCu8; LEN];
    let mut len = 0;

    let mut i = 0;
    loop {
        let str = strs[i].as_bytes();
        let mut j = 0;
        loop {
            const_buf[len] = str[j];
            j += 1;
            len += 1;
            if j >= str.len() {
                break;
            }
        }
        i += 1;
        if i >= strs.len() {
            break;
        }
    }

    const_buf
}
pub const fn as_slice<const LEN: usize>(a: &'static [u8; LEN]) -> &'static [u8] {
    a.as_slice()
}
pub const unsafe fn as_str<const LEN: usize>(a: &'static [u8; LEN]) -> &'static str {
    std::str::from_utf8_unchecked(a.as_slice())
}
pub const unsafe fn slice(a: &'static [u8], from: usize, to: usize) -> &'static str {
    // safety: the &[u8] is from strs to begin with, so this should be safe.
    std::str::from_utf8_unchecked(konst::slice::slice_range(a, from, to))
}

#[macro_export]
macro_rules! constcat_const {
    () => { "" };
    ($a:expr $(,)?) => { $a };
    ($($a:expr),* $(,)?) => {
        unsafe {
            $crate::constcat::as_str(
                &$crate::constcat::const_concat::<{ 0 $(+ $a.len())* }>(&[$($a,)*])
            )
        }
    };
}

#[macro_export]
macro_rules! constcat_generic {
    () => { "" };
    ($a:expr $(,)?) => { $a };
    ($($a:expr),* $(,)?) => {
        unsafe {
            use $crate::constcat;
            let len = 0 $(+ $a.len())*;
            let buf = match len {
                0 => b"",
                1 => constcat::as_slice(&constcat::const_concat::<1>(&[$($a,)*])),
                2 => constcat::as_slice(&constcat::const_concat::<2>(&[$($a,)*])),
                3 => constcat::as_slice(&constcat::const_concat::<3>(&[$($a,)*])),
                4 => constcat::as_slice(&constcat::const_concat::<4>(&[$($a,)*])),
                5 => constcat::as_slice(&constcat::const_concat::<5>(&[$($a,)*])),
                6 => constcat::as_slice(&constcat::const_concat::<6>(&[$($a,)*])),
                7 => constcat::as_slice(&constcat::const_concat::<7>(&[$($a,)*])),
                8 => constcat::as_slice(&constcat::const_concat::<8>(&[$($a,)*])),
                9 => constcat::as_slice(&constcat::const_concat::<9>(&[$($a,)*])),
                10 => constcat::as_slice(&constcat::const_concat::<10>(&[$($a,)*])),
                11 => constcat::as_slice(&constcat::const_concat::<11>(&[$($a,)*])),
                12 => constcat::as_slice(&constcat::const_concat::<12>(&[$($a,)*])),
                13 => constcat::as_slice(&constcat::const_concat::<13>(&[$($a,)*])),
                14 => constcat::as_slice(&constcat::const_concat::<14>(&[$($a,)*])),
                15 => constcat::as_slice(&constcat::const_concat::<15>(&[$($a,)*])),
                16 => constcat::as_slice(&constcat::const_concat::<16>(&[$($a,)*])),
                17 => constcat::as_slice(&constcat::const_concat::<17>(&[$($a,)*])),
                18 => constcat::as_slice(&constcat::const_concat::<18>(&[$($a,)*])),
                19 => constcat::as_slice(&constcat::const_concat::<19>(&[$($a,)*])),
                20 => constcat::as_slice(&constcat::const_concat::<20>(&[$($a,)*])),
                21 => constcat::as_slice(&constcat::const_concat::<21>(&[$($a,)*])),
                22 => constcat::as_slice(&constcat::const_concat::<22>(&[$($a,)*])),
                23 => constcat::as_slice(&constcat::const_concat::<23>(&[$($a,)*])),
                24 => constcat::as_slice(&constcat::const_concat::<24>(&[$($a,)*])),
                25 => constcat::as_slice(&constcat::const_concat::<25>(&[$($a,)*])),
                26 => constcat::as_slice(&constcat::const_concat::<26>(&[$($a,)*])),
                27 => constcat::as_slice(&constcat::const_concat::<27>(&[$($a,)*])),
                28 => constcat::as_slice(&constcat::const_concat::<28>(&[$($a,)*])),
                29 => constcat::as_slice(&constcat::const_concat::<29>(&[$($a,)*])),
                30 => constcat::as_slice(&constcat::const_concat::<30>(&[$($a,)*])),
                31 => constcat::as_slice(&constcat::const_concat::<31>(&[$($a,)*])),
                32 => constcat::as_slice(&constcat::const_concat::<32>(&[$($a,)*])),
                x if x <= 36 => constcat::as_slice(&constcat::const_concat::<36>(&[$($a,)*])),
                x if x <= 40 => constcat::as_slice(&constcat::const_concat::<40>(&[$($a,)*])),
                x if x <= 44 => constcat::as_slice(&constcat::const_concat::<44>(&[$($a,)*])),
                x if x <= 48 => constcat::as_slice(&constcat::const_concat::<48>(&[$($a,)*])),
                x if x <= 52 => constcat::as_slice(&constcat::const_concat::<52>(&[$($a,)*])),
                x if x <= 56 => constcat::as_slice(&constcat::const_concat::<56>(&[$($a,)*])),
                x if x <= 60 => constcat::as_slice(&constcat::const_concat::<60>(&[$($a,)*])),
                x if x <= 64 => constcat::as_slice(&constcat::const_concat::<64>(&[$($a,)*])),
                x if x <= 68 => constcat::as_slice(&constcat::const_concat::<68>(&[$($a,)*])),
                x if x <= 72 => constcat::as_slice(&constcat::const_concat::<72>(&[$($a,)*])),
                x if x <= 76 => constcat::as_slice(&constcat::const_concat::<76>(&[$($a,)*])),
                x if x <= 80 => constcat::as_slice(&constcat::const_concat::<80>(&[$($a,)*])),
                x if x <= 84 => constcat::as_slice(&constcat::const_concat::<84>(&[$($a,)*])),
                x if x <= 88 => constcat::as_slice(&constcat::const_concat::<88>(&[$($a,)*])),
                x if x <= 92 => constcat::as_slice(&constcat::const_concat::<92>(&[$($a,)*])),
                x if x <= 96 => constcat::as_slice(&constcat::const_concat::<96>(&[$($a,)*])),
                x if x <= 100 => constcat::as_slice(&constcat::const_concat::<100>(&[$($a,)*])),
                x if x <= 104 => constcat::as_slice(&constcat::const_concat::<104>(&[$($a,)*])),
                x if x <= 108 => constcat::as_slice(&constcat::const_concat::<108>(&[$($a,)*])),
                x if x <= 112 => constcat::as_slice(&constcat::const_concat::<112>(&[$($a,)*])),
                x if x <= 116 => constcat::as_slice(&constcat::const_concat::<116>(&[$($a,)*])),
                x if x <= 120 => constcat::as_slice(&constcat::const_concat::<120>(&[$($a,)*])),
                x if x <= 124 => constcat::as_slice(&constcat::const_concat::<124>(&[$($a,)*])),
                x if x <= 128 => constcat::as_slice(&constcat::const_concat::<128>(&[$($a,)*])),
                x if x <= 141 => constcat::as_slice(&constcat::const_concat::<141>(&[$($a,)*])),
                x if x <= 156 => constcat::as_slice(&constcat::const_concat::<156>(&[$($a,)*])),
                x if x <= 172 => constcat::as_slice(&constcat::const_concat::<172>(&[$($a,)*])),
                x if x <= 190 => constcat::as_slice(&constcat::const_concat::<190>(&[$($a,)*])),
                x if x <= 210 => constcat::as_slice(&constcat::const_concat::<210>(&[$($a,)*])),
                x if x <= 232 => constcat::as_slice(&constcat::const_concat::<232>(&[$($a,)*])),
                x if x <= 256 => constcat::as_slice(&constcat::const_concat::<256>(&[$($a,)*])),
                x if x <= 282 => constcat::as_slice(&constcat::const_concat::<282>(&[$($a,)*])),
                x if x <= 311 => constcat::as_slice(&constcat::const_concat::<311>(&[$($a,)*])),
                x if x <= 343 => constcat::as_slice(&constcat::const_concat::<343>(&[$($a,)*])),
                x if x <= 378 => constcat::as_slice(&constcat::const_concat::<378>(&[$($a,)*])),
                x if x <= 416 => constcat::as_slice(&constcat::const_concat::<416>(&[$($a,)*])),
                x if x <= 458 => constcat::as_slice(&constcat::const_concat::<458>(&[$($a,)*])),
                x if x <= 504 => constcat::as_slice(&constcat::const_concat::<504>(&[$($a,)*])),
                x if x <= 555 => constcat::as_slice(&constcat::const_concat::<555>(&[$($a,)*])),
                x if x <= 611 => constcat::as_slice(&constcat::const_concat::<611>(&[$($a,)*])),
                x if x <= 673 => constcat::as_slice(&constcat::const_concat::<673>(&[$($a,)*])),
                x if x <= 741 => constcat::as_slice(&constcat::const_concat::<741>(&[$($a,)*])),
                x if x <= 816 => constcat::as_slice(&constcat::const_concat::<816>(&[$($a,)*])),
                x if x <= 898 => constcat::as_slice(&constcat::const_concat::<898>(&[$($a,)*])),
                x if x <= 988 => constcat::as_slice(&constcat::const_concat::<988>(&[$($a,)*])),
                x if x <= 1087 => constcat::as_slice(&constcat::const_concat::<1087>(&[$($a,)*])),
                x if x <= 1196 => constcat::as_slice(&constcat::const_concat::<1196>(&[$($a,)*])),
                x if x <= 1316 => constcat::as_slice(&constcat::const_concat::<1316>(&[$($a,)*])),
                x if x <= 1448 => constcat::as_slice(&constcat::const_concat::<1448>(&[$($a,)*])),
                x if x <= 1593 => constcat::as_slice(&constcat::const_concat::<1593>(&[$($a,)*])),
                x if x <= 1753 => constcat::as_slice(&constcat::const_concat::<1753>(&[$($a,)*])),
                x if x <= 1929 => constcat::as_slice(&constcat::const_concat::<1929>(&[$($a,)*])),
                x if x <= 2122 => constcat::as_slice(&constcat::const_concat::<2122>(&[$($a,)*])),
                x if x <= 2335 => constcat::as_slice(&constcat::const_concat::<2335>(&[$($a,)*])),
                x if x <= 2569 => constcat::as_slice(&constcat::const_concat::<2569>(&[$($a,)*])),
                x if x <= 2826 => constcat::as_slice(&constcat::const_concat::<2826>(&[$($a,)*])),
                x if x <= 3109 => constcat::as_slice(&constcat::const_concat::<3109>(&[$($a,)*])),
                x if x <= 3420 => constcat::as_slice(&constcat::const_concat::<3420>(&[$($a,)*])),
                x if x <= 3763 => constcat::as_slice(&constcat::const_concat::<3763>(&[$($a,)*])),
                x if x <= 4140 => constcat::as_slice(&constcat::const_concat::<4140>(&[$($a,)*])),
                x if x <= 4554 => constcat::as_slice(&constcat::const_concat::<4554>(&[$($a,)*])),
                x if x <= 5010 => constcat::as_slice(&constcat::const_concat::<5010>(&[$($a,)*])),
                x if x <= 5511 => constcat::as_slice(&constcat::const_concat::<5511>(&[$($a,)*])),
                x if x <= 6063 => constcat::as_slice(&constcat::const_concat::<6063>(&[$($a,)*])),
                x if x <= 6670 => constcat::as_slice(&constcat::const_concat::<6670>(&[$($a,)*])),
                x if x <= 7338 => constcat::as_slice(&constcat::const_concat::<7338>(&[$($a,)*])),
                x if x <= 8072 => constcat::as_slice(&constcat::const_concat::<8072>(&[$($a,)*])),
                x if x <= 8880 => constcat::as_slice(&constcat::const_concat::<8880>(&[$($a,)*])),
                x if x <= 9768 => constcat::as_slice(&constcat::const_concat::<9768>(&[$($a,)*])),
                x if x <= 10745 => constcat::as_slice(&constcat::const_concat::<10745>(&[$($a,)*])),
                x if x <= 11820 => constcat::as_slice(&constcat::const_concat::<11820>(&[$($a,)*])),
                x if x <= 13003 => constcat::as_slice(&constcat::const_concat::<13003>(&[$($a,)*])),
                x if x <= 14304 => constcat::as_slice(&constcat::const_concat::<14304>(&[$($a,)*])),
                x if x <= 15735 => constcat::as_slice(&constcat::const_concat::<15735>(&[$($a,)*])),
                x if x <= 17309 => constcat::as_slice(&constcat::const_concat::<17309>(&[$($a,)*])),
                x if x <= 19040 => constcat::as_slice(&constcat::const_concat::<19040>(&[$($a,)*])),
                x if x <= 20944 => constcat::as_slice(&constcat::const_concat::<20944>(&[$($a,)*])),
                x if x <= 23039 => constcat::as_slice(&constcat::const_concat::<23039>(&[$($a,)*])),
                x if x <= 25343 => constcat::as_slice(&constcat::const_concat::<25343>(&[$($a,)*])),
                x if x <= 27878 => constcat::as_slice(&constcat::const_concat::<27878>(&[$($a,)*])),
                x if x <= 30666 => constcat::as_slice(&constcat::const_concat::<30666>(&[$($a,)*])),
                x if x <= 33733 => constcat::as_slice(&constcat::const_concat::<33733>(&[$($a,)*])),
                x if x <= 37107 => constcat::as_slice(&constcat::const_concat::<37107>(&[$($a,)*])),
                x if x <= 40818 => constcat::as_slice(&constcat::const_concat::<40818>(&[$($a,)*])),
                x if x <= 44900 => constcat::as_slice(&constcat::const_concat::<44900>(&[$($a,)*])),
                x if x <= 49391 => constcat::as_slice(&constcat::const_concat::<49391>(&[$($a,)*])),
                x if x <= 54331 => constcat::as_slice(&constcat::const_concat::<54331>(&[$($a,)*])),
                x if x <= 59765 => constcat::as_slice(&constcat::const_concat::<59765>(&[$($a,)*])),
                x if x <= 65535 => constcat::as_slice(&constcat::const_concat::<65535>(&[$($a,)*])),
                _ => panic!("String is too long for constcat_generic."),
            };
            constcat::slice(buf, 0, len)
        }
    };
}
