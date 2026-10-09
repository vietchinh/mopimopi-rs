//! Unit tests for `graph_bars`. They are compiled as a child module of that file but live here, outside `src/`.
//!
//! The animation starts from a transform that must put a bar exactly where it was before, whatever it does next: these
//! check that by applying the transform to the bar's new geometry and comparing with the old one.

use super::*;

fn g(edge: f64, width: f64) -> BarGeometry { BarGeometry { edge, width } }

/// Where the bar appears with the starting transform applied: `translateX(dx% of its width) scaleX(sx)` about its anchored edge.
fn shown_at_start(new: BarGeometry, flip: Flip) -> BarGeometry {
    g(new.edge + flip.dx / 100.0 * new.width, flip.sx * new.width)
}

fn close(a: BarGeometry, b: BarGeometry) -> bool { (a.edge - b.edge).abs() < 1e-9 && (a.width - b.width).abs() < 1e-9 }

#[test]
fn a_bar_that_did_not_move_is_not_animated() {
    assert_eq!(flip(Some(g(0.0, 40.0)), g(0.0, 40.0)), None);
    assert_eq!(flip(Some(g(12.5, 8.0)), g(12.5, 8.0)), None);
}

#[test]
fn a_new_bar_grows_out_of_its_anchored_edge() {
    assert_eq!(flip(None, g(0.0, 40.0)), Some(Flip { dx: 0.0, sx: 0.0 }));
    assert_eq!(flip(Some(g(5.0, 0.0)), g(5.0, 12.0)), Some(Flip { dx: 0.0, sx: 0.0 })); // one that had shrunk to nothing
}

#[test]
fn a_bar_that_ends_up_with_no_width_just_snaps() {
    assert_eq!(flip(Some(g(0.0, 40.0)), g(0.0, 0.0)), None);
}

#[test]
fn the_main_bar_growing_and_shrinking_is_a_scale_about_its_left_edge() {
    let grow = flip(Some(g(0.0, 40.0)), g(0.0, 90.0)).unwrap();
    assert_eq!(grow.dx, 0.0);
    assert!((grow.sx - 40.0 / 90.0).abs() < 1e-12);
    let shrink = flip(Some(g(0.0, 90.0)), g(0.0, 40.0)).unwrap();
    assert!((shrink.sx - 2.25).abs() < 1e-12);
}

#[test]
fn the_starting_transform_always_puts_the_bar_back_where_it_was() {
    // every combination of an old and a new geometry from a grid, either side
    let edges = [0.0, 3.0, 10.0, 47.5, 80.0];
    let widths = [0.5, 1.0, 7.0, 25.0, 60.0, 100.0];
    let mut checked = 0;
    for &old_edge in &edges {
        for &old_width in &widths {
            for &new_edge in &edges {
                for &new_width in &widths {
                    let (old, new) = (g(old_edge, old_width), g(new_edge, new_width));
                    if let Some(flip) = flip(Some(old), new) {
                        assert!(close(shown_at_start(new, flip), old), "{old:?} -> {new:?} via {flip:?}");
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 500);
}

#[test]
fn small_bars_sit_side_by_side_from_the_left() {
    // main bar 50% of the row; small bars of 20% and 30% of it
    let geometry = small_bar_geometry(50.0, &[20, 30], false).unwrap();
    assert_eq!(geometry, [g(0.0, 10.0), g(10.0, 15.0)]);
}

#[test]
fn small_bars_floated_right_are_measured_from_the_right_edge() {
    let geometry = small_bar_geometry(50.0, &[20, 30], true).unwrap();
    // the first ends at the container's right edge (50% of the row), the next ends where the first begins
    assert_eq!(geometry, [g(50.0, 10.0), g(40.0, 15.0)]);
}

#[test]
fn small_bars_that_would_wrap_are_left_to_the_browser() {
    assert_eq!(small_bar_geometry(50.0, &[60, 50], false), None);
    assert!(small_bar_geometry(50.0, &[60, 40], false).is_some()); // exactly full still fits on one line
}

#[test]
fn a_following_small_bar_slides_when_the_bar_before_it_changes_width() {
    // the overheal bar grows from 10% to 30% of the container, pushing the shield bar (20% wide) along, on either side
    for on_right in [false, true] {
        let old = small_bar_geometry(60.0, &[10, 20], on_right).unwrap()[1];
        let new = small_bar_geometry(60.0, &[30, 20], on_right).unwrap()[1];
        let flip = flip(Some(old), new).expect("the shield bar moved");
        assert!(close(shown_at_start(new, flip), old), "on_right={on_right}");
        assert!((flip.sx - 1.0).abs() < 1e-12, "it only slides, its width is unchanged");
    }
}

#[test]
fn a_bar_follows_the_main_bar_when_that_changes_width() {
    // the container (as wide as the main bar) goes from 40% to 80%: every small bar scales and, but for the first
    // one on the left, moves too
    for on_right in [false, true] {
        let before = small_bar_geometry(40.0, &[25, 25], on_right).unwrap();
        let after = small_bar_geometry(80.0, &[25, 25], on_right).unwrap();
        for i in 0..2 {
            let flip = flip(Some(before[i]), after[i]).unwrap();
            assert!(close(shown_at_start(after[i], flip), before[i]), "bar {i}, on_right={on_right}");
        }
    }
}
