
#!hreg = ...;
#
#turtle $[0, 0, 100, 100]
#    {
#        t_dir     1.0 0.0;
#        t_rot_deg 30;
#        t_area $[0, 0, 0.5, 0.5] {
#            t_color $[:r_lerp, hreg, 30, 50],
#            t_arc;
#            t_move_to 0.3 0.3;
#            t_rect 0.2 0.2;
#            t_move_to 0.3 0.3;
#            t_rect 0.2 0.2;
#        };
#        t_area $[0, 0, 0.5, 0.5] {
#
#        };
#        # ...
#    };






!outreg = new 0 :sin;
!s2     = new 1 :sin;
displayln "SIN: " outreg;
#range 1 10000 1 {
#    new _ :sin;
#};

!clr = $[
    $[:mul, outreg, 1000.0],
    $[:map, outreg, -1.0, 1.0, 1.0, 0.5],
    1.0, 1.0];
!clr2 = $[$[:mul, outreg, 2000.0], 1.0, 1.0, 1.0];
!clr3 = $[$[:mul, outreg, 100.0], 1.0, 1.0, 1.0];

!cborder = $[0.0, 1.0, 1.0, 1.0];
t :with_state {
    t :look_dir $[:mul, outreg, 4] 1;
    t :rect 0.2 1.0 cborder;
    t :line 0.75 2 clr2;
    t :look_dir $[:mul, outreg, 1] 1;
    t :line 0.75 2 clr2;
##    t :c_move 1.0  0.0;
#    t :rect 0.1 0.1 cborder;
##    t :c_move 0.0  1.0;
#    t :rect 0.1 0.1 cborder;
##    t :c_move -1.0 0;
#    t :rect 0.1 0.1 cborder;
};
#t :look_dir 1 1;
#t :line 1 0.5 clr;

## middle box
#t :rect
#  $[:mul, outreg, 0.2]
#  $[:reg, outreg, 0.3]
#  clr;
#
## right small fast blinking rect
##t :c_move 0.4 0;
#t :rect $[:mul, outreg, 0.1] $[:mul, outreg, 0.1] clr2;
#
## Left arm
##t :c_move -0.8 0;
##t :c_move $[:reg, outreg, 3.146];
##t :c_rot $[:reg, outreg, 3.146];
#t :rect $[:mul, outreg, 0.2] $[:mul, outreg, 0.2] clr3;
#t :line 1 $[:map, outreg, -1, 1, 2, 4] clr3;
#t :look_dir 0 1;
#t :line 0.2 $[:map, outreg, -1, 1, 2, 4] clr3;
#t :look_dir $[:reg, outreg] 1;
#t :line 0.2 $[:map, outreg, -1, 1, 0.1, 60] clr3;

t :cmds;

!:global draw = {|1|
    # displayln "DRAW!" _ outreg " " [reg outreg];
    [reg outreg] * 90.0
}
