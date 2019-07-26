!s1 = new 0 :sin;
!s2 = new 1 :sin;
input 0 :freq $[:mul, s2, 0.0001];
input 1 :freq 0.0003;
debug_reg :sin1 $[:reg, s1];
debug_reg :sin2 $[:reg, s2];

!clr = $[
    $[:mul, s1, 1000.0],
    $[:map, s1, -1.0, 1.0, 1.0, 0.5],
    1.0,
    1.0];
!clr2 = $[$[:mul, s1, 2000.0], 1.0, 1.0, 1.0];
!clr3 = $[$[:mul, s1,  100.0], 1.0, 1.0, 1.0];
!cborder = $[0.0, 1.0, 1.0, 1.0];

t :with_state {
    t :look_dir $[:mul, s1, 4] 1;
    t :line 0.75 2 clr2;
    t :rect 0.2 1.0 cborder;
    range 1 20 1 {
        t :rectline
            0.11 + [float(_) * 0.1]
            0.51 + [float(_) * 0.2]
            $[:reg, s1]
            clr3;
    };
    t :look_dir $[:mul, s1, 1] 1;
    t :line 0.75 2 clr2;
};

t :cmds;

!:global draw = {|1|
    # displayln "DRAW!" _ s1 " " [reg s1];
    [reg s1] * 90.0
}
