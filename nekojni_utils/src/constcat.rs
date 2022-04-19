pub const fn const_concat<const LEN: usize>(strs: &'static [&'static str]) -> [u8; LEN] {
    let mut const_buf: [u8; LEN] = [0; LEN];
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
                33 => constcat::as_slice(&constcat::const_concat::<33>(&[$($a,)*])),
                34 => constcat::as_slice(&constcat::const_concat::<34>(&[$($a,)*])),
                35 => constcat::as_slice(&constcat::const_concat::<35>(&[$($a,)*])),
                36 => constcat::as_slice(&constcat::const_concat::<36>(&[$($a,)*])),
                37 => constcat::as_slice(&constcat::const_concat::<37>(&[$($a,)*])),
                38 => constcat::as_slice(&constcat::const_concat::<38>(&[$($a,)*])),
                39 => constcat::as_slice(&constcat::const_concat::<39>(&[$($a,)*])),
                40 => constcat::as_slice(&constcat::const_concat::<40>(&[$($a,)*])),
                41 => constcat::as_slice(&constcat::const_concat::<41>(&[$($a,)*])),
                42 => constcat::as_slice(&constcat::const_concat::<42>(&[$($a,)*])),
                43 => constcat::as_slice(&constcat::const_concat::<43>(&[$($a,)*])),
                44 => constcat::as_slice(&constcat::const_concat::<44>(&[$($a,)*])),
                45 => constcat::as_slice(&constcat::const_concat::<45>(&[$($a,)*])),
                46 => constcat::as_slice(&constcat::const_concat::<46>(&[$($a,)*])),
                47 => constcat::as_slice(&constcat::const_concat::<47>(&[$($a,)*])),
                48 => constcat::as_slice(&constcat::const_concat::<48>(&[$($a,)*])),
                49 => constcat::as_slice(&constcat::const_concat::<49>(&[$($a,)*])),
                50 => constcat::as_slice(&constcat::const_concat::<50>(&[$($a,)*])),
                51 => constcat::as_slice(&constcat::const_concat::<51>(&[$($a,)*])),
                52 => constcat::as_slice(&constcat::const_concat::<52>(&[$($a,)*])),
                53 => constcat::as_slice(&constcat::const_concat::<53>(&[$($a,)*])),
                54 => constcat::as_slice(&constcat::const_concat::<54>(&[$($a,)*])),
                55 => constcat::as_slice(&constcat::const_concat::<55>(&[$($a,)*])),
                56 => constcat::as_slice(&constcat::const_concat::<56>(&[$($a,)*])),
                57 => constcat::as_slice(&constcat::const_concat::<57>(&[$($a,)*])),
                58 => constcat::as_slice(&constcat::const_concat::<58>(&[$($a,)*])),
                59 => constcat::as_slice(&constcat::const_concat::<59>(&[$($a,)*])),
                60 => constcat::as_slice(&constcat::const_concat::<60>(&[$($a,)*])),
                61 => constcat::as_slice(&constcat::const_concat::<61>(&[$($a,)*])),
                62 => constcat::as_slice(&constcat::const_concat::<62>(&[$($a,)*])),
                63 => constcat::as_slice(&constcat::const_concat::<63>(&[$($a,)*])),
                64 => constcat::as_slice(&constcat::const_concat::<64>(&[$($a,)*])),
                65 => constcat::as_slice(&constcat::const_concat::<65>(&[$($a,)*])),
                66 => constcat::as_slice(&constcat::const_concat::<66>(&[$($a,)*])),
                67 => constcat::as_slice(&constcat::const_concat::<67>(&[$($a,)*])),
                68 => constcat::as_slice(&constcat::const_concat::<68>(&[$($a,)*])),
                69 => constcat::as_slice(&constcat::const_concat::<69>(&[$($a,)*])),
                70 => constcat::as_slice(&constcat::const_concat::<70>(&[$($a,)*])),
                71 => constcat::as_slice(&constcat::const_concat::<71>(&[$($a,)*])),
                72 => constcat::as_slice(&constcat::const_concat::<72>(&[$($a,)*])),
                73 => constcat::as_slice(&constcat::const_concat::<73>(&[$($a,)*])),
                74 => constcat::as_slice(&constcat::const_concat::<74>(&[$($a,)*])),
                75 => constcat::as_slice(&constcat::const_concat::<75>(&[$($a,)*])),
                76 => constcat::as_slice(&constcat::const_concat::<76>(&[$($a,)*])),
                77 => constcat::as_slice(&constcat::const_concat::<77>(&[$($a,)*])),
                78 => constcat::as_slice(&constcat::const_concat::<78>(&[$($a,)*])),
                79 => constcat::as_slice(&constcat::const_concat::<79>(&[$($a,)*])),
                80 => constcat::as_slice(&constcat::const_concat::<80>(&[$($a,)*])),
                81 => constcat::as_slice(&constcat::const_concat::<81>(&[$($a,)*])),
                82 => constcat::as_slice(&constcat::const_concat::<82>(&[$($a,)*])),
                83 => constcat::as_slice(&constcat::const_concat::<83>(&[$($a,)*])),
                84 => constcat::as_slice(&constcat::const_concat::<84>(&[$($a,)*])),
                85 => constcat::as_slice(&constcat::const_concat::<85>(&[$($a,)*])),
                86 => constcat::as_slice(&constcat::const_concat::<86>(&[$($a,)*])),
                87 => constcat::as_slice(&constcat::const_concat::<87>(&[$($a,)*])),
                88 => constcat::as_slice(&constcat::const_concat::<88>(&[$($a,)*])),
                89 => constcat::as_slice(&constcat::const_concat::<89>(&[$($a,)*])),
                90 => constcat::as_slice(&constcat::const_concat::<90>(&[$($a,)*])),
                91 => constcat::as_slice(&constcat::const_concat::<91>(&[$($a,)*])),
                92 => constcat::as_slice(&constcat::const_concat::<92>(&[$($a,)*])),
                93 => constcat::as_slice(&constcat::const_concat::<93>(&[$($a,)*])),
                94 => constcat::as_slice(&constcat::const_concat::<94>(&[$($a,)*])),
                95 => constcat::as_slice(&constcat::const_concat::<95>(&[$($a,)*])),
                96 => constcat::as_slice(&constcat::const_concat::<96>(&[$($a,)*])),
                97 => constcat::as_slice(&constcat::const_concat::<97>(&[$($a,)*])),
                98 => constcat::as_slice(&constcat::const_concat::<98>(&[$($a,)*])),
                99 => constcat::as_slice(&constcat::const_concat::<99>(&[$($a,)*])),
                100 => constcat::as_slice(&constcat::const_concat::<100>(&[$($a,)*])),
                101 => constcat::as_slice(&constcat::const_concat::<101>(&[$($a,)*])),
                102 => constcat::as_slice(&constcat::const_concat::<102>(&[$($a,)*])),
                103 => constcat::as_slice(&constcat::const_concat::<103>(&[$($a,)*])),
                104 => constcat::as_slice(&constcat::const_concat::<104>(&[$($a,)*])),
                105 => constcat::as_slice(&constcat::const_concat::<105>(&[$($a,)*])),
                106 => constcat::as_slice(&constcat::const_concat::<106>(&[$($a,)*])),
                107 => constcat::as_slice(&constcat::const_concat::<107>(&[$($a,)*])),
                108 => constcat::as_slice(&constcat::const_concat::<108>(&[$($a,)*])),
                109 => constcat::as_slice(&constcat::const_concat::<109>(&[$($a,)*])),
                110 => constcat::as_slice(&constcat::const_concat::<110>(&[$($a,)*])),
                111 => constcat::as_slice(&constcat::const_concat::<111>(&[$($a,)*])),
                112 => constcat::as_slice(&constcat::const_concat::<112>(&[$($a,)*])),
                113 => constcat::as_slice(&constcat::const_concat::<113>(&[$($a,)*])),
                114 => constcat::as_slice(&constcat::const_concat::<114>(&[$($a,)*])),
                115 => constcat::as_slice(&constcat::const_concat::<115>(&[$($a,)*])),
                116 => constcat::as_slice(&constcat::const_concat::<116>(&[$($a,)*])),
                117 => constcat::as_slice(&constcat::const_concat::<117>(&[$($a,)*])),
                118 => constcat::as_slice(&constcat::const_concat::<118>(&[$($a,)*])),
                119 => constcat::as_slice(&constcat::const_concat::<119>(&[$($a,)*])),
                120 => constcat::as_slice(&constcat::const_concat::<120>(&[$($a,)*])),
                121 => constcat::as_slice(&constcat::const_concat::<121>(&[$($a,)*])),
                122 => constcat::as_slice(&constcat::const_concat::<122>(&[$($a,)*])),
                123 => constcat::as_slice(&constcat::const_concat::<123>(&[$($a,)*])),
                124 => constcat::as_slice(&constcat::const_concat::<124>(&[$($a,)*])),
                125 => constcat::as_slice(&constcat::const_concat::<125>(&[$($a,)*])),
                126 => constcat::as_slice(&constcat::const_concat::<126>(&[$($a,)*])),
                127 => constcat::as_slice(&constcat::const_concat::<127>(&[$($a,)*])),
                128 => constcat::as_slice(&constcat::const_concat::<128>(&[$($a,)*])),
                x if x <= 135 => constcat::as_slice(&constcat::const_concat::<135>(&[$($a,)*])),
                x if x <= 142 => constcat::as_slice(&constcat::const_concat::<142>(&[$($a,)*])),
                x if x <= 150 => constcat::as_slice(&constcat::const_concat::<150>(&[$($a,)*])),
                x if x <= 158 => constcat::as_slice(&constcat::const_concat::<158>(&[$($a,)*])),
                x if x <= 166 => constcat::as_slice(&constcat::const_concat::<166>(&[$($a,)*])),
                x if x <= 175 => constcat::as_slice(&constcat::const_concat::<175>(&[$($a,)*])),
                x if x <= 184 => constcat::as_slice(&constcat::const_concat::<184>(&[$($a,)*])),
                x if x <= 194 => constcat::as_slice(&constcat::const_concat::<194>(&[$($a,)*])),
                x if x <= 204 => constcat::as_slice(&constcat::const_concat::<204>(&[$($a,)*])),
                x if x <= 215 => constcat::as_slice(&constcat::const_concat::<215>(&[$($a,)*])),
                x if x <= 226 => constcat::as_slice(&constcat::const_concat::<226>(&[$($a,)*])),
                x if x <= 238 => constcat::as_slice(&constcat::const_concat::<238>(&[$($a,)*])),
                x if x <= 250 => constcat::as_slice(&constcat::const_concat::<250>(&[$($a,)*])),
                x if x <= 263 => constcat::as_slice(&constcat::const_concat::<263>(&[$($a,)*])),
                x if x <= 277 => constcat::as_slice(&constcat::const_concat::<277>(&[$($a,)*])),
                x if x <= 291 => constcat::as_slice(&constcat::const_concat::<291>(&[$($a,)*])),
                x if x <= 306 => constcat::as_slice(&constcat::const_concat::<306>(&[$($a,)*])),
                x if x <= 322 => constcat::as_slice(&constcat::const_concat::<322>(&[$($a,)*])),
                x if x <= 339 => constcat::as_slice(&constcat::const_concat::<339>(&[$($a,)*])),
                x if x <= 356 => constcat::as_slice(&constcat::const_concat::<356>(&[$($a,)*])),
                x if x <= 374 => constcat::as_slice(&constcat::const_concat::<374>(&[$($a,)*])),
                x if x <= 393 => constcat::as_slice(&constcat::const_concat::<393>(&[$($a,)*])),
                x if x <= 413 => constcat::as_slice(&constcat::const_concat::<413>(&[$($a,)*])),
                x if x <= 434 => constcat::as_slice(&constcat::const_concat::<434>(&[$($a,)*])),
                x if x <= 456 => constcat::as_slice(&constcat::const_concat::<456>(&[$($a,)*])),
                x if x <= 479 => constcat::as_slice(&constcat::const_concat::<479>(&[$($a,)*])),
                x if x <= 503 => constcat::as_slice(&constcat::const_concat::<503>(&[$($a,)*])),
                x if x <= 529 => constcat::as_slice(&constcat::const_concat::<529>(&[$($a,)*])),
                x if x <= 556 => constcat::as_slice(&constcat::const_concat::<556>(&[$($a,)*])),
                x if x <= 584 => constcat::as_slice(&constcat::const_concat::<584>(&[$($a,)*])),
                x if x <= 614 => constcat::as_slice(&constcat::const_concat::<614>(&[$($a,)*])),
                x if x <= 645 => constcat::as_slice(&constcat::const_concat::<645>(&[$($a,)*])),
                x if x <= 678 => constcat::as_slice(&constcat::const_concat::<678>(&[$($a,)*])),
                x if x <= 712 => constcat::as_slice(&constcat::const_concat::<712>(&[$($a,)*])),
                x if x <= 748 => constcat::as_slice(&constcat::const_concat::<748>(&[$($a,)*])),
                x if x <= 786 => constcat::as_slice(&constcat::const_concat::<786>(&[$($a,)*])),
                x if x <= 826 => constcat::as_slice(&constcat::const_concat::<826>(&[$($a,)*])),
                x if x <= 868 => constcat::as_slice(&constcat::const_concat::<868>(&[$($a,)*])),
                x if x <= 912 => constcat::as_slice(&constcat::const_concat::<912>(&[$($a,)*])),
                x if x <= 958 => constcat::as_slice(&constcat::const_concat::<958>(&[$($a,)*])),
                x if x <= 1006 => constcat::as_slice(&constcat::const_concat::<1006>(&[$($a,)*])),
                x if x <= 1057 => constcat::as_slice(&constcat::const_concat::<1057>(&[$($a,)*])),
                x if x <= 1110 => constcat::as_slice(&constcat::const_concat::<1110>(&[$($a,)*])),
                x if x <= 1166 => constcat::as_slice(&constcat::const_concat::<1166>(&[$($a,)*])),
                x if x <= 1225 => constcat::as_slice(&constcat::const_concat::<1225>(&[$($a,)*])),
                x if x <= 1287 => constcat::as_slice(&constcat::const_concat::<1287>(&[$($a,)*])),
                x if x <= 1352 => constcat::as_slice(&constcat::const_concat::<1352>(&[$($a,)*])),
                x if x <= 1420 => constcat::as_slice(&constcat::const_concat::<1420>(&[$($a,)*])),
                x if x <= 1491 => constcat::as_slice(&constcat::const_concat::<1491>(&[$($a,)*])),
                x if x <= 1566 => constcat::as_slice(&constcat::const_concat::<1566>(&[$($a,)*])),
                x if x <= 1645 => constcat::as_slice(&constcat::const_concat::<1645>(&[$($a,)*])),
                x if x <= 1728 => constcat::as_slice(&constcat::const_concat::<1728>(&[$($a,)*])),
                x if x <= 1815 => constcat::as_slice(&constcat::const_concat::<1815>(&[$($a,)*])),
                x if x <= 1906 => constcat::as_slice(&constcat::const_concat::<1906>(&[$($a,)*])),
                x if x <= 2002 => constcat::as_slice(&constcat::const_concat::<2002>(&[$($a,)*])),
                x if x <= 2103 => constcat::as_slice(&constcat::const_concat::<2103>(&[$($a,)*])),
                x if x <= 2209 => constcat::as_slice(&constcat::const_concat::<2209>(&[$($a,)*])),
                x if x <= 2320 => constcat::as_slice(&constcat::const_concat::<2320>(&[$($a,)*])),
                x if x <= 2436 => constcat::as_slice(&constcat::const_concat::<2436>(&[$($a,)*])),
                x if x <= 2558 => constcat::as_slice(&constcat::const_concat::<2558>(&[$($a,)*])),
                x if x <= 2686 => constcat::as_slice(&constcat::const_concat::<2686>(&[$($a,)*])),
                x if x <= 2821 => constcat::as_slice(&constcat::const_concat::<2821>(&[$($a,)*])),
                x if x <= 2963 => constcat::as_slice(&constcat::const_concat::<2963>(&[$($a,)*])),
                x if x <= 3112 => constcat::as_slice(&constcat::const_concat::<3112>(&[$($a,)*])),
                x if x <= 3268 => constcat::as_slice(&constcat::const_concat::<3268>(&[$($a,)*])),
                x if x <= 3432 => constcat::as_slice(&constcat::const_concat::<3432>(&[$($a,)*])),
                x if x <= 3604 => constcat::as_slice(&constcat::const_concat::<3604>(&[$($a,)*])),
                x if x <= 3785 => constcat::as_slice(&constcat::const_concat::<3785>(&[$($a,)*])),
                x if x <= 3975 => constcat::as_slice(&constcat::const_concat::<3975>(&[$($a,)*])),
                x if x <= 4174 => constcat::as_slice(&constcat::const_concat::<4174>(&[$($a,)*])),
                x if x <= 4383 => constcat::as_slice(&constcat::const_concat::<4383>(&[$($a,)*])),
                x if x <= 4603 => constcat::as_slice(&constcat::const_concat::<4603>(&[$($a,)*])),
                x if x <= 4834 => constcat::as_slice(&constcat::const_concat::<4834>(&[$($a,)*])),
                x if x <= 5076 => constcat::as_slice(&constcat::const_concat::<5076>(&[$($a,)*])),
                x if x <= 5330 => constcat::as_slice(&constcat::const_concat::<5330>(&[$($a,)*])),
                x if x <= 5597 => constcat::as_slice(&constcat::const_concat::<5597>(&[$($a,)*])),
                x if x <= 5877 => constcat::as_slice(&constcat::const_concat::<5877>(&[$($a,)*])),
                x if x <= 6171 => constcat::as_slice(&constcat::const_concat::<6171>(&[$($a,)*])),
                x if x <= 6480 => constcat::as_slice(&constcat::const_concat::<6480>(&[$($a,)*])),
                x if x <= 6804 => constcat::as_slice(&constcat::const_concat::<6804>(&[$($a,)*])),
                x if x <= 7145 => constcat::as_slice(&constcat::const_concat::<7145>(&[$($a,)*])),
                x if x <= 7503 => constcat::as_slice(&constcat::const_concat::<7503>(&[$($a,)*])),
                x if x <= 7879 => constcat::as_slice(&constcat::const_concat::<7879>(&[$($a,)*])),
                x if x <= 8273 => constcat::as_slice(&constcat::const_concat::<8273>(&[$($a,)*])),
                x if x <= 8687 => constcat::as_slice(&constcat::const_concat::<8687>(&[$($a,)*])),
                x if x <= 9122 => constcat::as_slice(&constcat::const_concat::<9122>(&[$($a,)*])),
                x if x <= 9579 => constcat::as_slice(&constcat::const_concat::<9579>(&[$($a,)*])),
                x if x <= 10058 => constcat::as_slice(&constcat::const_concat::<10058>(&[$($a,)*])),
                x if x <= 10561 => constcat::as_slice(&constcat::const_concat::<10561>(&[$($a,)*])),
                x if x <= 11090 => constcat::as_slice(&constcat::const_concat::<11090>(&[$($a,)*])),
                x if x <= 11645 => constcat::as_slice(&constcat::const_concat::<11645>(&[$($a,)*])),
                x if x <= 12228 => constcat::as_slice(&constcat::const_concat::<12228>(&[$($a,)*])),
                x if x <= 12840 => constcat::as_slice(&constcat::const_concat::<12840>(&[$($a,)*])),
                x if x <= 13482 => constcat::as_slice(&constcat::const_concat::<13482>(&[$($a,)*])),
                x if x <= 14157 => constcat::as_slice(&constcat::const_concat::<14157>(&[$($a,)*])),
                x if x <= 14865 => constcat::as_slice(&constcat::const_concat::<14865>(&[$($a,)*])),
                x if x <= 15609 => constcat::as_slice(&constcat::const_concat::<15609>(&[$($a,)*])),
                x if x <= 16390 => constcat::as_slice(&constcat::const_concat::<16390>(&[$($a,)*])),
                x if x <= 17210 => constcat::as_slice(&constcat::const_concat::<17210>(&[$($a,)*])),
                x if x <= 18071 => constcat::as_slice(&constcat::const_concat::<18071>(&[$($a,)*])),
                x if x <= 18975 => constcat::as_slice(&constcat::const_concat::<18975>(&[$($a,)*])),
                x if x <= 19924 => constcat::as_slice(&constcat::const_concat::<19924>(&[$($a,)*])),
                x if x <= 20921 => constcat::as_slice(&constcat::const_concat::<20921>(&[$($a,)*])),
                x if x <= 21968 => constcat::as_slice(&constcat::const_concat::<21968>(&[$($a,)*])),
                x if x <= 23067 => constcat::as_slice(&constcat::const_concat::<23067>(&[$($a,)*])),
                x if x <= 24221 => constcat::as_slice(&constcat::const_concat::<24221>(&[$($a,)*])),
                x if x <= 25433 => constcat::as_slice(&constcat::const_concat::<25433>(&[$($a,)*])),
                x if x <= 26705 => constcat::as_slice(&constcat::const_concat::<26705>(&[$($a,)*])),
                x if x <= 28041 => constcat::as_slice(&constcat::const_concat::<28041>(&[$($a,)*])),
                x if x <= 29444 => constcat::as_slice(&constcat::const_concat::<29444>(&[$($a,)*])),
                x if x <= 30917 => constcat::as_slice(&constcat::const_concat::<30917>(&[$($a,)*])),
                x if x <= 32463 => constcat::as_slice(&constcat::const_concat::<32463>(&[$($a,)*])),
                x if x <= 34087 => constcat::as_slice(&constcat::const_concat::<34087>(&[$($a,)*])),
                x if x <= 35792 => constcat::as_slice(&constcat::const_concat::<35792>(&[$($a,)*])),
                x if x <= 37582 => constcat::as_slice(&constcat::const_concat::<37582>(&[$($a,)*])),
                x if x <= 39462 => constcat::as_slice(&constcat::const_concat::<39462>(&[$($a,)*])),
                x if x <= 41436 => constcat::as_slice(&constcat::const_concat::<41436>(&[$($a,)*])),
                x if x <= 43508 => constcat::as_slice(&constcat::const_concat::<43508>(&[$($a,)*])),
                x if x <= 45684 => constcat::as_slice(&constcat::const_concat::<45684>(&[$($a,)*])),
                x if x <= 47969 => constcat::as_slice(&constcat::const_concat::<47969>(&[$($a,)*])),
                x if x <= 50368 => constcat::as_slice(&constcat::const_concat::<50368>(&[$($a,)*])),
                x if x <= 52887 => constcat::as_slice(&constcat::const_concat::<52887>(&[$($a,)*])),
                x if x <= 55532 => constcat::as_slice(&constcat::const_concat::<55532>(&[$($a,)*])),
                x if x <= 58309 => constcat::as_slice(&constcat::const_concat::<58309>(&[$($a,)*])),
                x if x <= 61225 => constcat::as_slice(&constcat::const_concat::<61225>(&[$($a,)*])),
                x if x <= 64287 => constcat::as_slice(&constcat::const_concat::<64287>(&[$($a,)*])),
                x if x <= 65535 => constcat::as_slice(&constcat::const_concat::<65535>(&[$($a,)*])),
                _ => panic!("String is too long for constcat_generic."),
            };
            constcat::slice(buf, 0, len)
        }
    };
}
