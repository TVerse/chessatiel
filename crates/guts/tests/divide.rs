use guts::{MoveGenerator, Position};
use itertools::Itertools;
use std::str::FromStr;

#[test]
#[ignore]
fn test_divide_7() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        7,
        "a2a3: 106743106
b2b3: 133233975
c2c3: 144074944
d2d3: 227598692
e2e3: 306138410
f2f3: 102021008
g2g3: 135987651
h2h3: 106678423
a2a4: 137077337
b2b4: 134087476
c2c4: 157756443
d2d4: 269605599
e2e4: 309478263
f2f4: 119614841
g2g4: 130293018
h2h4: 138495290
b1a3: 120142144
b1c3: 148527161
g1f3: 147678554
g1h3: 120669525",
    )
}

#[test]
#[ignore]
fn test_divide_6_d2d4() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1",
        6,
        "a7a6: 10117373
b7b6: 12072971
c7c6: 11811839
d7d6: 18383299
e7e6: 21458408
f7f6: 10005904
g7g6: 12007707
h7h6: 10289713
a7a5: 12209124
b7b5: 11995635
c7c5: 14865637
d7d5: 17379278
e7e5: 24621022
f7f5: 10932394
g7g5: 11111372
h7h5: 12301391
b8a6: 11058740
b8c6: 13040495
g8f6: 12906488
g8h6: 11036809",
    )
}

#[test]
#[ignore]
fn test_divide_5_d2d4_a7a5() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1",
        5,
        "a2a3: 373860
b2b3: 416782
c2c3: 425410
e2e3: 566051
f2f3: 368907
g2g3: 428056
h2h3: 374097
d4d5: 397872
a2a4: 372870
b2b4: 472729
c2c4: 455746
e2e4: 701214
f2f4: 331409
g2g4: 424795
h2h4: 431066
b1d2: 286109
b1a3: 386469
b1c3: 433866
g1f3: 447202
g1h3: 397179
c1d2: 392771
c1e3: 371206
c1f4: 503469
c1g5: 465611
c1h6: 400890
d1d2: 452342
d1d3: 828833
e1d2: 302313",
    )
}

#[test]
fn test_divide_4_d2d4_a7a5_e1d2() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/3P4/8/PPPKPPPP/RNBQ1BNR b kq - 0 1",
        4,
        "a5a4: 12684
b7b6: 13620
c7c6: 13086
d7d6: 17262
e7e6: 18436
f7f6: 12430
g7g6: 13285
h7h6: 12428
b7b5: 13573
c7c5: 15063
d7d5: 16033
e7e5: 20097
f7f5: 13003
g7g5: 13610
h7h5: 13652
b8a6: 13570
b8c6: 14165
g8f6: 13619
g8h6: 12947
a8a6: 16699
a8a7: 13051",
    )
}

#[test]
fn test_divide_3_d2d4_a7a5_e1d2_c7c5() {
    test_divide(
        "rnbqkbnr/1p1ppppp/8/p1p5/3P4/8/PPPKPPPP/RNBQ1BNR w kq - 0 1",
        3,
        "a2a3: 544
b2b3: 592
c2c3: 592
e2e3: 773
f2f3: 521
g2g3: 590
h2h3: 544
d4d5: 503
a2a4: 544
b2b4: 679
c2c4: 592
e2e4: 795
f2f4: 545
g2g4: 591
h2h4: 590
d4c5: 507
b1a3: 568
b1c3: 592
g1f3: 614
g1h3: 567
d1e1: 567
d2e1: 662
d2c3: 657
d2d3: 721
d2e3: 613",
    )
}

#[test]
fn test_divide_2_d2d4_a7a5_e1d2_c7c5_d4c5() {
    test_divide(
        "rnbqkbnr/1p1ppppp/8/p1P5/8/8/PPPKPPPP/RNBQ1BNR b kq - 0 1",
        2,
        "a5a4: 23
b7b6: 25
d7d6: 25
e7e6: 24
f7f6: 24
g7g6: 24
h7h6: 24
b7b5: 25
d7d5: 25
e7e5: 24
f7f5: 24
g7g5: 24
h7h5: 24
b8a6: 24
b8c6: 23
g8f6: 24
g8h6: 24
a8a6: 24
a8a7: 24
d8b6: 25
d8c7: 24",
    )
}

#[test]
fn test_divide_1_d2d4_a7a5_e1d2_c7c5_d4c5_d7d5() {
    test_divide(
        "rnbqkbnr/1p2pppp/8/p1Pp4/8/8/PPPKPPPP/RNBQ1BNR w kq d6 0 1",
        1,
        "a2a3: 1
b2b3: 1
c2c3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
c5c6: 1
a2a4: 1
b2b4: 1
c2c4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
c5d6: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
d1e1: 1
d2e1: 1
d2c3: 1
d2d3: 1
d2e3: 1",
    )
}

#[test]
fn test_divide_3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        3,
        "a2a3: 380
b2b3: 420
c2c3: 420
d2d3: 539
e2e3: 599
f2f3: 380
g2g3: 420
h2h3: 380
a2a4: 420
b2b4: 421
c2c4: 441
d2d4: 560
e2e4: 600
f2f4: 401
g2g4: 421
h2h4: 420
b1a3: 400
b1c3: 440
g1f3: 440
g1h3: 400",
    );
}

#[test]
fn test_divide_2_b1a3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 0 1",
        2,
        "a7a6: 20
b7b6: 20
c7c6: 20
d7d6: 20
e7e6: 20
f7f6: 20
g7g6: 20
h7h6: 20
a7a5: 20
b7b5: 20
c7c5: 20
d7d5: 20
e7e5: 20
f7f5: 20
g7g5: 20
h7h5: 20
b8a6: 20
b8c6: 20
g8f6: 20
g8h6: 20",
    );
}

#[test]
fn test_divide_1_b1a3_a7a5() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
        1,
        "b2b3: 1
c2c3: 1
d2d3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
b2b4: 1
c2c4: 1
d2d4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
g1f3: 1
g1h3: 1
a3b1: 1
a3c4: 1
a3b5: 1
a1b1: 1",
    );
}

#[test]
fn test_divide_1_b2b3_d7d5() {
    test_divide(
        "rnbqkbnr/ppp1pppp/8/3p4/8/1P6/P1PPPPPP/RNBQKBNR w KQkq - 0 1",
        1,
        "a2a3: 1
c2c3: 1
d2d3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
b3b4: 1
a2a4: 1
c2c4: 1
d2d4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
c1b2: 1
c1a3: 1",
    );
}

#[test]
fn test_divide_2_b2b3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/1P6/P1PPPPPP/RNBQKBNR b KQkq - 0 1",
        2,
        "a7a6: 21
b7b6: 21
c7c6: 21
d7d6: 21
e7e6: 21
f7f6: 21
g7g6: 21
h7h6: 21
a7a5: 21
b7b5: 21
c7c5: 21
d7d5: 21
e7e5: 21
f7f5: 21
g7g5: 21
h7h5: 21
b8a6: 21
b8c6: 21
g8f6: 21
g8h6: 21",
    )
}

#[test]
fn test_divide_2_d2d3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1",
        2,
        "a7a6: 27
b7b6: 27
c7c6: 27
d7d6: 27
e7e6: 27
f7f6: 27
g7g6: 27
h7h6: 27
a7a5: 27
b7b5: 27
c7c5: 27
d7d5: 27
e7e5: 27
f7f5: 27
g7g5: 26
h7h5: 27
b8a6: 27
b8c6: 27
g8f6: 27
g8h6: 27",
    );
}

#[test]
fn test_divide_1_d2d3_a7a5() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1",
        1,
        "a2a3: 1
b2b3: 1
c2c3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
d3d4: 1
a2a4: 1
b2b4: 1
c2c4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1d2: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
c1d2: 1
c1e3: 1
c1f4: 1
c1g5: 1
c1h6: 1
d1d2: 1
e1d2: 1",
    );
}

#[test]
fn test_divide_4() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        4,
        "a2a3: 8457
b2b3: 9345
c2c3: 9272
d2d3: 11959
e2e3: 13134
f2f3: 8457
g2g3: 9345
h2h3: 8457
a2a4: 9329
b2b4: 9332
c2c4: 9744
d2d4: 12435
e2e4: 13160
f2f4: 8929
g2g4: 9328
h2h4: 9329
b1a3: 8885
b1c3: 9755
g1f3: 9748
g1h3: 8881",
    )
}

#[test]
fn test_divide_3_c2c3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/2P5/PP1PPPPP/RNBQKBNR b KQkq - 0 1",
        3,
        "a7a6: 397
b7b6: 439
c7c6: 441
d7d6: 545
e7e6: 627
f7f6: 396
g7g6: 439
h7h6: 397
a7a5: 438
b7b5: 443
c7c5: 461
d7d5: 566
e7e5: 628
f7f5: 418
g7g5: 440
h7h5: 439
b8a6: 418
b8c6: 462
g8f6: 460
g8h6: 418",
    )
}

#[test]
fn test_divide_2_c2c3_d7d5() {
    test_divide(
        "rnbqkbnr/ppp1pppp/8/3p4/8/2P5/PP1PPPPP/RNBQKBNR w KQkq - 0 1",
        2,
        "a2a3: 28
b2b3: 28
d2d3: 28
e2e3: 28
f2f3: 28
g2g3: 28
h2h3: 28
c3c4: 29
a2a4: 28
b2b4: 28
d2d4: 27
e2e4: 29
f2f4: 28
g2g4: 27
h2h4: 28
b1a3: 28
g1f3: 28
g1h3: 28
d1c2: 28
d1b3: 28
d1a4: 6",
    )
}

#[test]
fn test_divide_1_c2c3_d7d5_d1a4() {
    test_divide(
        "rnbqkbnr/ppp1pppp/8/3p4/Q7/2P5/PP1PPPPP/RNB1KBNR b KQkq - 0 1",
        1,
        "c7c6: 1
b7b5: 1
b8c6: 1
b8d7: 1
c8d7: 1
d8d7: 1",
    )
}

#[test]
fn test_divide_3_b1a3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
        3,
        "b2b3: 400
c2c3: 460
d2d3: 519
e2e3: 599
f2f3: 380
g2g3: 420
h2h3: 380
b2b4: 401
c2c4: 441
d2d4: 540
e2e4: 600
f2f4: 401
g2g4: 421
h2h4: 420
g1f3: 440
g1h3: 400
a3b1: 400
a3c4: 480
a3b5: 475
a1b1: 380",
    )
}

#[test]
fn test_kiwipete_3() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        3,
        "a2a3: 2186
b2b3: 1964
g2g3: 1882
d5d6: 1991
a2a4: 2149
g2g4: 1843
g2h3: 1970
d5e6: 2241
c3b1: 2038
c3d1: 2040
c3a4: 2203
c3b5: 2138
e5d3: 1803
e5c4: 1880
e5g4: 1878
e5c6: 2027
e5g6: 1997
e5d7: 2124
e5f7: 2080
d2c1: 1963
d2e3: 2136
d2f4: 2000
d2g5: 2134
d2h6: 2019
e2d1: 1733
e2f1: 2060
e2d3: 2050
e2c4: 2082
e2b5: 2057
e2a6: 1907
a1b1: 1969
a1c1: 1968
a1d1: 1885
h1f1: 1929
h1g1: 2013
f3d3: 2005
f3e3: 2174
f3g3: 2214
f3h3: 2360
f3f4: 2132
f3g4: 2169
f3f5: 2396
f3h5: 2267
f3f6: 2111
e1d1: 1894
e1f1: 1855
e1g1: 2059
e1c1: 1887",
    )
}

#[test]
#[ignore]
fn test_kiwipete_4() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        4,
        "a2a3: 94405
b2b3: 81066
g2g3: 77468
d5d6: 79551
a2a4: 90978
g2g4: 75677
g2h3: 82759
d5e6: 97464
c3b1: 84773
c3d1: 84782
c3a4: 91447
c3b5: 81498
e5d3: 77431
e5c4: 77752
e5g4: 79912
e5c6: 83885
e5g6: 83866
e5d7: 93913
e5f7: 88799
d2c1: 83037
d2e3: 90274
d2f4: 84869
d2g5: 87951
d2h6: 82323
e2d1: 74963
e2f1: 88728
e2d3: 85119
e2c4: 84835
e2b5: 79739
e2a6: 69334
a1b1: 83348
a1c1: 83263
a1d1: 79695
h1f1: 81563
h1g1: 84876
f3d3: 83727
f3e3: 92505
f3g3: 94461
f3h3: 98524
f3f4: 90488
f3g4: 92037
f3f5: 104992
f3h5: 95034
f3f6: 77838
e1d1: 79989
e1f1: 77887
e1g1: 86975
e1c1: 79803",
    )
}

#[test]
fn test_kiwipete_3_e2c4() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1pB1P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 0 1",
        3,
        "b4b3: 2223
g6g5: 2042
c7c6: 2123
d7d6: 2055
c7c5: 1978
h3g2: 2424
e6d5: 2220
b4c3: 2132
b6a4: 2036
b6c4: 2009
b6d5: 2016
b6c8: 1780
f6e4: 2632
f6g4: 2340
f6d5: 2282
f6h5: 2191
f6h7: 2091
f6g8: 2092
a6c4: 1957
a6b5: 2142
a6b7: 2182
a6c8: 1878
g7h6: 2119
g7f8: 1882
a8b8: 2137
a8c8: 1985
a8d8: 1987
h8h4: 2126
h8h5: 2084
h8h6: 1932
h8h7: 1933
h8f8: 1732
h8g8: 1832
e7c5: 2434
e7d6: 2159
e7d8: 1930
e7f8: 1925
e8d8: 1959
e8f8: 1915
e8g8: 1935
e8c8: 2004",
    )
}

#[test]
fn test_kiwipete_2_e2c4_c7c5() {
    test_divide(
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1pB1P3/2N2Q1p/PPPB1PPP/R3K2R w KQkq c6 0 1",
        2,
        "a2a3: 39
b2b3: 37
g2g3: 37
d5d6: 36
a2a4: 39
g2g4: 37
g2h3: 38
d5e6: 41
d5c6: 40
c3b1: 37
c3d1: 37
c3e2: 37
c3a4: 37
c3b5: 36
e5d3: 39
e5g4: 39
e5c6: 38
e5g6: 37
e5d7: 40
e5f7: 39
d2c1: 38
d2e3: 38
d2f4: 38
d2g5: 37
d2h6: 36
c4f1: 42
c4e2: 41
c4b3: 41
c4d3: 40
c4b5: 37
c4a6: 34
a1b1: 38
a1c1: 38
a1d1: 38
h1f1: 38
h1g1: 38
f3d1: 38
f3e2: 38
f3d3: 38
f3e3: 38
f3g3: 38
f3h3: 38
f3f4: 38
f3g4: 38
f3f5: 40
f3h5: 38
f3f6: 34
e1d1: 38
e1f1: 38
e1e2: 38
e1g1: 38
e1c1: 38",
    )
}

#[test]
fn test_kiwipete_1_e2c4_c7c5_d5c6() {
    test_divide(
        "r3k2r/p2pqpb1/bnP1pnp1/4N3/1pB1P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
d7d6: 1
d7d5: 1
h3g2: 1
d7c6: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_3_a1b1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/1R2K2R b Kkq - 0 1",
        3,
        "b4b3: 2085
g6g5: 1911
c7c6: 1996
d7d6: 1919
c7c5: 1904
h3g2: 2246
e6d5: 2000
b4c3: 2080
b6a4: 1905
b6c4: 1916
b6d5: 1857
b6c8: 1681
f6e4: 2464
f6g4: 2174
f6d5: 2095
f6h5: 2054
f6h7: 1964
f6g8: 1965
a6e2: 1821
a6d3: 1950
a6c4: 1961
a6b5: 2003
a6b7: 1971
a6c8: 1697
g7h6: 1985
g7f8: 1773
a8b8: 2003
a8c8: 1866
a8d8: 1868
h8h4: 1992
h8h5: 1956
h8h6: 1818
h8h7: 1819
h8f8: 1638
h8g8: 1728
e7c5: 2311
e7d6: 2021
e7d8: 1816
e7f8: 1811
e8d8: 1831
e8f8: 1792
e8g8: 1821
e8c8: 1880",
    )
}

#[test]
fn test_kiwipete_2_a1b1_h3g2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/1R2K2R w Kkq - 0 1",
        2,
        "a2a3: 53
b2b3: 51
h2h3: 51
d5d6: 50
a2a4: 53
h2h4: 50
d5e6: 55
c3d1: 51
c3a4: 51
c3b5: 48
e5d3: 52
e5c4: 51
e5g4: 53
e5c6: 50
e5g6: 51
e5d7: 54
e5f7: 53
d2c1: 52
d2e3: 52
d2f4: 52
d2g5: 51
d2h6: 48
e2d1: 53
e2f1: 57
e2d3: 51
e2c4: 50
e2b5: 48
e2a6: 45
b1a1: 52
b1c1: 52
b1d1: 52
h1f1: 52
h1g1: 44
f3g2: 44
f3d3: 51
f3e3: 52
f3g3: 52
f3h3: 51
f3f4: 52
f3g4: 52
f3f5: 54
f3h5: 50
f3f6: 48
e1d1: 52",
    )
}

#[test]
fn test_kiwipete_1_a1b1_h3g2_a2a3() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/1R2K2R b Kkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
c7c6: 1
d7d6: 1
c7c5: 1
g2h1q: 1
g2h1r: 1
g2h1b: 1
g2h1n: 1
g2g1q: 1
g2g1r: 1
g2g1b: 1
g2g1n: 1
b4a3: 1
e6d5: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h2: 1
h8h3: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_2_e1f1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R4K1R b kq - 0 1",
        2,
        "b4b3: 46
g6g5: 44
c7c6: 46
d7d6: 44
c7c5: 46
h3g2: 4
e6d5: 45
b4c3: 45
b6a4: 44
b6c4: 44
b6d5: 45
b6c8: 45
f6e4: 48
f6g4: 44
f6d5: 46
f6h5: 46
f6h7: 46
f6g8: 46
a6e2: 5
a6d3: 43
a6c4: 43
a6b5: 44
a6b7: 46
a6c8: 46
g7h6: 45
g7f8: 45
a8b8: 45
a8c8: 45
a8d8: 45
h8h4: 45
h8h5: 45
h8h6: 45
h8h7: 45
h8f8: 45
h8g8: 45
e7c5: 45
e7d6: 44
e7d8: 45
e7f8: 45
e8d8: 45
e8f8: 45
e8g8: 45
e8c8: 45",
    )
}

#[test]
fn test_kiwipete_1_e1f1_a6e2() {
    test_divide(
        "r3k2r/p1ppqpb1/1n2Pnp1/4N3/1p2P3/2N2Q1p/PPPBbPPP/R4K1R w kq - 0 1",
        1,
        "f1e1: 1
f1g1: 1
f1e2: 1
c3e2: 1
f3e2: 1",
    )
}

#[test]
fn test_kiwipete_1_e1f1_h3g2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/R4K1R w kq - 0 1",
        1,
        "f1e1: 1
f1g1: 1
f1g2: 1
f3g2: 1",
    )
}

#[test]
fn test_kiwipete_2_a1b1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1",
        2,
        "b4b3: 49
g6g5: 49
c7c6: 51
d7d6: 49
c7c5: 51
h3g2: 48
b4a3: 51
e6d5: 50
b4c3: 48
b6a4: 49
b6c4: 48
b6d5: 50
b6c8: 50
f6e4: 53
f6g4: 49
f6d5: 51
f6h5: 51
f6h7: 51
f6g8: 51
a6e2: 43
a6d3: 48
a6c4: 48
a6b5: 49
a6b7: 50
a6c8: 50
g7h6: 50
g7f8: 50
a8b8: 50
a8c8: 50
a8d8: 50
h8h4: 50
h8h5: 50
h8h6: 50
h8h7: 50
h8f8: 50
h8g8: 50
e7c5: 50
e7d6: 49
e7d8: 50
e7f8: 50
e8d8: 50
e8f8: 50
e8g8: 50
e8c8: 50",
    )
}

#[test]
fn test_kiwipete_1_a1b1_h3g2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/R3K2R w KQkq - 0 1",
        1,
        "b2b3: 1
h2h3: 1
a3a4: 1
d5d6: 1
h2h4: 1
a3b4: 1
d5e6: 1
c3b1: 1
c3d1: 1
c3a2: 1
c3a4: 1
c3b5: 1
e5d3: 1
e5c4: 1
e5g4: 1
e5c6: 1
e5g6: 1
e5d7: 1
e5f7: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2c4: 1
e2b5: 1
e2a6: 1
a1b1: 1
a1c1: 1
a1d1: 1
a1a2: 1
h1f1: 1
h1g1: 1
f3g2: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3h3: 1
f3f4: 1
f3g4: 1
f3f5: 1
f3h5: 1
f3f6: 1
e1d1: 1
e1c1: 1",
    )
}

#[test]
fn test_kiwipete_1_a1b1_a6e2() {
    test_divide(
        "r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBbPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
a7a6: 1
c7c6: 1
d7d6: 1
a7a5: 1
c7c5: 1
h3g2: 1
b4a3: 1
e6d5: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2f3: 1
e2c4: 1
e2b5: 1
e2a6: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        2,
        "a2a3: 44
b2b3: 42
g2g3: 42
d5d6: 41
a2a4: 44
g2g4: 42
g2h3: 43
d5e6: 46
c3b1: 42
c3d1: 42
c3a4: 42
c3b5: 39
e5d3: 43
e5c4: 42
e5g4: 44
e5c6: 41
e5g6: 42
e5d7: 45
e5f7: 44
d2c1: 43
d2e3: 43
d2f4: 43
d2g5: 42
d2h6: 41
e2d1: 44
e2f1: 44
e2d3: 42
e2c4: 41
e2b5: 39
e2a6: 36
a1b1: 43
a1c1: 43
a1d1: 43
h1f1: 43
h1g1: 43
f3d3: 42
f3e3: 43
f3g3: 43
f3h3: 43
f3f4: 43
f3g4: 43
f3f5: 45
f3h5: 43
f3f6: 39
e1d1: 43
e1f1: 43
e1g1: 43
e1c1: 43",
    )
}

#[test]
fn test_kiwipete_1_f3h3() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N4Q/PPPBBPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
c7c6: 1
d7d6: 1
c7c5: 1
e6d5: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h3: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_1_d5e6() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2Pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
c7c6: 1
d7d6: 1
c7c5: 1
d7d5: 1
h3g2: 1
f7e6: 1
b4c3: 1
d7e6: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7e6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        1,
        "a2a3: 1
b2b3: 1
g2g3: 1
d5d6: 1
a2a4: 1
g2g4: 1
g2h3: 1
d5e6: 1
c3b1: 1
c3d1: 1
c3a4: 1
c3b5: 1
e5d3: 1
e5c4: 1
e5g4: 1
e5c6: 1
e5g6: 1
e5d7: 1
e5f7: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2c4: 1
e2b5: 1
e2a6: 1
a1b1: 1
a1c1: 1
a1d1: 1
h1f1: 1
h1g1: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3h3: 1
f3f4: 1
f3g4: 1
f3f5: 1
f3h5: 1
f3f6: 1
e1d1: 1
e1f1: 1
e1g1: 1
e1c1: 1",
    )
}

#[test]
fn test_case_1() {
    test_divide(
        "4k3/3pqp2/4P3/8/8/8/8/4K3 w - - 0 1",
        2,
        "e1f2: 18
e1e2: 18
e1d1: 18
e1f1: 18
e1d2: 18",
    )
}

#[test]
#[ignore]
fn test_kiwipete_5() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        5,
        "a2a3: 4627439
b2b3: 3768824
g2g3: 3472039
d5d6: 3835265
a2a4: 4387586
g2g4: 3338154
g2h3: 3819456
d5e6: 4727437
c3b1: 3996171
c3d1: 3995761
c3a4: 4628497
c3b5: 4317482
e5d3: 3288812
e5c4: 3494887
e5g4: 3415992
e5c6: 4083458
e5g6: 3949417
e5d7: 4404043
e5f7: 4164923
d2c1: 3793390
d2e3: 4407041
d2f4: 3941257
d2g5: 4370915
d2h6: 3967365
e2d1: 3074219
e2f1: 4095479
e2d3: 4066966
e2c4: 4182989
e2b5: 4032348
e2a6: 3553501
a1b1: 3827454
a1c1: 3814203
a1d1: 3568344
h1f1: 3685756
h1g1: 3989454
f3d3: 3949570
f3e3: 4477772
f3g3: 4669768
f3h3: 5067173
f3f4: 4327936
f3g4: 4514010
f3f5: 5271134
f3h5: 4743335
f3f6: 3975992
e1d1: 3559113
e1f1: 3377351
e1g1: 4119629
e1c1: 3551583",
    )
}

#[test]
#[ignore]
fn test_kiwipete_4_e5d7() {
    test_divide(
        "r3k2r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
        4,
        "b4b3: 108157
e6e5: 89589
g6g5: 95338
c7c6: 103829
c7c5: 97285
h3g2: 103446
e6d5: 101464
b4c3: 100178
b6a4: 93613
b6c4: 92643
b6d5: 92813
b6d7: 77507
b6c8: 84558
f6e4: 127875
f6g4: 109211
f6d5: 109394
f6h5: 107158
f6d7: 90772
f6h7: 102958
f6g8: 105147
a6e2: 79234
a6d3: 95491
a6c4: 96675
a6b5: 102152
a6b7: 101293
a6c8: 89678
g7h6: 99015
g7f8: 94666
a8b8: 102034
a8c8: 95342
a8d8: 97711
h8h4: 102859
h8h5: 103509
h8h6: 94738
h8h7: 94868
h8f8: 87898
h8g8: 90441
e7c5: 112204
e7d6: 108649
e7d7: 84280
e7d8: 93462
e7f8: 94576
e8d7: 87547
e8d8: 102896
e8c8: 99890",
    )
}

#[test]
fn test_kiwipete_3_e5d7_e8c8() {
    test_divide(
        "2kr3r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQ - 0 1",
        3,
        "a2a3: 2306
b2b3: 2075
g2g3: 1987
e4e5: 2163
d5d6: 2156
a2a4: 2266
g2g4: 1989
g2h3: 2078
d5e6: 2134
c3b1: 2155
c3d1: 2157
c3a4: 2332
c3b5: 2273
d7c5: 2235
d7e5: 2247
d7b6: 178
d7f6: 1948
d7b8: 2145
d7f8: 2088
d2c1: 2071
d2e3: 2257
d2f4: 2246
d2g5: 2259
d2h6: 2139
e2d1: 1816
e2f1: 2169
e2d3: 2210
e2c4: 2246
e2b5: 2154
e2a6: 47
a1b1: 2077
a1c1: 2076
a1d1: 1987
h1f1: 2034
h1g1: 2124
f3d3: 2116
f3e3: 2250
f3g3: 2429
f3h3: 2449
f3f4: 2343
f3g4: 2288
f3f5: 2528
f3h5: 2396
f3f6: 2147
e1d1: 1999
e1f1: 1959
e1g1: 2173
e1c1: 1989",
    )
}

#[test]
fn test_kiwipete_2_e5d7_e8c8_e2a6() {
    test_divide(
        "2kr3r/p1pNqpb1/Bn2pnp1/3P4/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQ - 0 1",
        2,
        "c8d7: 47",
    )
}

#[test]
fn test_kiwipete_1_e5d7_e8c8_e2a6_c8d7() {
    test_divide(
        "3r3r/p1pkqpb1/Bn2pnp1/3P4/1p2P3/2N2Q1p/PPPB1PPP/R3K2R w KQ - 0 1",
        1,
        "a2a3: 1
b2b3: 1
g2g3: 1
e4e5: 1
d5d6: 1
a2a4: 1
g2g4: 1
g2h3: 1
d5e6: 1
c3b1: 1
c3d1: 1
c3e2: 1
c3a4: 1
c3b5: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
a6f1: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
a1b1: 1
a1c1: 1
a1d1: 1
h1f1: 1
h1g1: 1
f3d1: 1
f3e2: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3h3: 1
f3f4: 1
f3g4: 1
f3f5: 1
f3h5: 1
f3f6: 1
e1d1: 1
e1f1: 1
e1e2: 1
e1g1: 1
e1c1: 1",
    )
}

#[test]
fn test_kiwipete_2_e5d7_e8c8_d7b8() {
    test_divide(
        "1Nkr3r/p1p1qpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQ - 0 1",
        2,
        "b4b3: 46
e6e5: 43
g6g5: 44
c7c6: 46
c7c5: 46
h3g2: 43
e6d5: 45
b4c3: 45
b6a4: 44
b6c4: 43
b6d5: 45
b6d7: 45
b6a8: 45
f6e4: 47
f6g4: 44
f6d5: 46
f6h5: 46
f6d7: 46
f6h7: 46
f6e8: 46
f6g8: 46
a6e2: 38
a6d3: 43
a6c4: 43
a6b5: 44
a6b7: 45
g7h6: 45
g7f8: 45
d8d5: 45
d8d6: 44
d8d7: 45
d8e8: 45
d8f8: 45
d8g8: 45
h8h4: 45
h8h5: 45
h8h6: 45
h8h7: 45
h8e8: 45
h8f8: 45
h8g8: 45
e7c5: 45
e7d6: 44
e7d7: 45
e7e8: 45
e7f8: 45
c8b7: 45
c8b8: 42",
    )
}

fn test_divide(position: &str, depth: usize, expected: &str) {
    let generator = MoveGenerator::new();

    let position = Position::from_str(position).unwrap();

    let divided = generator.divide(&position, depth);
    let mut vec = divided
        .into_iter()
        .map(|(m, c)| {
            (
                format!(
                    "{}{}{}",
                    m.from(),
                    m.to(),
                    m.promotion()
                        .map(|p| p.to_string().to_ascii_lowercase())
                        .unwrap_or_else(|| "".to_string())
                ),
                c.to_string(),
            )
        })
        .collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = expected
        .split('\n')
        .map(|s| {
            let split = s.split(": ").collect_vec();
            (split[0].to_owned(), split[1].to_owned())
        })
        .collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    assert_eq!(vec, expected);
}
