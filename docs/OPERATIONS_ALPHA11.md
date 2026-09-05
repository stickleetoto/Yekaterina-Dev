# alpha.11 operation expansion

alpha.11 adds 76 trust-oriented opcodes.

## verify.* (23)

`finite`, `finite_all`, `near`, `abs_change`, `rel_change`, `monotonic_inc`, `monotonic_dec`, `bounded`, `residual_l2`, `residual_rms`, `residual_max`, `in_range`, `positive`, `nonnegative`, `nonzero`, `percent_error`, `stable_digits`, `sign_consistent`, `shape_equal`, `convergence`, `grid_convergence`, `condition_ok`, `sequence_audit`.

## frame.* (20)

`vec3`, `point3`, `identity`, `rot_x`, `rot_y`, `rot_z`, `translate`, `compose`, `inverse`, `transform_vec`, `transform_point`, `add_vec`, `sub_vec`, `point_plus_vec`, `point_minus_point`, `dot`, `cross`, `distance`, `basis`, `same`.

## curve.* (16)

`is_closed`, `closure_error`, `signed_area`, `area`, `arc_length`, `bbox`, `orientation`, `self_intersections`, `is_simple_closed`, `min_segment`, `max_segment`, `duplicate_points`, `backtrack_count`, `centroid`, `point_at_fraction`, `audit`.

## predicate.* (17)

`point_in_polygon`, `point_on_segment`, `segments_intersect`, `point_in_circle`, `circle_in_circle`, `circles_intersect`, `aabb_intersects`, `aabb_contains`, `point_in_aabb`, `polygon_in_polygon`, `polygons_intersect`, `disjoint`, `circle_in_polygon`, `circle_polygon_clearance`, `point_polygon_clearance`, `segment_circle_intersect`, `polygon_bbox`.
