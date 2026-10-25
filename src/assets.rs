use citro2d::prelude::*;

pub fn load_sheet(path: &str) -> &'static SpriteSheet {
    Box::leak(Box::new(
        SpriteSheet::new(path).unwrap_or_else(|error| panic!("failed to load {path}: {error}")),
    ))
}

pub fn place(sprite: &mut Sprite<'_>, (x, y): (f32, f32)) {
    sprite.bounding_box = BoundingBox::with_center((x, y), sprite.bounding_box.size);
}
