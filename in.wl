
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
displayln "SIN: " outreg;
#range 1 10000 1 {
#    new _ :sin;
#};

t :move_trans 1 1;
t :rot_trans $[:reg, outreg, 2];
t :rect
  $[:mul, outreg, 0.2]
  $[:reg, outreg, 0.3];
t :rot_trans $[:mul, outreg, -1];
t :move_trans 1 1;
t :rot_trans $[:mul, outreg, 1];
t :rect
  $[:mul, outreg, 0.2]
  $[:reg, outreg, 0.3];
#t :rot_ctx $[:mul, outreg, 3.14 * 3];
#t :rect
#t :move 1 0;
#t :rot_ctx $[:mul, outreg, 3.14 * 10];
#t :move 0.1 0.1;
#t :rect
#  $[:mul, outreg, 0.1]
#  $[:reg, outreg, 0.1];
#t :move 0.3 0.3;
#t :rect
#  $[:mul, outreg, 0.1]
#  $[:reg, outreg, 0.1];
t :cmds;

!:global draw = {|1|
    # displayln "DRAW!" _ outreg " " [reg outreg];
    [reg outreg] * 90.0
}
