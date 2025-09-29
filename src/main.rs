use anyhow::Result;
use opencv::{
    prelude::*,
    videoio,
    highgui,
    imgproc,
    core::{
        AlgorithmHint,
        Point,
        Rect,
        Scalar,
        Vector,
        absdiff
    }
};

#[derive(Debug)]
struct GridCell {
    point1: (i32, i32),
    point2: (i32, i32),
    motion: bool
}
impl GridCell {
    fn new(point1: (i32, i32), point2: (i32, i32), motion: bool) -> Self {
        GridCell {
            point1,
            point2,
            motion
        }
    }
}


fn check_cell<'a>(grid: &'a Vec<Vec<GridCell>>, rows: i32, cols: i32, row: i32, col: i32, current_motion: &mut Vec<&'a GridCell>, visited: &mut Vec<Vec<bool>>) {
    if row < 0 || row >= rows || col < 0 || col >= cols || visited[row as usize][col as usize] || !grid[row as usize][col as usize].motion {
        return;
    }
    
    visited[row as usize][col as usize] = true;
    current_motion.push(&grid[row as usize][col as usize]);
    
    let direction: Vec<(i32, i32)> = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
    
    for di in direction {
        check_cell(grid, rows, cols, row + di.0, col + di.1, current_motion, visited);
    }
}

fn get_connected_cells<'a>(grid: &'a mut Vec<Vec<GridCell>>, motion: &mut Vec<Vec<&'a GridCell>>) {
    let rows = grid.len();
    let cols = grid[0].len();
    
    let mut visited = vec![vec![false; cols]; rows];
    
    for row in 0..rows {
        for col in 0..cols {
            if grid[row][col].motion && !visited[row][col] {
                let mut current_motion: Vec<&'a GridCell> = Vec::new();
                check_cell(grid, rows as i32, cols as i32, row as i32 , col as i32, &mut current_motion, &mut visited);
                motion.push(current_motion);
            }
        }
    }
}

fn detect_motion(prev_frame: &Mat, current_frame: &Mat, contours: &mut Vector<Vector<Point>>, threshold: f64) {
    let mut prev_gray = Mat::default();
    let mut current_gray = Mat::default();
    let mut frame_diff = Mat::default();
    let mut motion_mask = Mat::default();
    
    imgproc::cvt_color(prev_frame, &mut prev_gray, imgproc::COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT).expect("Failed to gray scale prev frame");
    imgproc::cvt_color(current_frame, &mut current_gray, imgproc::COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT).expect("Failed to gray scale current frame");

    absdiff(&prev_gray, &current_gray, &mut frame_diff).expect("Failed to calculate absolute difference");
    let _ = imgproc::threshold(&frame_diff, &mut motion_mask, threshold, 255.0, 0);
    imgproc::find_contours(&motion_mask, contours, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_SIMPLE, Point::new(0, 0)).expect("Failed to find contours");
}

fn get_roi(contours: Vector<Vector<Point>>, grid: &mut Vec<Vec<GridCell>>, rois: &mut Vec<(Point, Point)>, cells: i32) {
    let mut motion: Vec<Vec<&GridCell>> = Vec::new();
    
    for contour in contours {
        for points in contour {
            grid[(points.y / cells) as usize][(points.x / cells) as usize].motion = true;
        }
    }
    
    get_connected_cells(grid, &mut motion);
    
    let mut point1 = Point::default();
    let mut point2 = Point::default();

    for mut m in motion {
        m.sort_by(|a, b| { a.point1.0.cmp(&b.point1.0) });
        point1.x = m[0].point1.0;

        m.reverse();
        point2.x = m[0].point2.0;
        
        m.sort_by(|a, b| { a.point1.1.cmp(&b.point1.1) });
        point1.y = m[0].point1.1;
    
        m.reverse();
        point2.y = m[0].point2.1;
        
        rois.push((point1, point2));
    }
}

fn run<'a, 'b>(frame: &Mat, prev_frame: &Mat, grid: &'a mut Vec<Vec<GridCell>>, rois: &mut Vec<(Point, Point)>, cells: i32) {
    let mut grid_y = (0, cells);
    let mut motion_contours: Vector<Vector<Point>> = Vector::default();

    for i in 0..frame.mat_size()[0] / cells {
        let mut grid_x = (0, cells);
        grid.push(Vec::new());
        
        for _ in 0..frame.mat_size()[1] / cells {
            grid[i as usize].push(GridCell::new(
                (grid_x.0, grid_y.0),
                (grid_x.1, grid_y.1),
                false
            ));
            grid_x = (grid_x.0 + cells, grid_x.1 + cells);
        }
        grid_y = (grid_y.0 + cells, grid_y.1 + cells);
    }
    
    detect_motion(prev_frame, frame, &mut motion_contours, 40.0);
    get_roi(motion_contours, grid, rois, cells);
}

fn main() -> Result<()> {
    // Open a GUI window
    highgui::named_window("window", highgui::WINDOW_AUTOSIZE)?;
    
    // Open the web-camera
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    // if you want to run it on an mp4 file use
    // let mut cam = videoio::VideoCapture::from_file("path/to/file.mp4", videoio::CAP_ANY)?;
    
    let mut frame = Mat::default(); // This array will store the webcam data
    let mut prev_frame: Option<Mat> = None; // This will store the prev frame for the difference
    
    loop {
        let mut grid: Vec<Vec<GridCell>> = Vec::new();
        let mut rois: Vec<(Point, Point)> = Vec::new(); // Stores the points for the overlapping boxes

        cam.read(&mut frame)?;

        if prev_frame.is_none() {
            prev_frame = Some(frame.clone());
        };

        run(&frame, &prev_frame.unwrap(),  &mut grid, &mut rois, 10);

        let mut draw_frame = frame.clone();
        prev_frame = Some(frame.clone());
        
        // Draw all Cells with detected motion in them
        for row in &grid {
            for cell in row {
                if cell.motion {
                    imgproc::rectangle(&mut draw_frame, Rect::new(cell.point1.0, cell.point1.1,  cell.point2.0 - cell.point1.0, cell.point2.1 - cell.point1.1), Scalar::new(0.0, 255.0, 0.0, 1.0), 1, 8, 0)?;
                }
            }
        }
       
        // Draw the rois
        for roi in &rois {
            imgproc::rectangle(&mut draw_frame, Rect::new(roi.0.x, roi.0.y, roi.1.x - roi.0.x, roi.1.y - roi.0.y), Scalar::new(255.0, 0.0, 0.0, 1.0), 1, 8, 0)?;
        }
        
        highgui::imshow("window", &draw_frame)?;

        let key = highgui::wait_key(1)?;
        if key == 113 {  // End program, when user presses 'q'
            break;
        }
    } 
    Ok(())
}