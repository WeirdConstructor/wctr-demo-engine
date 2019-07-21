!outreg = new 0 :sin;
range 1 10000 1 {
    new _ :sin;
};
!:global draw = {|1|
    # displayln "DRAW!" _ outreg " " [reg outreg];
    [reg outreg] * 90.0
}
