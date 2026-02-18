// Hypothesis:
// - During iterative deepening, due to move ordering, there won't significant changes from
//   previous iterations, or if so they will occur at the end of the PV.
// - Keep track of the "promising" moves outside of the PV, regenerate legal moves on the
//   fly. Avoid storing all the moves.

pub fn search() {}
