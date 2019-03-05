extern crate image;

use image::{GenericImage, GenericImageView};
use std::time::{SystemTime};

#[derive(Debug, Clone, Copy)]
struct Quadrant {
    x: u32,
    y: u32,
    height: u32,
    width: u32,
    score: f32,
}

fn main() {
    let src = image::open("images/girl_with_pearl.jpg").unwrap();
    let dim = src.dimensions();
    let mut dest = create_average_background_image(&src);
    let mut quads = Vec::new();
    let full_image_quad = Quadrant { x: 0, y: 0, width: dim.0, height: dim.1, score: -1.0 };
    quads.push(full_image_quad);
    let colour = get_quad_average_colour(&src, &full_image_quad);
    for i in 0..200 {
        let now = SystemTime::now();
        dest = break_up_worst_quad(&dest, &src, &mut quads);
        draw_outlines(&dest, &quads, colour).save(format!("images/girl_with_pearl_step{:?}.jpg", i+1)).unwrap();
        println!("time for step {:?}: {:?}\tnum of quads: {:?}", i+1, now.elapsed(), quads.len());
    }
}

fn break_up_worst_quad(dest: &image::DynamicImage, src: &image::DynamicImage, quads: &mut Vec<Quadrant>) -> image::DynamicImage {
    calculate_all_scores(dest, src, quads);
    let mut img = dest.clone();
    let worst_index = get_worst_quadrant(quads);
    let quad = quads.remove(worst_index);
    // If height/width is odd, we get weird banding left over
    // So that needs to be taken care of
    let mut w1 = quad.width/2;
    let w2 = quad.width/2;
    if quad.width%2 != 0 {
        w1 += 1;
    }
    let mut h1 = quad.height/2;
    let h2 = quad.height/2;
    if quad.height%2 != 0 {
        h1 += 1;
    }
    let quad1 = Quadrant {
        x: quad.x, y: quad.y,
        width: w1, height: h1,
        score: -1.0,
    };
    img = fix_quadrant(&img, src, quad1);
    quads.push(quad1);
    let quad2 = Quadrant {
        x: quad.x+w1, y: quad.y,
        width: w2, height: h1,
        score: -1.0,
    };
    img = fix_quadrant(&img, src, quad2);
    quads.push(quad2);
    let quad3 = Quadrant {
        x: quad.x+w1, y: quad.y+h1,
        width: w2, height: h2,
        score: -1.0,
    };
    img = fix_quadrant(&img, src, quad3);
    quads.push(quad3);
    let quad4 = Quadrant {
        x: quad.x, y: quad.y+h1,
        width: w1, height: h2,
        score: -1.0,
    };
    img = fix_quadrant(&img, src, quad4);
    quads.push(quad4);
    img
}

fn fix_quadrant(dest: &image::DynamicImage, src: &image::DynamicImage, quadrant: Quadrant) -> image::DynamicImage {
    let colour = get_quad_average_colour(src, &quadrant);
    set_quad_average_colour(dest, &quadrant, colour)
}

fn get_worst_quadrant(quads: &mut Vec<Quadrant>) -> usize {
    // Iterate through all the quads, find their scores
    // and return the index of the worst quad.
    let mut worst_index = 0;
    let mut worst_score = quads[0].score;
    for i in 1..quads.len() {
        if quads[i].score > worst_score {
            worst_index = i;
            worst_score = quads[i].score
        }
    }
    worst_index
}

fn calculate_all_scores(dest: &image::DynamicImage, src: &image::DynamicImage, quads: &mut Vec<Quadrant>) {
    for quadrant in quads {
        if quadrant.score < 0.0 {
            quadrant.score = calculate_quadrant_score(&quadrant, dest, src);
        }
    }
}

fn get_quad_average_colour(src: &image::DynamicImage, quadrant: &Quadrant) -> [u8; 3] {
    let mut avg_red = 0;
    let mut avg_green = 0;
    let mut avg_blue = 0;
    for x in quadrant.x..quadrant.x+quadrant.width {
        for y in quadrant.y..quadrant.y+quadrant.height {
            let p1 = src.get_pixel(x, y);
            avg_red += p1[0] as u32;
            avg_green += p1[1] as u32;
            avg_blue += p1[2] as u32;
        }
    }
    let total_pixels = quadrant.width*quadrant.height;
    [(avg_red/total_pixels) as u8, (avg_green/total_pixels) as u8, (avg_blue/total_pixels) as u8]
}

fn set_quad_average_colour(dest: &image::DynamicImage, quadrant: &Quadrant, color: [u8; 3]) -> image::DynamicImage {
    let mut img = dest.clone();
    for x in quadrant.x..quadrant.x+quadrant.width {
        for y in quadrant.y..quadrant.y+quadrant.height {
            img.put_pixel(x, y, image::Rgba{data: [color[0] as u8, color[1] as u8, color[2] as u8, 255]});
        }
    }
    img
}

fn calculate_quadrant_score(quadrant: &Quadrant, dest: &image::DynamicImage, src: &image::DynamicImage) -> f32 {
    let mut square_error = 0.0;
    for x in quadrant.x..quadrant.x+quadrant.width {
        for y in quadrant.y..quadrant.y+quadrant.height {
            let p1 = dest.get_pixel(x, y);
            let p2 = src.get_pixel(x, y);
            let r1 = p1[0] as f32;
            let g1 = p1[1] as f32;
            let b1 = p1[2] as f32;
            let r2 = p2[0] as f32;
            let g2 = p2[1] as f32;
            let b2 = p2[2] as f32;
            square_error += (r2-r1).powf(2.0);
            square_error += (g2-g1).powf(2.0);
            square_error += (b2-b1).powf(2.0);
        }
    }
    square_error.powf(0.5)
}

fn create_average_background_image(src: &image::DynamicImage) -> image::DynamicImage {
    let src_pixels = image_to_vector(src);
    let dim = src.dimensions();
    let image_width = dim.0;
    let image_height = dim.1;

    let mut avg_red = 0;
    let mut avg_green = 0;
    let mut avg_blue = 0;
    for pixel in src_pixels {
        avg_red += pixel[0] as u32;
        avg_green += pixel[1] as u32;
        avg_blue += pixel[2] as u32;
    }
    avg_red = avg_red / (image_width * image_height);
    avg_green = avg_green / (image_width * image_height);
    avg_blue = avg_blue / (image_width * image_height);
    println!("rgb = {:},{:},{:}", avg_red, avg_green, avg_blue);

    // TODO (03 Mar 2019 sam): See if there is a better way to do this
    // than replacing every pixel one by one...
    let mut dest = image::DynamicImage::new_rgb8(image_width, image_height);
    for x in  0..image_width {
        for y in 0..image_height {
            dest.put_pixel(x, y, image::Rgba{data: [avg_red as u8, avg_green as u8, avg_blue as u8, 255]});
        }
    }
    dest
}

fn draw_outlines(dest: &image::DynamicImage, quads: &Vec<Quadrant>, colour: [u8; 3]) -> image::DynamicImage {
    let mut img = dest.clone();
    for quadrant in quads {
        for x in quadrant.x..quadrant.x+quadrant.width {
            img.put_pixel(x, quadrant.y, image::Rgba{data: [colour[0], colour[1], colour[2], 255]});
        }
        for y in quadrant.y..quadrant.y+quadrant.height {
            img.put_pixel(quadrant.x, y, image::Rgba{data: [colour[0], colour[1], colour[2], 255]});
        }
    }
    for x in 0..img.dimensions().0 {
        img.put_pixel(x, img.dimensions().1-1, image::Rgba{data: [colour[0], colour[1], colour[2], 255]});
    }
    for y in 0..img.dimensions().1 {
        img.put_pixel(img.dimensions().0-1, y, image::Rgba{data: [colour[0], colour[1], colour[2], 255]});
    }
    img
}

fn image_to_vector(image: &image::DynamicImage) -> Vec<[u8; 3]> {
    let mut pixels = Vec::new();
    for pixel in image.pixels() {
        pixels.push([pixel.2[0], pixel.2[1], pixel.2[2]]);
    }
    pixels
}
