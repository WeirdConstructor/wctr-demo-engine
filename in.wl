!outreg = new 0 :sin;
range 1 100000 1 {
    new _ :sin;
};
!:global draw = {|1|
#    displayln "DRAW!" outreg " " [reg outreg];
    [reg outreg] * 90.0
}
