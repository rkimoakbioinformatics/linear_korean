use crate::error::*;
use crate::structs::FontTables;
use ahash::{HashMap, HashSet};
use write_fonts::tables::glyf::{Anchor, CompositeGlyph, Glyph, SimpleGlyph, Transform};

const DEFAULT_RASTER_RESOLUTION: i16 = 192;
const MAX_MASK_PIXELS: usize = 1_500_000;
const MAX_DEBUG_POINTS_PER_LAYER: usize = 40_000;

#[derive(Clone, Copy)]
struct Node {
    x: f32,
    y: f32,
    on_curve: bool,
}

type Segment = ((f32, f32), (f32, f32));

#[derive(Clone, Default)]
struct GlyphMask {
    filled_pixels: Vec<(i32, i32)>,
}

#[derive(Clone)]
pub struct CollisionRasterDebug {
    pub width: u32,
    pub height: u32,
    pub component_a: Vec<[u32; 2]>,
    pub component_b: Vec<[u32; 2]>,
    pub overlap: Vec<[u32; 2]>,
}

pub struct CollisionChecker {
    scale: f32,
    margin_px: i32,
    mask_cache: HashMap<u16, GlyphMask>,
}

impl CollisionChecker {
    pub fn new(glyph_width: i16, raster_resolution: Option<i16>) -> Result<Self, Error> {
        if glyph_width <= 0 {
            return Err(Error::Collision(CollisionError {
                msg: format!("Invalid glyph_width {} for collision checking", glyph_width),
                debug: None,
            }));
        }
        let resolution = raster_resolution.unwrap_or(DEFAULT_RASTER_RESOLUTION);
        if resolution <= 0 {
            return Err(Error::Collision(CollisionError {
                msg: format!(
                    "Invalid raster resolution {} for collision checking",
                    resolution
                ),
                debug: None,
            }));
        }
        Ok(Self {
            scale: resolution as f32 / glyph_width as f32,
            margin_px: 0,
            mask_cache: HashMap::default(),
        })
    }

    pub fn composite_collision_debug(
        &mut self,
        font_tables: &FontTables,
        glyph: &CompositeGlyph,
    ) -> Result<Option<CollisionRasterDebug>, Error> {
        let mut occupied_owner: HashMap<i64, usize> = HashMap::default();
        let mut transformed_masks: Vec<Vec<(i32, i32)>> = Vec::new();
        let mut stack: Vec<u16> = Vec::new();
        for (component_index, component) in glyph.components().iter().enumerate() {
            let (offset_x, offset_y) = component_offset(component)?;
            let shift_x_px = (offset_x as f32 * self.scale).round() as i32;
            let shift_y_px = (offset_y as f32 * self.scale).round() as i32;
            let source_glyph_id = component.glyph.to_u16();
            let source_mask = self.get_or_build_mask(font_tables, source_glyph_id, &mut stack)?;

            let transformed_mask = source_mask
                .filled_pixels
                .iter()
                .map(|(x, y)| (*x + shift_x_px, *y + shift_y_px))
                .collect::<Vec<_>>();

            let mut collided_owner: Option<usize> = None;
            let mut overlap_keys: HashSet<i64> = HashSet::default();
            for (world_x, world_y) in transformed_mask.iter() {
                if let Some(owner) =
                    find_occupied_owner(&occupied_owner, *world_x, *world_y, self.margin_px)
                {
                    collided_owner = Some(owner);
                    overlap_keys.insert(pixel_key(*world_x, *world_y));
                }
            }
            if let Some(owner_index) = collided_owner {
                return Ok(Some(build_collision_raster_debug(
                    transformed_masks[owner_index].as_slice(),
                    transformed_mask.as_slice(),
                    &overlap_keys,
                )));
            }

            for (world_x, world_y) in transformed_mask.iter() {
                occupied_owner
                    .entry(pixel_key(*world_x, *world_y))
                    .or_insert(component_index);
            }
            transformed_masks.push(transformed_mask);
        }
        Ok(None)
    }

    fn get_or_build_mask(
        &mut self,
        font_tables: &FontTables,
        glyph_id: u16,
        stack: &mut Vec<u16>,
    ) -> Result<GlyphMask, Error> {
        if let Some(existing) = self.mask_cache.get(&glyph_id) {
            return Ok(existing.clone());
        }
        if stack.contains(&glyph_id) {
            return Err(Error::Collision(CollisionError {
                msg: format!(
                    "Recursive composite glyph dependency detected while checking GID {}",
                    glyph_id
                ),
                debug: None,
            }));
        }
        stack.push(glyph_id);
        let mask = self.build_mask(font_tables, glyph_id, stack)?;
        stack.pop();
        self.mask_cache.insert(glyph_id, mask.clone());
        Ok(mask)
    }

    fn build_mask(
        &mut self,
        font_tables: &FontTables,
        glyph_id: u16,
        stack: &mut Vec<u16>,
    ) -> Result<GlyphMask, Error> {
        let glyph = match font_tables.glyphs.get(glyph_id as usize) {
            Some(v) => v,
            None => {
                return Err(Error::Collision(CollisionError {
                    msg: format!("No glyph at GID {} for collision checking", glyph_id),
                    debug: None,
                }));
            }
        };
        match glyph {
            Glyph::Empty => Ok(GlyphMask::default()),
            Glyph::Simple(simple) => self.build_simple_mask(simple),
            Glyph::Composite(composite) => self.build_composite_mask(font_tables, composite, stack),
        }
    }

    fn build_simple_mask(&self, glyph: &SimpleGlyph) -> Result<GlyphMask, Error> {
        if glyph.contours.is_empty() {
            return Ok(GlyphMask::default());
        }
        let x_min_px = (glyph.bbox.x_min as f32 * self.scale).floor() as i32;
        let y_min_px = (glyph.bbox.y_min as f32 * self.scale).floor() as i32;
        let x_max_px = (glyph.bbox.x_max as f32 * self.scale).ceil() as i32;
        let y_max_px = (glyph.bbox.y_max as f32 * self.scale).ceil() as i32;
        if x_max_px < x_min_px || y_max_px < y_min_px {
            return Ok(GlyphMask::default());
        }
        let width = (x_max_px - x_min_px + 1) as usize;
        let height = (y_max_px - y_min_px + 1) as usize;
        if width.saturating_mul(height) > MAX_MASK_PIXELS {
            return Err(Error::Collision(CollisionError {
                msg: format!(
                    "Mask raster size {}x{} exceeds limit for collision checking",
                    width, height
                ),
                debug: None,
            }));
        }
        let mut segments: Vec<Segment> = Vec::new();
        for contour in glyph.contours.iter() {
            append_contour_segments(contour, self.scale, &mut segments);
        }
        if segments.is_empty() {
            return Ok(GlyphMask::default());
        }
        let mut filled: Vec<(i32, i32)> = Vec::new();
        for y in y_min_px..=y_max_px {
            let py = y as f32 + 0.5;
            for x in x_min_px..=x_max_px {
                let px = x as f32 + 0.5;
                if winding_number(&segments, px, py) != 0 {
                    filled.push((x, y));
                }
            }
        }
        Ok(GlyphMask {
            filled_pixels: filled,
        })
    }

    fn build_composite_mask(
        &mut self,
        font_tables: &FontTables,
        glyph: &CompositeGlyph,
        stack: &mut Vec<u16>,
    ) -> Result<GlyphMask, Error> {
        let mut merged: HashSet<i64> = HashSet::default();
        for component in glyph.components() {
            let (offset_x, offset_y) = component_offset(component)?;
            let shift_x_px = (offset_x as f32 * self.scale).round() as i32;
            let shift_y_px = (offset_y as f32 * self.scale).round() as i32;
            let source_glyph_id = component.glyph.to_u16();
            let source_mask = self.get_or_build_mask(font_tables, source_glyph_id, stack)?;
            for (x, y) in source_mask.filled_pixels.iter() {
                merged.insert(pixel_key(*x + shift_x_px, *y + shift_y_px));
            }
        }
        let mut filled: Vec<(i32, i32)> = Vec::with_capacity(merged.len());
        for key in merged.into_iter() {
            filled.push(decode_pixel_key(key));
        }
        Ok(GlyphMask {
            filled_pixels: filled,
        })
    }
}

fn build_collision_raster_debug(
    component_a_pixels: &[(i32, i32)],
    component_b_pixels: &[(i32, i32)],
    overlap_keys: &HashSet<i64>,
) -> CollisionRasterDebug {
    let mut union_min_x = i32::MAX;
    let mut union_min_y = i32::MAX;
    let mut union_max_x = i32::MIN;
    let mut union_max_y = i32::MIN;
    for (x, y) in component_a_pixels.iter().chain(component_b_pixels.iter()) {
        union_min_x = union_min_x.min(*x);
        union_min_y = union_min_y.min(*y);
        union_max_x = union_max_x.max(*x);
        union_max_y = union_max_y.max(*y);
    }
    if union_min_x > union_max_x || union_min_y > union_max_y {
        return CollisionRasterDebug {
            width: 1,
            height: 1,
            component_a: Vec::new(),
            component_b: Vec::new(),
            overlap: Vec::new(),
        };
    }
    let width = (union_max_x - union_min_x + 1) as u32;
    let height = (union_max_y - union_min_y + 1) as u32;
    let normalize = |x: i32, y: i32| -> [u32; 2] {
        [
            (x - union_min_x) as u32,
            (union_max_y - y) as u32, // font y-axis points upward
        ]
    };
    let step_a = sample_step(component_a_pixels.len(), MAX_DEBUG_POINTS_PER_LAYER);
    let mut component_a = component_a_pixels
        .iter()
        .enumerate()
        .filter(|(index, _)| index % step_a == 0)
        .map(|(_, (x, y))| normalize(*x, *y))
        .collect::<Vec<_>>();
    let step_b = sample_step(component_b_pixels.len(), MAX_DEBUG_POINTS_PER_LAYER);
    let mut component_b = component_b_pixels
        .iter()
        .enumerate()
        .filter(|(index, _)| index % step_b == 0)
        .map(|(_, (x, y))| normalize(*x, *y))
        .collect::<Vec<_>>();
    let step_overlap = sample_step(overlap_keys.len(), MAX_DEBUG_POINTS_PER_LAYER);
    let mut overlap = overlap_keys
        .iter()
        .enumerate()
        .filter(|(index, _)| index % step_overlap == 0)
        .map(|(_, key)| {
            let (x, y) = decode_pixel_key(*key);
            normalize(x, y)
        })
        .collect::<Vec<_>>();
    component_a.sort_unstable();
    component_b.sort_unstable();
    overlap.sort_unstable();
    CollisionRasterDebug {
        width,
        height,
        component_a,
        component_b,
        overlap,
    }
}

fn sample_step(total: usize, max_samples: usize) -> usize {
    if total <= max_samples || max_samples == 0 {
        return 1;
    }
    (total as f32 / max_samples as f32).ceil() as usize
}

fn component_offset(component: &write_fonts::tables::glyf::Component) -> Result<(i16, i16), Error> {
    if !is_identity_transform(&component.transform) {
        return Err(Error::Collision(CollisionError {
            msg: format!(
                "Non-identity component transform is not supported for collision checking (GID {})",
                component.glyph.to_u16()
            ),
            debug: None,
        }));
    }
    match component.anchor {
        Anchor::Offset { x, y } => Ok((x, y)),
        Anchor::Point { .. } => Err(Error::Collision(CollisionError {
            msg: format!(
                "Point-anchor component is not supported for collision checking (GID {})",
                component.glyph.to_u16()
            ),
            debug: None,
        })),
    }
}

fn is_identity_transform(transform: &Transform) -> bool {
    let eps = 1e-6f32;
    (transform.xx.to_f32() - 1.0).abs() < eps
        && (transform.yy.to_f32() - 1.0).abs() < eps
        && transform.xy.to_f32().abs() < eps
        && transform.yx.to_f32().abs() < eps
}

fn pixel_key(x: i32, y: i32) -> i64 {
    ((x as i64) << 32) | (y as u32 as i64)
}

fn decode_pixel_key(key: i64) -> (i32, i32) {
    ((key >> 32) as i32, key as i32)
}

fn find_occupied_owner(
    occupied_owner: &HashMap<i64, usize>,
    x: i32,
    y: i32,
    margin: i32,
) -> Option<usize> {
    if margin <= 0 {
        return occupied_owner.get(&pixel_key(x, y)).copied();
    }
    for dy in -margin..=margin {
        for dx in -margin..=margin {
            if let Some(owner) = occupied_owner.get(&pixel_key(x + dx, y + dy)) {
                return Some(*owner);
            }
        }
    }
    None
}

fn append_contour_segments(
    contour: &write_fonts::tables::glyf::Contour,
    scale: f32,
    out: &mut Vec<Segment>,
) {
    let points: Vec<Node> = contour
        .iter()
        .map(|point| Node {
            x: point.x as f32 * scale,
            y: point.y as f32 * scale,
            on_curve: point.on_curve,
        })
        .collect();
    if points.len() < 2 {
        return;
    }
    let mut expanded: Vec<Node> = Vec::with_capacity(points.len() * 2);
    for i in 0..points.len() {
        let current = points[i];
        let next = points[(i + 1) % points.len()];
        expanded.push(current);
        if !current.on_curve && !next.on_curve {
            expanded.push(Node {
                x: (current.x + next.x) * 0.5,
                y: (current.y + next.y) * 0.5,
                on_curve: true,
            });
        }
    }
    if expanded.is_empty() {
        return;
    }
    let start = if expanded[0].on_curve {
        expanded[0]
    } else {
        let last = expanded[expanded.len() - 1];
        if last.on_curve {
            last
        } else {
            Node {
                x: (last.x + expanded[0].x) * 0.5,
                y: (last.y + expanded[0].y) * 0.5,
                on_curve: true,
            }
        }
    };
    let mut current = (start.x, start.y);
    let mut index = if expanded[0].on_curve { 1usize } else { 0usize };
    let mut remaining = expanded.len();
    while remaining > 0 {
        let point = expanded[index % expanded.len()];
        if point.on_curve {
            push_line(current, (point.x, point.y), out);
            current = (point.x, point.y);
            index += 1;
            remaining -= 1;
        } else {
            let next = expanded[(index + 1) % expanded.len()];
            let end = if next.on_curve {
                (next.x, next.y)
            } else {
                ((point.x + next.x) * 0.5, (point.y + next.y) * 0.5)
            };
            push_quadratic(current, (point.x, point.y), end, out);
            current = end;
            if next.on_curve {
                index += 2;
                remaining = remaining.saturating_sub(2);
            } else {
                index += 1;
                remaining -= 1;
            }
        }
    }
}

fn push_line(from: (f32, f32), to: (f32, f32), out: &mut Vec<Segment>) {
    if (from.0 - to.0).abs() < 1e-6 && (from.1 - to.1).abs() < 1e-6 {
        return;
    }
    out.push((from, to));
}

fn push_quadratic(from: (f32, f32), control: (f32, f32), to: (f32, f32), out: &mut Vec<Segment>) {
    let flatness = distance_to_line(control, from, to);
    let mut steps = ((flatness / 0.25).ceil() as usize).max(4);
    if steps > 48 {
        steps = 48;
    }
    let mut previous = from;
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let mt = 1.0 - t;
        let x = mt * mt * from.0 + 2.0 * mt * t * control.0 + t * t * to.0;
        let y = mt * mt * from.1 + 2.0 * mt * t * control.1 + t * t * to.1;
        let current = (x, y);
        push_line(previous, current, out);
        previous = current;
    }
}

fn distance_to_line(point: (f32, f32), line_start: (f32, f32), line_end: (f32, f32)) -> f32 {
    let (x0, y0) = point;
    let (x1, y1) = line_start;
    let (x2, y2) = line_end;
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
        return ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt();
    }
    ((dy * x0 - dx * y0 + x2 * y1 - y2 * x1).abs()) / (dx * dx + dy * dy).sqrt()
}

fn winding_number(segments: &[Segment], x: f32, y: f32) -> i32 {
    let mut winding = 0i32;
    for ((x0, y0), (x1, y1)) in segments.iter() {
        if *y0 <= y {
            if *y1 > y && is_left((*x0, *y0), (*x1, *y1), (x, y)) > 0.0 {
                winding += 1;
            }
        } else if *y1 <= y && is_left((*x0, *y0), (*x1, *y1), (x, y)) < 0.0 {
            winding -= 1;
        }
    }
    winding
}

fn is_left(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (p.0 - a.0) * (b.1 - a.1)
}
