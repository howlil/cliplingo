use super::{ScreenPoint, ScreenRect, ScreenSize};

pub fn place_popup(
    anchor: &ScreenRect,
    popup: &ScreenSize,
    work_area: &ScreenRect,
    margin: f64,
) -> ScreenPoint {
    let max_x = (work_area.right() - popup.width).max(work_area.x);
    let max_y = (work_area.bottom() - popup.height).max(work_area.y);

    let x = anchor.right().clamp(work_area.x, max_x);
    let below = anchor.bottom() + margin;
    let preferred_y = if below + popup.height <= work_area.bottom() {
        below
    } else {
        anchor.y - popup.height - margin
    };
    let y = preferred_y.clamp(work_area.y, max_y);

    ScreenPoint { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> ScreenRect {
        ScreenRect {
            x,
            y,
            width,
            height,
        }
    }

    fn size(width: f64, height: f64) -> ScreenSize {
        ScreenSize { width, height }
    }

    #[test]
    fn places_below_selection_when_it_fits() {
        let result = place_popup(
            &rect(100.0, 100.0, 80.0, 20.0),
            &size(420.0, 180.0),
            &rect(0.0, 0.0, 1920.0, 1040.0),
            8.0,
        );
        assert_eq!(result, ScreenPoint { x: 180.0, y: 128.0 });
    }

    #[test]
    fn flips_above_when_bottom_would_overflow() {
        let result = place_popup(
            &rect(100.0, 900.0, 80.0, 20.0),
            &size(420.0, 180.0),
            &rect(0.0, 0.0, 1920.0, 1040.0),
            8.0,
        );
        assert_eq!(result.y, 712.0);
    }

    #[test]
    fn clamps_to_right_edge() {
        let result = place_popup(
            &rect(1850.0, 100.0, 40.0, 20.0),
            &size(420.0, 180.0),
            &rect(0.0, 0.0, 1920.0, 1040.0),
            8.0,
        );
        assert_eq!(result.x, 1500.0);
    }

    #[test]
    fn respects_negative_coordinate_work_area() {
        let result = place_popup(
            &rect(-800.0, 200.0, 100.0, 20.0),
            &size(420.0, 180.0),
            &rect(-1920.0, 0.0, 1920.0, 1040.0),
            8.0,
        );
        assert_eq!(result.x, -700.0);
    }
}
