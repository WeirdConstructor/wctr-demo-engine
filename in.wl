!outreg = new 0 :sin;
!s2     = new 1 :sin;

!clr = $[
    $[:mul, outreg, 1000.0],
    $[:map, outreg, -1.0, 1.0, 1.0, 0.5],
    1.0,
    1.0];
!clr2 = $[$[:mul, outreg, 2000.0], 1.0, 1.0, 1.0];
!clr3 = $[$[:mul, outreg,  100.0], 1.0, 1.0, 1.0];
!cborder = $[0.0, 1.0, 1.0, 1.0];

t :with_state {
    t :look_dir $[:mul, outreg, 4] 1;
    t :line 0.75 2 clr2;
    t :rect 0.2 1.0 cborder;
#    range 1 20 1 {
#        displayln "XXX:" _;
#        t :rectline [0.11 + [float(_) * 0.1]] [0.51 + [float(_) * 0.2]] $[:reg, outreg] clr3;
#    };
#    t :look_dir $[:mul, outreg, 1] 1;
#    t :line 0.75 2 clr2;
};

t :cmds;

!:global draw = {|1|
    # displayln "DRAW!" _ outreg " " [reg outreg];
    [reg outreg] * 90.0
}
